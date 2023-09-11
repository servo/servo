/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifdef WR_FEATURE_TEXTURE_EXTERNAL
// Please check https://www.khronos.org/registry/OpenGL/extensions/OES/OES_EGL_image_external_essl3.txt
// for this extension.
#extension GL_OES_EGL_image_external_essl3 : require
#endif

#ifdef WR_FEATURE_TEXTURE_EXTERNAL_ESSL1
// Some GLES 3 devices do not support GL_OES_EGL_image_external_essl3, so we
// must use GL_OES_EGL_image_external instead and make the shader ESSL1
// compatible.
#extension GL_OES_EGL_image_external : require
#endif

#ifdef WR_FEATURE_ADVANCED_BLEND
#extension GL_KHR_blend_equation_advanced : require
#endif

#ifdef WR_FEATURE_DUAL_SOURCE_BLENDING
#ifdef GL_ES
#extension GL_EXT_blend_func_extended : require
#else
#extension GL_ARB_explicit_attrib_location : require
#endif
#endif

#include base

#if defined(WR_FEATURE_TEXTURE_EXTERNAL_ESSL1)
#define TEX_SAMPLE(sampler, tex_coord) texture2D(sampler, tex_coord.xy)
#else
#define TEX_SAMPLE(sampler, tex_coord) texture(sampler, tex_coord.xy)
#endif

#if defined(WR_FEATURE_TEXTURE_EXTERNAL) && defined(PLATFORM_ANDROID)
// On some Mali GPUs we have encountered crashes in glDrawElements when using
// textureSize(samplerExternalOES) in a vertex shader without potentially
// sampling from the texture. This tricks the driver in to thinking the texture
// may be sampled from, avoiding the crash. See bug 1692848.
uniform bool u_mali_workaround_dummy;
#define TEX_SIZE(sampler) (u_mali_workaround_dummy ? ivec2(texture(sampler, vec2(0.0, 0.0)).rr) : textureSize(sampler, 0))
#else
#define TEX_SIZE(sampler) textureSize(sampler, 0)
#endif

//======================================================================================
// Vertex shader attributes and uniforms
//======================================================================================
#ifdef WR_VERTEX_SHADER
    // A generic uniform that shaders can optionally use to configure
    // an operation mode for this batch.
    uniform int uMode;

    // Uniform inputs
    uniform mat4 uTransform;       // Orthographic projection

    // Attribute inputs
    attribute vec2 aPosition;

    // get_fetch_uv is a macro to work around a macOS Intel driver parsing bug.
    // TODO: convert back to a function once the driver issues are resolved, if ever.
    // https://github.com/servo/webrender/pull/623
    // https://github.com/servo/servo/issues/13953
    // Do the division with unsigned ints because that's more efficient with D3D
    #define get_fetch_uv(i, vpi)  ivec2(int(vpi * (uint(i) % (WR_MAX_VERTEX_TEXTURE_WIDTH/vpi))), int(uint(i) / (WR_MAX_VERTEX_TEXTURE_WIDTH/vpi)))
#endif

//======================================================================================
// Fragment shader attributes and uniforms
//======================================================================================
#ifdef WR_FRAGMENT_SHADER
    // Uniform inputs

    // Fragment shader outputs
    #ifdef WR_FEATURE_ADVANCED_BLEND
        layout(blend_support_all_equations) out;
    #endif

    #if __VERSION__ == 100
        #define oFragColor gl_FragColor
    #elif defined(WR_FEATURE_DUAL_SOURCE_BLENDING)
        layout(location = 0, index = 0) out vec4 oFragColor;
        layout(location = 0, index = 1) out vec4 oFragBlend;
    #else
        out vec4 oFragColor;
    #endif

    // Write an output color in normal shaders.
    void write_output(vec4 color) {
        oFragColor = color;
    }

    #define EPSILON                     0.0001

    // "Show Overdraw" color. Premultiplied.
    #define WR_DEBUG_OVERDRAW_COLOR     vec4(0.110, 0.077, 0.027, 0.125)

    float distance_to_line(vec2 p0, vec2 perp_dir, vec2 p) {
        vec2 dir_to_p0 = p0 - p;
        return dot(normalize(perp_dir), dir_to_p0);
    }

