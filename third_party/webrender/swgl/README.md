# swgl

Software OpenGL implementation for WebRender

## Overview
This is a relatively simple single threaded software rasterizer designed
for use by WebRender. It will shade one quad at a time using a 4xf32 vector
with one vertex per lane. It rasterizes quads usings spans and shades that
span 4 pixels at a time.

## Building
clang-cl is required to build on Windows. This can be done by installing
the llvm binaries from https://releases.llvm.org/ and adding the installation
to the path with something like `set PATH=%PATH%;C:\Program Files\LLVM\bin`.
Then `set CC=clang-cl` and `set CXX=clang-cl`. That should be sufficient
for `cc-rs` to use `clang-cl` instead of `cl`.

## Extensions
SWGL contains a number of OpenGL and GLSL extensions designed to both ease
integration with WebRender and to help accelerate span rasterization.

GLSL extension intrinsics are generally prefixed with `swgl_` to distinguish
them from other items in the GLSL namespace.

Inside GLSL, the `SWGL` preprocessor token is defined so that usage of SWGL
extensions may be conditionally compiled.

```
void swgl_drawSpanRGBA8();
void swgl_drawSpanR8();

int swgl_SpanLength;
int swgl_StepSize;

mixed swgl_interpStep(mixed varying_input);
void swgl_stepInterp();
```

SWGL's default fragment processing calls a fragment shader's `main` function
on groups of fragments in units of `swgl_StepSize`. On return, the value of
gl_FragColor is read, packed to an appropriate pixel format, and sent to the
blend stage for output to the destination framebuffer. This can be inefficient
for some types of fragment shaders, such as those that must lookup from a
texture and immediately output it, unpacking the texels only to subsequently
repack them at cost. Also, various per-fragment conditions in the shader might
need to be repeatedly checked, even though they are actually constant over
the entire primitive.

To work around this inefficiency, SWGL allows fragments to optionally be
processed over entire spans. This can both side-step the packing inefficiency
as well as more efficiently deal with conditions that remain constant over an
entire span. SWGL also introduces various specialized intrinsics for more
efficiently dealing with certain types of primitive spans with optimal
fixed-function processing.

Inside a fragment shader, a `swgl_drawSpan` function may be defined to override
the normal fragment processing for that fragment shader. The function must then
call some form of `swgl_commit` intrinsic to actually output to the destination
framebuffer via the blend stage, as normal fragment processing does not take
place otherwise as would have happened in `main`. This function is used by the
rasterizer to process an entire span of fragments that have passed the depth
test (if applicable) and clipping, but have not yet been output to the blend
stage.

The amount of fragments within the span to be processed is designated by
`swgl_SpanLength` and is always aligned to units of `swgl_StepSize`.
The size of a group of fragments in terms of which `swgl_commit` intrinsics
process and output fragments is designated by `swgl_StepSize`. The
`swgl_commit` intrinsics will deduct accordingly from `swgl_SpanLength` in
units of `swgl_StepSize` to reflect the fragments actually processed, which
may be less than the entire span or up to the entire span.

Fragments should be output until `swgl_SpanLength` becomes zero to process the
entire span. If `swgl_drawSpan` returns while leaving any fragments unprocessed,
the remaining fragments will be processed as normal by the fragment shader's
`main` function. This can be used to conditionally handle certain fast-paths
in a fragment shader by otherwise defaulting to the `main` function if
`swgl_drawSpan` can't appropriately process some or all of the fragments.

The values of any varying inputs to the fragment shader will be set to their
values for the start of the span, but do not automatically update over the
the course of a span within a given call to `swgl_drawSpan`. The
`swgl_interpStep` intrinsic may be used to get the derivative per `swgl_StepSize`
group of fragments of a varying input so that the caller may update such
variables manually if desired or otherwise use that information for processing.
The `swgl_stepInterp` intrinsic forces all such varying inputs to advance by
a single step.

The RGBA8 version will be used when the destination framebuffer is RGBA8 format,
and the R8 version will be used when the destination framebuffer is R8. Various
other intrinsics described below may have restrictions on whether they can be
used only with a certain destination framebuffer format and are noted as such if
so.

```
void swgl_clipMask(sampler2D mask, vec2 offset, vec2 bb_origin, vec2 bb_size);
```

When called from the the vertex shader, this specifies a clip mask texture to
be used to mask the currently drawn primitive while blending is enabled. This
mask will only apply to the current primitive.

The mask must be an R8 texture that will be interpreted as alpha weighting
applied to the source pixel prior to the blend stage. It is sampled 1:1 with
nearest filtering without any applied transform. The given offset specifies
the positioning of the clip mask relative to the framebuffer's viewport.

The supplied bounding box constrains sampling of the clip mask to only fall
within the given rectangle, specified relative to the clip mask offset.
Anything falling outside this rectangle will be clipped entirely. If the
rectangle is empty, then the clip mask will be ignored.

```
void swgl_antiAlias(int edgeMask);
```

When called from the vertex shader, this enables anti-aliasing for the
currently drawn primitive while blending is enabled. This setting will only
apply to the current primitive. Anti-aliasing will be applied only to the
edges corresponding to bits supplied in the mask. For simple use-cases,
the edge mask can be set to all 1 bits to enable AA for the entire quad.

The order of the bits in the edge mask must match the winding order in which
the vertices are output in the vertex shader if processed as a quad, so that
the edge ends on that vertex. The easiest way to understand this ordering
is that for a rectangle (x0,y0,x1,y1) then the edge Nth edge bit corresponds
to the edge where Nth coordinate in the rectangle is constant.

