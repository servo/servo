/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define WR_VERTEX_SHADER_MAIN_FUNCTION text_shader_main_vs
#define WR_BRUSH_FS_FUNCTION text_brush_fs
#define WR_BRUSH_VS_FUNCTION text_brush_vs
// The text brush shader doesn't use this but the macro must be defined
// to compile the brush infrastructure.
#define VECS_PER_SPECIFIC_BRUSH 0

#include shared,prim_shared

#ifdef WR_VERTEX_SHADER
// Forward-declare the text vertex shader's main entry-point before including
// the brush shader.
void text_shader_main_vs(
    Instance instance,
    PrimitiveHeader ph,
    Transform transform,
    PictureTask task,
    ClipArea clip_area
);
#endif

#include brush

#define V_COLOR             flat_varying_vec4_0
#define V_MASK_SWIZZLE      flat_varying_vec4_1.xy
// Normalized bounds of the source image in the texture.
#define V_UV_BOUNDS         flat_varying_vec4_2

// Interpolated UV coordinates to sample.
#define V_UV                varying_vec4_0.xy
#define V_LAYER             varying_vec4_0.z


#ifdef WR_FEATURE_GLYPH_TRANSFORM
#define V_UV_CLIP           varying_vec4_1
#endif

#ifdef WR_VERTEX_SHADER

#define VECS_PER_TEXT_RUN           2
#define GLYPHS_PER_GPU_BLOCK        2U

#ifdef WR_FEATURE_GLYPH_TRANSFORM
RectWithSize transform_rect(RectWithSize rect, mat2 transform) {
    vec2 center = transform * (rect.p0 + rect.size * 0.5);
    vec2 radius = mat2(abs(transform[0]), abs(transform[1])) * (rect.size * 0.5);
    return RectWithSize(center - radius, radius * 2.0);
}

bool rect_inside_rect(RectWithSize little, RectWithSize big) {
    return all(lessThanEqual(vec4(big.p0, little.p0 + little.size),
                             vec4(little.p0, big.p0 + big.size)));
}
#endif //WR_FEATURE_GLYPH_TRANSFORM

struct Glyph {
    vec2 offset;
};

Glyph fetch_glyph(int specific_prim_address,
                  int glyph_index) {
    // Two glyphs are packed in each texel in the GPU cache.
    int glyph_address = specific_prim_address +
                        VECS_PER_TEXT_RUN +
                        int(uint(glyph_index) / GLYPHS_PER_GPU_BLOCK);
    vec4 data = fetch_from_gpu_cache_1(glyph_address);
    // Select XY or ZW based on glyph index.
    // We use "!= 0" instead of "== 1" here in order to work around a driver
    // bug with equality comparisons on integers.
    vec2 glyph = mix(data.xy, data.zw,
                     bvec2(uint(glyph_index) % GLYPHS_PER_GPU_BLOCK != 0U));

    return Glyph(glyph);
}

struct GlyphResource {
    vec4 uv_rect;
    float layer;
    vec2 offset;
    float scale;
};

GlyphResource fetch_glyph_resource(int address) {
    vec4 data[2] = fetch_from_gpu_cache_2(address);
    return GlyphResource(data[0], data[1].x, data[1].yz, data[1].w);
}

struct TextRun {
    vec4 color;
    vec4 bg_color;
};

TextRun fetch_text_run(int address) {
    vec4 data[2] = fetch_from_gpu_cache_2(address);
    return TextRun(data[0], data[1]);
}

vec2 get_snap_bias(int subpx_dir) {
    // In subpixel mode, the subpixel offset has already been
    // accounted for while rasterizing the glyph. However, we
    // must still round with a subpixel bias rather than rounding
    // to the nearest whole pixel, depending on subpixel direciton.
    switch (subpx_dir) {
        case SUBPX_DIR_NONE:
        default:
            return vec2(0.5);
        case SUBPX_DIR_HORIZONTAL:
            // Glyphs positioned [-0.125, 0.125] get a
            // subpx position of zero. So include that
            // offset in the glyph position to ensure
            // we round to the correct whole position.
            return vec2(0.125, 0.5);
        case SUBPX_DIR_VERTICAL:
            return vec2(0.5, 0.125);
        case SUBPX_DIR_MIXED:
            return vec2(0.125);
    }
}

