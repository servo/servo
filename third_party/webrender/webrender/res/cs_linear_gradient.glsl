/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared,rect,render_task,gpu_cache,gradient

varying vec2 v_pos;

flat varying vec2 v_scale_dir;
flat varying float v_start_offset;

#ifdef WR_VERTEX_SHADER

#define EXTEND_MODE_REPEAT 1

// Rectangle in origin+size format
PER_INSTANCE in vec4 aTaskRect;
PER_INSTANCE in vec2 aStartPoint;
PER_INSTANCE in vec2 aEndPoint;
PER_INSTANCE in vec2 aScale;
PER_INSTANCE in int aExtendMode;
PER_INSTANCE in int aGradientStopsAddress;

void main(void) {
    vec2 pos = aTaskRect.xy + aTaskRect.zw * aPosition.xy;
    gl_Position = uTransform * vec4(pos, 0.0, 1.0);

    v_pos = aPosition.xy * aScale;

    vec2 dir = aEndPoint - aStartPoint;

    // Normalize UV and offsets to 0..1 scale.
    v_scale_dir = dir / dot(dir, dir);
    v_start_offset = dot(aStartPoint, v_scale_dir);

    v_scale_dir *= aTaskRect.zw;

    v_gradient_repeat = float(aExtendMode == EXTEND_MODE_REPEAT);
    v_gradient_address = aGradientStopsAddress;
}
#endif


#ifdef WR_FRAGMENT_SHADER

void main(void) {
    // Project position onto a direction vector to compute offset.
    float offset = dot(v_pos, v_scale_dir) - v_start_offset;

    oFragColor = sample_gradient(offset);
}


#ifdef SWGL_DRAW_SPAN
void swgl_drawSpanRGBA8() {
    int address = swgl_validateGradient(sGpuCache, get_gpu_cache_uv(v_gradient_address), int(GRADIENT_ENTRIES + 2.0));
    if (address < 0) {
        return;
    }

    float offset = dot(v_pos, v_scale_dir) - v_start_offset;
    swgl_commitLinearGradientRGBA8(sGpuCache, address, GRADIENT_ENTRIES, v_gradient_repeat != 0.0,
                                   offset);
}
#endif


#endif