SWGL tries to use an anti-aliasing method that is reasonably close to WR's
signed-distance field approximation. WR would normally try to discern the
2D local-space coordinates of a given destination pixel relative to the
2D local-space bounding rectangle of a primitive. It then uses the screen-
space derivative to try to determine the how many local-space units equate
to a distance of around one screen-space pixel. A distance approximation
of coverage is then used based on the distance in local-space from the
the current pixel's center, roughly at half-intensity at pixel center
and ranging to zero or full intensity within a radius of half a pixel
away from the center. To account for AAing going outside the normal geometry
boundaries of the primitive, WR has to extrude the primitive by a local-space
estimate to allow some AA to happen within the extruded region.

SWGL can ultimately do this approximation more simply and get around the
extrusion limitations by just ensuring spans encompass any pixel that is
partially covered when computing span boundaries. Further, since SWGL already
knows the slope of an edge and the coordinate of the span relative to the span
boundaries, finding the partial coverage of a given span becomes easy to do
without requiring any extra interpolants to track against local-space bounds.
Essentially, SWGL just performs anti-aliasing on the actual geometry bounds,
but when the pixels on a span's edge are determined to be partially covered
during span rasterization, it uses the same distance field method as WR on
those span boundary pixels to estimate the coverage based on edge slope.

```
void swgl_commitTextureLinearRGBA8(sampler, vec2 uv, vec4 uv_bounds);
void swgl_commitTextureLinearR8(sampler, vec2 uv, vec4 uv_bounds);
void swgl_commitTextureLinearR8ToRGBA8(sampler, vec2 uv, vec4 uv_bounds);

void swgl_commitTextureLinearColorRGBA8(sampler, vec2 uv, vec4 uv_bounds, vec4|float color);
void swgl_commitTextureLinearColorR8(sampler, vec2 uv, vec4 uv_bounds, vec4|float color);
void swgl_commitTextureLinearColorR8ToRGBA8(sampler, vec2 uv, vec4 uv_bounds, vec4|float color);

void swgl_commitTextureLinearRepeatRGBA8(sampler, vec2 uv, vec2 tile_repeat, vec4 uv_repeat, vec4 uv_bounds);
void swgl_commitTextureLinearRepeatColorRGBA8(sampler, vec2 uv, vec2 tile_repeat, vec4 uv_repeat, vec4 uv_bounds, vec4|float color);

void swgl_commitTextureNearestRGBA8(sampler, vec2 uv, vec4 uv_bounds);
void swgl_commitTextureNearestColorRGBA8(sampler, vec2 uv, vec4 uv_bounds, vec4|float color);

void swgl_commitTextureNearestRepeatRGBA8(sampler, vec2 uv, vec2 tile_repeat, vec4 uv_repeat, vec4 uv_bounds);
void swgl_commitTextureNearestRepeatColorRGBA8(sampler, vec2 uv, vec2 tile_repeat, vec4 uv_repeat, vec4 uv_bounds, vec4|float color);

void swgl_commitTextureRGBA8(sampler, vec2 uv, vec4 uv_bounds);
void swgl_commitTextureColorRGBA8(sampler, vec2 uv, vec4 uv_bounds, vec4|float color);

void swgl_commitTextureRepeatRGBA8(sampler, vec2 uv, vec2 tile_repeat, vec4 uv_repeat, vec4 uv_bounds);
void swgl_commitTextureRepeatColorRGBA8(sampler, vec2 uv, vec2 tile_repeat, vec4 uv_repeat, vec4 uv_bounds, vec4|float color);

void swgl_commitPartialTextureLinearR8(int len, sampler, vec2 uv, vec4 uv_bounds);
void swgl_commitPartialTextureLinearInvertR8(int len, sampler, vec2 uv, vec4 uv_bounds);
```

Samples and commits an entire span of texture starting at the given uv and
within the supplied uv bounds. The color variations also accept a supplied color
that modulates the result.

The RGBA8 versions may only be used to commit within `swgl_drawSpanRGBA8`, and
the R8 versions may only be used to commit within `swgl_drawSpanR8`. The R8ToRGBA8
versions may be used to sample from an R8 source while committing to an RGBA8
framebuffer.

The Linear variations use a linear filter that bilinearly interpolates between
the four samples near the pixel. The Nearest variations use a nearest filter
that chooses the closest aliased sample to the center of the pixel. If neither
Linear nor Nearest is specified in the `swgl_commitTexture` variation name, then
it will automatically select either the Linear or Nearest variation depending
on the sampler's specified filter.

The Repeat variations require an optional repeat rect that specifies how to
scale and offset the UVs, assuming the UVs are normalized to repeat in the
range 0 to 1. For NearestRepeat variations, it is assumed the repeat rect is
always within the bounds. The tile repeat limit, if non-zero, specifies the
maximum number of repetitions allowed.

The Partial variations allow committing only a sub-span rather the entire
remaining span. These are currently only implemented in linear R8 variants
for optimizing clip shaders in WebRender. The Invert variant of these is
useful for implementing clip-out modes by inverting the source texture value.

```
// Premultiplied alpha over blend, but with source color set to source alpha modulated with a constant color.
void swgl_blendDropShadow(vec4 color);
// Premultiplied alpha over blend, but treats the source as a subpixel mask modulated with a constant color.
void swgl_blendSubpixelText(vec4 color);
```

SWGL allows overriding the blend mode per-primitive by calling `swgl_blend`
intrinsics in the vertex shader. The existing blend mode set by the GL is
replaced with the one specified by the intrinsic for the current primitive.
The blend mode will be reset to the blend mode set by the GL for the next
primitive after the current one, even within the same draw call.

