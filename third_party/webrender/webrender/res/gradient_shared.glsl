/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include gradient

// Size of the gradient pattern's rectangle, used to compute horizontal and vertical
// repetitions. Not to be confused with another kind of repetition of the pattern
// which happens along the gradient stops.
flat varying vec2 v_repeated_size;

varying vec2 v_pos;

#ifdef WR_FEATURE_ALPHA_PASS
flat varying vec2 v_tile_repeat;
#endif

#ifdef WR_VERTEX_SHADER
void write_gradient_vertex(
    VertexInfo vi,
    RectWithSize local_rect,
    RectWithSize segment_rect,
    ivec4 prim_user_data,
    int brush_flags,
    vec4 texel_rect,
    int extend_mode,
    vec2 stretch_size
) {
    if ((brush_flags & BRUSH_FLAG_SEGMENT_RELATIVE) != 0) {
        v_pos = (vi.local_pos - segment_rect.p0) / segment_rect.size;
        v_pos = v_pos * (texel_rect.zw - texel_rect.xy) + texel_rect.xy;
        v_pos = v_pos * local_rect.size;
    } else {
        v_pos = vi.local_pos - local_rect.p0;
    }

    vec2 tile_repeat = local_rect.size / stretch_size;
    v_repeated_size = stretch_size;

    // Normalize UV to 0..1 scale.
    v_pos /= v_repeated_size;

    v_gradient_address = prim_user_data.x;

    // Whether to repeat the gradient along the line instead of clamping.
    v_gradient_repeat = float(extend_mode == EXTEND_MODE_REPEAT);

#ifdef WR_FEATURE_ALPHA_PASS
    v_tile_repeat = tile_repeat;
#endif
}
#endif //WR_VERTEX_SHADER

#ifdef WR_FRAGMENT_SHADER
vec2 compute_repeated_pos() {
#if defined(WR_FEATURE_ALPHA_PASS) && !defined(SWGL_ANTIALIAS)
    // Handle top and left inflated edges (see brush_image).
    vec2 local_pos = max(v_pos, vec2(0.0));

    // Apply potential horizontal and vertical repetitions.
    vec2 pos = fract(local_pos);

    // Handle bottom and right inflated edges (see brush_image).
    if (local_pos.x >= v_tile_repeat.x) {
        pos.x = 1.0;
    }
    if (local_pos.y >= v_tile_repeat.y) {
        pos.y = 1.0;
    }
    return pos;
#else
    // Apply potential horizontal and vertical repetitions.
    return fract(v_pos);
#endif
}

#endif //WR_FRAGMENT_SHADER

