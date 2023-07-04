/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define VECS_PER_SOLID_BRUSH 1
#define VECS_PER_SPECIFIC_BRUSH VECS_PER_SOLID_BRUSH

#define WR_BRUSH_VS_FUNCTION solid_brush_vs
#define WR_BRUSH_FS_FUNCTION solid_brush_fs

#include shared,prim_shared,brush

#define V_COLOR             flat_varying_vec4_0

#ifdef WR_FEATURE_ALPHA_PASS
#define V_LOCAL_POS         varying_vec4_0.xy
#endif

#ifdef WR_VERTEX_SHADER

struct SolidBrush {
    vec4 color;
};

SolidBrush fetch_solid_primitive(int address) {
    vec4 data = fetch_from_gpu_cache_1(address);
    return SolidBrush(data);
}

void solid_brush_vs(
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
    SolidBrush prim = fetch_solid_primitive(prim_address);

    float opacity = float(prim_user_data.x) / 65535.0;
    V_COLOR = prim.color * opacity;

#ifdef WR_FEATURE_ALPHA_PASS
    V_LOCAL_POS = vi.local_pos;
#endif
}
#endif

#ifdef WR_FRAGMENT_SHADER
Fragment solid_brush_fs() {
    vec4 color = V_COLOR;
#ifdef WR_FEATURE_ALPHA_PASS
    color *= init_transform_fs(V_LOCAL_POS);
#endif
    return Fragment(color);
}
#endif

// Undef macro names that could be re-defined by other shaders.
#undef V_COLOR
#undef V_LOCAL_POS
