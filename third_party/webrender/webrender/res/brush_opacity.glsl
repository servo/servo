/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define VECS_PER_SPECIFIC_BRUSH 3
#define WR_FEATURE_TEXTURE_2D

#include shared,prim_shared,brush

// Interpolated UV coordinates to sample.
varying vec2 v_uv;

// Normalized bounds of the source image in the texture, adjusted to avoid
// sampling artifacts.
flat varying vec4 v_uv_sample_bounds;

#if defined(PLATFORM_ANDROID) && !defined(SWGL)
// Work around Adreno 3xx driver bug. See the v_perspective comment in
// brush_image or bug 1630356 for details.
flat varying vec2 v_perspective_vec;
#define v_perspective v_perspective_vec.x
#else
// Flag to allow perspective interpolation of UV.
flat varying float v_perspective;
#endif

flat varying float v_opacity;

#ifdef WR_VERTEX_SHADER
void brush_vs(
    VertexInfo vi,
    int prim_address,
    RectWithSize local_rect,
    RectWithSize segment_rect,
    ivec4 prim_user_data,
    int specific_resource_address,
    mat4 transform,
    PictureTask pic_task,
    int brush_flags,
    vec4 unused
) {
    ImageSource res = fetch_image_source(prim_user_data.x);
    vec2 uv0 = res.uv_rect.p0;
    vec2 uv1 = res.uv_rect.p1;

    vec2 texture_size = vec2(TEX_SIZE(sColor0).xy);
    vec2 f = (vi.local_pos - local_rect.p0) / local_rect.size;
    f = get_image_quad_uv(prim_user_data.x, f);
    vec2 uv = mix(uv0, uv1, f);
    float perspective_interpolate = (brush_flags & BRUSH_FLAG_PERSPECTIVE_INTERPOLATION) != 0 ? 1.0 : 0.0;

    v_uv = uv / texture_size * mix(vi.world_pos.w, 1.0, perspective_interpolate);
    v_perspective = perspective_interpolate;

    v_uv_sample_bounds = vec4(uv0 + vec2(0.5), uv1 - vec2(0.5)) / texture_size.xyxy;

    v_opacity = clamp(float(prim_user_data.y) / 65536.0, 0.0, 1.0);
}
#endif

#ifdef WR_FRAGMENT_SHADER
Fragment brush_fs() {
    float perspective_divisor = mix(gl_FragCoord.w, 1.0, v_perspective);
    vec2 uv = v_uv * perspective_divisor;
    // Clamp the uvs to avoid sampling artifacts.
    uv = clamp(uv, v_uv_sample_bounds.xy, v_uv_sample_bounds.zw);

    // No need to un-premultiply since we'll only apply a factor to the alpha.
    vec4 color = texture(sColor0, uv);

    float alpha = v_opacity;

    #ifdef WR_FEATURE_ALPHA_PASS
        alpha *= antialias_brush();
    #endif

    // Pre-multiply the contribution of the opacity factor.
    return Fragment(alpha * color);
}

#if defined(SWGL_DRAW_SPAN) && !defined(WR_FEATURE_DUAL_SOURCE_BLENDING)
void swgl_drawSpanRGBA8() {
    float perspective_divisor = mix(swgl_forceScalar(gl_FragCoord.w), 1.0, v_perspective);
    vec2 uv = v_uv * perspective_divisor;

    swgl_commitTextureLinearColorRGBA8(sColor0, uv, v_uv_sample_bounds, v_opacity);
}
#endif

#endif
