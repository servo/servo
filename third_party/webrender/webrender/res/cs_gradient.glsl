/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared

varying float vPos;
flat varying vec4 vStops;
flat varying vec4 vColor0;
flat varying vec4 vColor1;
flat varying vec4 vColor2;
flat varying vec4 vColor3;

#ifdef WR_VERTEX_SHADER

PER_INSTANCE in vec4 aTaskRect;
PER_INSTANCE in float aAxisSelect;
PER_INSTANCE in vec4 aStops;
PER_INSTANCE in vec4 aColor0;
PER_INSTANCE in vec4 aColor1;
PER_INSTANCE in vec4 aColor2;
PER_INSTANCE in vec4 aColor3;
PER_INSTANCE in vec2 aStartStop;

void main(void) {
    vPos = mix(aStartStop.x, aStartStop.y, mix(aPosition.x, aPosition.y, aAxisSelect));

    vStops = aStops;
    vColor0 = aColor0;
    vColor1 = aColor1;
    vColor2 = aColor2;
    vColor3 = aColor3;

    gl_Position = uTransform * vec4(aTaskRect.xy + aTaskRect.zw * aPosition.xy, 0.0, 1.0);
}
#endif

#ifdef WR_FRAGMENT_SHADER
float linear_step(float edge0, float edge1, float x) {
    if (edge0 >= edge1) {
        return 0.0;
    }

    return clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
}

void main(void) {
    vec4 color = vColor0;

    color = mix(color, vColor1, linear_step(vStops.x, vStops.y, vPos));
    color = mix(color, vColor2, linear_step(vStops.y, vStops.z, vPos));
    color = mix(color, vColor3, linear_step(vStops.z, vStops.w, vPos));

    oFragColor = color;
}
#endif