void text_shader_main_vs(
    Instance instance,
    PrimitiveHeader ph,
    Transform transform,
    PictureTask task,
    ClipArea clip_area
) {
    int glyph_index = instance.segment_index;
    int subpx_dir = (instance.flags >> 8) & 0xff;
    int color_mode = instance.flags & 0xff;

    // Note that the reference frame relative offset is stored in the prim local
    // rect size during batching, instead of the actual size of the primitive.
    TextRun text = fetch_text_run(ph.specific_prim_address);
    vec2 text_offset = ph.local_rect.size;

    if (color_mode == COLOR_MODE_FROM_PASS) {
        color_mode = uMode;
    }

    // Note that the unsnapped reference frame relative offset has already
    // been subtracted from the prim local rect origin during batching.
    // It was done this way to avoid pushing both the snapped and the
    // unsnapped offsets to the shader.
    Glyph glyph = fetch_glyph(ph.specific_prim_address, glyph_index);
    glyph.offset += ph.local_rect.p0;

    GlyphResource res = fetch_glyph_resource(instance.resource_address);

    vec2 snap_bias = get_snap_bias(subpx_dir);

    // Glyph space refers to the pixel space used by glyph rasterization during frame
    // building. If a non-identity transform was used, WR_FEATURE_GLYPH_TRANSFORM will
    // be set. Otherwise, regardless of whether the raster space is LOCAL or SCREEN,
    // we ignored the transform during glyph rasterization, and need to snap just using
    // the device pixel scale and the raster scale.
#ifdef WR_FEATURE_GLYPH_TRANSFORM
    // Transform from local space to glyph space.
    mat2 glyph_transform = mat2(transform.m) * task.device_pixel_scale;
    vec2 glyph_translation = transform.m[3].xy * task.device_pixel_scale;

    // Transform from glyph space back to local space.
    mat2 glyph_transform_inv = inverse(glyph_transform);

    // Glyph raster pixels include the impact of the transform. This path can only be
    // entered for 3d transforms that can be coerced into a 2d transform; they have no
    // perspective, and have a 2d inverse. This is a looser condition than axis aligned
    // transforms because it also allows 2d rotations.
    vec2 raster_glyph_offset = floor(glyph_transform * glyph.offset + snap_bias);

    // We want to eliminate any subpixel translation in device space to ensure glyph
    // snapping is stable for equivalent glyph subpixel positions. Note that we must take
    // into account the translation from the transform for snapping purposes.
    vec2 raster_text_offset = floor(glyph_transform * text_offset + glyph_translation + 0.5) - glyph_translation;

    // Compute the glyph rect in glyph space.
    RectWithSize glyph_rect = RectWithSize(res.offset + raster_glyph_offset + raster_text_offset,
                                           res.uv_rect.zw - res.uv_rect.xy);

    // The glyph rect is in glyph space, so transform it back to local space.
    RectWithSize local_rect = transform_rect(glyph_rect, glyph_transform_inv);

    // Select the corner of the glyph's local space rect that we are processing.
    vec2 local_pos = local_rect.p0 + local_rect.size * aPosition.xy;

    // If the glyph's local rect would fit inside the local clip rect, then select a corner from
    // the device space glyph rect to reduce overdraw of clipped pixels in the fragment shader.
    // Otherwise, fall back to clamping the glyph's local rect to the local clip rect.
    if (rect_inside_rect(local_rect, ph.local_clip_rect)) {
        local_pos = glyph_transform_inv * (glyph_rect.p0 + glyph_rect.size * aPosition.xy);
    }
#else
    float raster_scale = float(ph.user_data.x) / 65535.0;

    // Scale in which the glyph is snapped when rasterized.
    float glyph_raster_scale = raster_scale * task.device_pixel_scale;

    // Scale from glyph space to local space.
    float glyph_scale_inv = res.scale / glyph_raster_scale;

    // Glyph raster pixels do not include the impact of the transform. Instead it was
    // replaced with an identity transform during glyph rasterization. As such only the
    // impact of the raster scale (if in local space) and the device pixel scale (for both
    // local and screen space) are included.
    //
    // This implies one or more of the following conditions:
    // - The transform is an identity. In that case, setting WR_FEATURE_GLYPH_TRANSFORM
    //   should have the same output result as not. We just distingush which path to use
    //   based on the transform used during glyph rasterization. (Screen space).
    // - The transform contains an animation. We will imply local raster space in such
    //   cases to avoid constantly rerasterizing the glyphs.
    // - The transform has perspective or does not have a 2d inverse (Screen or local space).
    // - The transform's scale will result in result in very large rasterized glyphs and
    //   we clamped the size. This will imply local raster space.
    vec2 raster_glyph_offset = floor(glyph.offset * glyph_raster_scale + snap_bias) / res.scale;

    // Compute the glyph rect in local space.
    //
    // The transform may be animated, so we don't want to do any snapping here for the
    // text offset to avoid glyphs wiggling. The text offset should have been snapped
    // already for axis aligned transforms excluding any animations during frame building.
    RectWithSize glyph_rect = RectWithSize(glyph_scale_inv * (res.offset + raster_glyph_offset) + text_offset,
                                           glyph_scale_inv * (res.uv_rect.zw - res.uv_rect.xy));

    // Select the corner of the glyph rect that we are processing.
    vec2 local_pos = glyph_rect.p0 + glyph_rect.size * aPosition.xy;
#endif

    VertexInfo vi = write_vertex(
        local_pos,
        ph.local_clip_rect,
        ph.z,
        transform,
        task
    );

#ifdef WR_FEATURE_GLYPH_TRANSFORM
    vec2 f = (glyph_transform * vi.local_pos - glyph_rect.p0) / glyph_rect.size;
    V_UV_CLIP = vec4(f, 1.0 - f);
#else
    vec2 f = (vi.local_pos - glyph_rect.p0) / glyph_rect.size;
#endif

    write_clip(vi.world_pos, clip_area);

    switch (color_mode) {
        case COLOR_MODE_ALPHA:
        case COLOR_MODE_BITMAP:
            V_MASK_SWIZZLE = vec2(0.0, 1.0);
            V_COLOR = text.color;
            break;
        case COLOR_MODE_SUBPX_BG_PASS2:
        case COLOR_MODE_SUBPX_DUAL_SOURCE:
            V_MASK_SWIZZLE = vec2(1.0, 0.0);
            V_COLOR = text.color;
            break;
        case COLOR_MODE_SUBPX_CONST_COLOR:
        case COLOR_MODE_SUBPX_BG_PASS0:
        case COLOR_MODE_COLOR_BITMAP:
            V_MASK_SWIZZLE = vec2(1.0, 0.0);
            V_COLOR = vec4(text.color.a);
            break;
        case COLOR_MODE_SUBPX_BG_PASS1:
            V_MASK_SWIZZLE = vec2(-1.0, 1.0);
            V_COLOR = vec4(text.color.a) * text.bg_color;
            break;
        default:
            V_MASK_SWIZZLE = vec2(0.0);
            V_COLOR = vec4(1.0);
    }

    vec2 texture_size = vec2(textureSize(sColor0, 0));
    vec2 st0 = res.uv_rect.xy / texture_size;
    vec2 st1 = res.uv_rect.zw / texture_size;

    V_UV = mix(st0, st1, f);
    V_LAYER = res.layer;
    V_UV_BOUNDS = (res.uv_rect + vec4(0.5, 0.5, -0.5, -0.5)) / texture_size.xyxy;
}

