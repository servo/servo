/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define VECS_PER_SPECIFIC_BRUSH 1

#include shared,prim_shared,brush

flat varying vec4 v_color;

#ifdef WR_VERTEX_SHADER

struct SolidBrush {
    vec4 color;
};

SolidBrush fetch_solid_primitive(int address) {
    vec4 data = fetch_from_gpu_cache_1(address);
    return SolidBrush(data);
}

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
    SolidBrush prim = fetch_solid_primitive(prim_address);

    float opacity = float(prim_user_data.x) / 65535.0;
    v_color = prim.color * opacity;
}
#endif

#ifdef WR_FRAGMENT_SHADER
Fragment brush_fs() {
    vec4 color = v_color;
#ifdef WR_FEATURE_ALPHA_PASS
    color *= antialias_brush();
#endif
    return Fragment(color);
}

#if defined(SWGL_DRAW_SPAN) && (!defined(WR_FEATURE_ALPHA_PASS) || !defined(WR_FEATURE_DUAL_SOURCE_BLENDING))
void swgl_drawSpanRGBA8() {
    swgl_commitSolidRGBA8(v_color);
}

void swgl_drawSpanR8() {
    swgl_commitSolidR8(v_color.x);
}
#endif

#endif
