/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define VECS_PER_CONIC_GRADIENT_BRUSH 2
#define VECS_PER_SPECIFIC_BRUSH VECS_PER_CONIC_GRADIENT_BRUSH

#define WR_BRUSH_VS_FUNCTION conic_gradient_brush_vs
#define WR_BRUSH_FS_FUNCTION conic_gradient_brush_fs

#include shared,prim_shared,brush

#define V_GRADIENT_ADDRESS  flat_varying_highp_int_address_0

#define V_CENTER            flat_varying_vec4_0.xy
#define V_START_OFFSET      flat_varying_vec4_0.z
#define V_END_OFFSET        flat_varying_vec4_0.w
#define V_ANGLE             flat_varying_vec4_1.w

// Size of the gradient pattern's rectangle, used to compute horizontal and vertical
// repetitions. Not to be confused with another kind of repetition of the pattern
// which happens along the gradient stops.
#define V_REPEATED_SIZE     flat_varying_vec4_1.xy
// Repetition along the gradient stops.
#define V_GRADIENT_REPEAT   flat_varying_vec4_1.z

#define V_POS               varying_vec4_0.zw

#ifdef WR_FEATURE_ALPHA_PASS
#define V_LOCAL_POS         varying_vec4_0.xy
#define V_TILE_REPEAT       flat_varying_vec4_2.xy
#endif

#define PI                  3.141592653589793

#ifdef WR_VERTEX_SHADER

struct ConicGradient {
    vec2 center_point;
    vec2 start_end_offset;
    float angle;
    int extend_mode;
    vec2 stretch_size;
};

ConicGradient fetch_gradient(int address) {
    vec4 data[2] = fetch_from_gpu_cache_2(address);
    return ConicGradient(
        data[0].xy,
        data[0].zw,
        float(data[1].x),
        int(data[1].y),
        data[1].zw
    );
}

void conic_gradient_brush_vs(
    VertexInfo vi,
    int prim_address,
    RectWithSize local_rect,
    RectWithSize segment_rect,
    ivec4 prim_user_data,
    int specific_resource_address,
    mat4 transform,
    PictureTask pic_task,
    int brush_flags,
    vec4 texel_rect
) {
    ConicGradient gradient = fetch_gradient(prim_address);

    if ((brush_flags & BRUSH_FLAG_SEGMENT_RELATIVE) != 0) {
        V_POS = (vi.local_pos - segment_rect.p0) / segment_rect.size;
        V_POS = V_POS * (texel_rect.zw - texel_rect.xy) + texel_rect.xy;
        V_POS = V_POS * local_rect.size;
    } else {
        V_POS = vi.local_pos - local_rect.p0;
    }

    V_CENTER = gradient.center_point;
    V_ANGLE = gradient.angle;
    V_START_OFFSET = gradient.start_end_offset.x;
    V_END_OFFSET = gradient.start_end_offset.y;

    vec2 tile_repeat = local_rect.size / gradient.stretch_size;
    V_REPEATED_SIZE = gradient.stretch_size;

    V_GRADIENT_ADDRESS = prim_user_data.x;

    // Whether to repeat the gradient along the line instead of clamping.
    V_GRADIENT_REPEAT = float(gradient.extend_mode != EXTEND_MODE_CLAMP);

#ifdef WR_FEATURE_ALPHA_PASS
    V_TILE_REPEAT = tile_repeat;
    V_LOCAL_POS = vi.local_pos;
#endif
}
#endif

#ifdef WR_FRAGMENT_SHADER
Fragment conic_gradient_brush_fs() {

#ifdef WR_FEATURE_ALPHA_PASS
    // Handle top and left inflated edges (see brush_image).
    vec2 local_pos = max(V_POS, vec2(0.0));

    // Apply potential horizontal and vertical repetitions.
    vec2 pos = mod(local_pos, V_REPEATED_SIZE);

    vec2 prim_size = V_REPEATED_SIZE * V_TILE_REPEAT;
    // Handle bottom and right inflated edges (see brush_image).
    if (local_pos.x >= prim_size.x) {
        pos.x = V_REPEATED_SIZE.x;
    }
    if (local_pos.y >= prim_size.y) {
        pos.y = V_REPEATED_SIZE.y;
    }
#else
    // Apply potential horizontal and vertical repetitions.
    vec2 pos = mod(V_POS, V_REPEATED_SIZE);
#endif

    vec2 current_dir = pos - V_CENTER;
    float current_angle = atan(current_dir.y, current_dir.x) + (PI / 2.0 - V_ANGLE);
    float offset = mod(current_angle / (2.0 * PI), 1.0) - V_START_OFFSET;
    offset = offset / (V_END_OFFSET - V_START_OFFSET);

    vec4 color = sample_gradient(V_GRADIENT_ADDRESS,
                                 offset,
                                 V_GRADIENT_REPEAT);

#ifdef WR_FEATURE_ALPHA_PASS
    color *= init_transform_fs(V_LOCAL_POS);
#endif

    return Fragment(color);
}
#endif

// Undef macro names that could be re-defined by other shaders.
#undef V_GRADIENT_ADDRESS
#undef V_START_POINT
#undef V_SCALE_DIR
#undef V_REPEATED_SIZE
#undef V_GRADIENT_REPEAT
#undef V_POS
#undef V_LOCAL_POS
#undef V_TILE_REPEAT