// fwidth is not defined in ESSL 1, but that's okay because we don't need
// it for any ESSL 1 shader variants.
#if __VERSION__ != 100
    /// Find the appropriate half range to apply the AA approximation over.
    /// This range represents a coefficient to go from one CSS pixel to half a device pixel.
    float compute_aa_range(vec2 position) {
        // The constant factor is chosen to compensate for the fact that length(fw) is equal
        // to sqrt(2) times the device pixel ratio in the typical case.
        //
        // This coefficient is chosen to ensure that any sample 0.5 pixels or more inside of
        // the shape has no anti-aliasing applied to it (since pixels are sampled at their center,
        // such a pixel (axis aligned) is fully inside the border). We need this so that antialiased
        // curves properly connect with non-antialiased vertical or horizontal lines, among other things.
        //
        // Lines over a half-pixel away from the pixel center *can* intersect with the pixel square;
        // indeed, unless they are horizontal or vertical, they are guaranteed to. However, choosing
        // a nonzero area for such pixels causes noticeable artifacts at the junction between an anti-
        // aliased corner and a straight edge.
        //
        // We may want to adjust this constant in specific scenarios (for example keep the principled
        // value for straight edges where we want pixel-perfect equivalence with non antialiased lines
        // when axis aligned, while selecting a larger and smoother aa range on curves).
        //
        // As a further optimization, we compute the reciprocal of this range, such that we
        // can then use the cheaper inversesqrt() instead of length(). This also elides a
        // division that would otherwise be necessary inside distance_aa.
        #ifdef SWGL
            // SWGL uses an approximation for fwidth() such that it returns equal x and y.
            // Thus, sqrt(2)/length(w) = sqrt(2)/sqrt(x*x + x*x) = recip(x).
            return recip(fwidth(position).x);
        #else
            // sqrt(2)/length(w) = inversesqrt(0.5 * dot(w, w))
            vec2 w = fwidth(position);
            return inversesqrt(0.5 * dot(w, w));
        #endif
    }
#endif

    /// Return the blending coefficient for distance antialiasing.
    ///
    /// 0.0 means inside the shape, 1.0 means outside.
    ///
    /// This makes the simplifying assumption that the area of a 1x1 pixel square
    /// under a line is reasonably similar to just the signed Euclidian distance
    /// from the center of the square to that line. This diverges slightly from
    /// better approximations of the exact area, but the difference between the
    /// methods is not perceptibly noticeable, while this approximation is much
    /// faster to compute.
    ///
    /// See the comments in `compute_aa_range()` for more information on the
    /// cutoff values of -0.5 and 0.5.
    float distance_aa(float aa_range, float signed_distance) {
        float dist = signed_distance * aa_range;
        return clamp(0.5 - dist, 0.0, 1.0);
    }

    /// Component-wise selection.
    ///
    /// The idea of using this is to ensure both potential branches are executed before
    /// selecting the result, to avoid observable timing differences based on the condition.
    ///
    /// Example usage: color = if_then_else(LessThanEqual(color, vec3(0.5)), vec3(0.0), vec3(1.0));
    ///
    /// The above example sets each component to 0.0 or 1.0 independently depending on whether
    /// their values are below or above 0.5.
    ///
    /// This is written as a macro in order to work with vectors of any dimension.
    ///
    /// Note: Some older android devices don't support mix with bvec. If we ever run into them
    /// the only option we have is to polyfill it with a branch per component.
    #define if_then_else(cond, then_branch, else_branch) mix(else_branch, then_branch, cond)
#endif

//======================================================================================
// Shared shader uniforms
//======================================================================================
#ifdef WR_FEATURE_TEXTURE_2D
uniform sampler2D sColor0;
uniform sampler2D sColor1;
uniform sampler2D sColor2;
#elif defined WR_FEATURE_TEXTURE_RECT
uniform sampler2DRect sColor0;
uniform sampler2DRect sColor1;
uniform sampler2DRect sColor2;
#elif defined(WR_FEATURE_TEXTURE_EXTERNAL) || defined(WR_FEATURE_TEXTURE_EXTERNAL_ESSL1)
uniform samplerExternalOES sColor0;
uniform samplerExternalOES sColor1;
uniform samplerExternalOES sColor2;
#endif

#ifdef WR_FEATURE_DITHERING
uniform sampler2D sDither;
#endif

//======================================================================================
// Interpolator definitions
//======================================================================================

//======================================================================================
// VS only types and UBOs
//======================================================================================

//======================================================================================
// VS only functions
//======================================================================================
