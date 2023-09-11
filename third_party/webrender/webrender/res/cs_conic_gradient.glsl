/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared,rect,render_task,gpu_cache,gradient

#define PI                  3.141592653589793

varying vec2 v_pos;

flat varying vec2 v_center;
flat varying float v_start_offset;
flat varying float v_offset_scale;
flat varying float v_angle;

#ifdef WR_VERTEX_SHADER

#define EXTEND_MODE_REPEAT 1

// Rectangle in origin+size format
PER_INSTANCE in vec4 aTaskRect;
PER_INSTANCE in vec2 aCenter;
PER_INSTANCE in vec2 aScale;
PER_INSTANCE in float aStartOffset;
PER_INSTANCE in float aEndOffset;
PER_INSTANCE in float aAngle;
PER_INSTANCE in int aExtendMode;
PER_INSTANCE in int aGradientStopsAddress;

void main(void) {
    // Store 1/d where d = end_offset - start_offset
    // If d = 0, we can't get its reciprocal. Instead, just use a zero scale.
    float d = aEndOffset - aStartOffset;
    v_offset_scale = d != 0.0 ? 1.0 / d : 0.0;

    vec2 pos = aTaskRect.xy + aTaskRect.zw * aPosition.xy;
    gl_Position = uTransform * vec4(pos, 0.0, 1.0);

    v_angle = PI / 2.0 - aAngle;
    v_start_offset = aStartOffset * v_offset_scale;

    // v_pos and v_center are in a coordinate space relative to the task rect
    // (so they are independent of the task origin).
    v_center = aCenter * v_offset_scale;
    v_pos = aTaskRect.zw * aPosition.xy * v_offset_scale * aScale;

    v_gradient_repeat = float(aExtendMode == EXTEND_MODE_REPEAT);
    v_gradient_address = aGradientStopsAddress;
}
#endif


#ifdef WR_FRAGMENT_SHADER

void main(void) {
    // Use inverse trig to find the angle offset from the relative position.
    vec2 current_dir = v_pos - v_center;
    float current_angle = atan(current_dir.y, current_dir.x) + v_angle;
    float offset = fract(current_angle / (2.0 * PI)) * v_offset_scale - v_start_offset;

    oFragColor = sample_gradient(offset);
}

#endif
