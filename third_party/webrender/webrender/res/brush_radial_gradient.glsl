/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define VECS_PER_RADIAL_GRADIENT_BRUSH 2
#define VECS_PER_SPECIFIC_BRUSH VECS_PER_RADIAL_GRADIENT_BRUSH

#define WR_BRUSH_VS_FUNCTION radial_gradient_brush_vs
#define WR_BRUSH_FS_FUNCTION radial_gradient_brush_fs

#include shared,prim_shared,brush

#define V_GRADIENT_ADDRESS  flat_varying_highp_int_address_0

#define V_CENTER            flat_varying_vec4_0.xy
#define V_START_RADIUS      flat_varying_vec4_0.z
#define V_END_RADIUS        flat_varying_vec4_0.w

#define V_REPEATED_SIZE     flat_varying_vec4_1.xy
#define V_GRADIENT_REPEAT   flat_varying_vec4_1.z

#define V_POS               varying_vec4_0.zw

#ifdef WR_FEATURE_ALPHA_PASS
#define V_LOCAL_POS         varying_vec4_0.xy
#define V_TILE_REPEAT       flat_varying_vec4_2.xy
#endif

#ifdef WR_VERTEX_SHADER

struct RadialGradient {
    vec4 center_start_end_radius;
    float ratio_xy;
    int extend_mode;
    vec2 stretch_size;
};

RadialGradient fetch_radial_gradient(int address) {
    vec4 data[2] = fetch_from_gpu_cache_2(address);
    return RadialGradient(
        data[0],
        data[1].x,
        int(data[1].y),
        data[1].zw
    );
}

void radial_gradient_brush_vs(
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
    RadialGradient gradient = fetch_radial_gradient(prim_address);

    if ((brush_flags & BRUSH_FLAG_SEGMENT_RELATIVE) != 0) {
        V_POS = (vi.local_pos - segment_rect.p0) / segment_rect.size;
        V_POS = V_POS * (texel_rect.zw - texel_rect.xy) + texel_rect.xy;
        V_POS = V_POS * local_rect.size;
    } else {
        V_POS = vi.local_pos - local_rect.p0;
    }

    V_CENTER = gradient.center_start_end_radius.xy;
    V_START_RADIUS = gradient.center_start_end_radius.z;
    V_END_RADIUS = gradient.center_start_end_radius.w;

    // Transform all coordinates by the y scale so the
    // fragment shader can work with circles
    vec2 tile_repeat = local_rect.size / gradient.stretch_size;
    V_POS.y *= gradient.ratio_xy;
    V_CENTER.y *= gradient.ratio_xy;
    V_REPEATED_SIZE = gradient.stretch_size;
    V_REPEATED_SIZE.y *=  gradient.ratio_xy;

    V_GRADIENT_ADDRESS = prim_user_data.x;

    // Whether to repeat the gradient instead of clamping.
    V_GRADIENT_REPEAT = float(gradient.extend_mode != EXTEND_MODE_CLAMP);

#ifdef WR_FEATURE_ALPHA_PASS
    V_TILE_REPEAT = tile_repeat.xy;
    V_LOCAL_POS = vi.local_pos;
#endif
}
#endif

#ifdef WR_FRAGMENT_SHADER
Fragment radial_gradient_brush_fs() {

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

    vec2 pd = pos - V_CENTER;
    float rd = V_END_RADIUS - V_START_RADIUS;

    // Solve for t in length(t - pd) = V_START_RADIUS + t * rd
    // using a quadratic equation in form of At^2 - 2Bt + C = 0
    float A = -(rd * rd);
    float B = V_START_RADIUS * rd;
    float C = dot(pd, pd) - V_START_RADIUS * V_START_RADIUS;

    float offset;
    if (A == 0.0) {
        // Since A is 0, just solve for -2Bt + C = 0
        if (B == 0.0) {
            discard;
        }
        float t = 0.5 * C / B;
        if (V_START_RADIUS + rd * t >= 0.0) {
            offset = t;
        } else {
            discard;
        }
    } else {
        float discr = B * B - A * C;
        if (discr < 0.0) {
            discard;
        }
        discr = sqrt(discr);
        float t0 = (B + discr) / A;
        float t1 = (B - discr) / A;
        if (V_START_RADIUS + rd * t0 >= 0.0) {
            offset = t0;
        } else if (V_START_RADIUS + rd * t1 >= 0.0) {
            offset = t1;
        } else {
            discard;
        }
    }

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
#undef V_CENTER
#undef V_START_RADIUS
#undef V_END_RADIUS
#undef V_REPEATED_SIZE
#undef V_GRADIENT_REPEAT
#undef V_POS
#undef V_LOCAL_POS
#undef V_TILE_REPEAT
