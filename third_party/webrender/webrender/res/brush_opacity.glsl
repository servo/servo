/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define VECS_PER_OPACITY_BRUSH 3
#define VECS_PER_SPECIFIC_BRUSH VECS_PER_OPACITY_BRUSH

#define WR_BRUSH_VS_FUNCTION opacity_brush_vs
#define WR_BRUSH_FS_FUNCTION opacity_brush_fs

#include shared,prim_shared,brush

// Interpolated UV coordinates to sample.
#define V_UV                varying_vec4_0.zw
#define V_LOCAL_POS         varying_vec4_0.xy

// Normalized bounds of the source image in the texture.
#define V_UV_BOUNDS         flat_varying_vec4_1

// Layer index to sample.
#define V_LAYER             flat_varying_vec4_2.x
// Flag to allow perspective interpolation of UV.
#define V_PERSPECTIVE       flat_varying_vec4_2.y

#define V_OPACITY           flat_varying_vec4_2.z

#ifdef WR_VERTEX_SHADER
void opacity_brush_vs(
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
    ImageResource res = fetch_image_resource(prim_user_data.x);
    vec2 uv0 = res.uv_rect.p0;
    vec2 uv1 = res.uv_rect.p1;

    vec2 texture_size = vec2(textureSize(sColor0, 0).xy);
    vec2 f = (vi.local_pos - local_rect.p0) / local_rect.size;
    f = get_image_quad_uv(prim_user_data.x, f);
    vec2 uv = mix(uv0, uv1, f);
    float perspective_interpolate = (brush_flags & BRUSH_FLAG_PERSPECTIVE_INTERPOLATION) != 0 ? 1.0 : 0.0;

    V_UV = uv / texture_size * mix(vi.world_pos.w, 1.0, perspective_interpolate);
    V_LAYER = res.layer;
    V_PERSPECTIVE = perspective_interpolate;

    // TODO: The image shader treats this differently: deflate the rect by half a pixel on each side and
    // clamp the uv in the frame shader. Does it make sense to do the same here?
    V_UV_BOUNDS = vec4(uv0, uv1) / texture_size.xyxy;
    V_LOCAL_POS = vi.local_pos;

    V_OPACITY = float(prim_user_data.y) / 65536.0;
}
#endif

#ifdef WR_FRAGMENT_SHADER
Fragment opacity_brush_fs() {
    float perspective_divisor = mix(gl_FragCoord.w, 1.0, V_PERSPECTIVE);
    vec2 uv = V_UV * perspective_divisor;
    vec4 Cs = texture(sColor0, vec3(uv, V_LAYER));

    // Un-premultiply the input.
    float alpha = Cs.a;
    vec3 color = alpha != 0.0 ? Cs.rgb / alpha : Cs.rgb;

    alpha *= V_OPACITY;

    // Fail-safe to ensure that we don't sample outside the rendered
    // portion of a blend source.
    alpha *= min(point_inside_rect(uv, V_UV_BOUNDS.xy, V_UV_BOUNDS.zw),
                 init_transform_fs(V_LOCAL_POS));

    // Pre-multiply the alpha into the output value.
    return Fragment(alpha * vec4(color, 1.0));
}
#endif

// Undef macro names that could be re-defined by other shaders.
#undef V_UV
#undef V_LOCAL_POS
#undef V_UV_BOUNDS
#undef V_LAYER
#undef V_PERSPECTIVE
#undef V_OPACITY