void text_brush_vs(
    VertexInfo vi,
    int prim_address,
    RectWithSize prim_rect,
    RectWithSize segment_rect,
    ivec4 prim_user_data,
    int specific_resource_address,
    mat4 transform,
    PictureTask pic_task,
    int brush_flags,
    vec4 segment_data
) {
    // This function is empty and unused for now. It has to be defined to build the shader
    // as a brush, but the brush shader currently branches into text_shader_main_vs earlier
    // instead of using the regular brush vertex interface for text.
    // In the future we should strive to further unify text and brushes, and actually make
    // use of this function.
}

#endif // WR_VERTEX_SHADER

#ifdef WR_FRAGMENT_SHADER

Fragment text_brush_fs(void) {
    Fragment frag;

    vec3 tc = vec3(clamp(V_UV, V_UV_BOUNDS.xy, V_UV_BOUNDS.zw), V_LAYER);
    vec4 mask = texture(sColor0, tc);
    mask.rgb = mask.rgb * V_MASK_SWIZZLE.x + mask.aaa * V_MASK_SWIZZLE.y;

    #ifdef WR_FEATURE_GLYPH_TRANSFORM
        mask *= float(all(greaterThanEqual(V_UV_CLIP, vec4(0.0))));
    #endif

    frag.color = V_COLOR * mask;

    #ifdef WR_FEATURE_DUAL_SOURCE_BLENDING
        frag.blend = V_COLOR.a * mask;
    #endif

    return frag;
}

#endif // WR_FRAGMENT_SHADER

// Undef macro names that could be re-defined by other shaders.
#undef V_COLOR
#undef V_MASK_SWIZZLE
#undef V_UV_BOUNDS
#undef V_UV
#undef V_LAYER
#undef V_UV_CLIP
