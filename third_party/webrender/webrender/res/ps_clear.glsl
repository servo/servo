/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared

varying vec4 vColor;

#ifdef WR_VERTEX_SHADER
PER_INSTANCE in vec4 aRect;
PER_INSTANCE in vec4 aColor;

void main(void) {
    vec2 pos = aRect.xy + aPosition.xy * aRect.zw;
    gl_Position = uTransform * vec4(pos, 0.0, 1.0);
    gl_Position.z = gl_Position.w; // force depth clear to 1.0
    vColor = aColor;
}
#endif

#ifdef WR_FRAGMENT_SHADER
void main(void) {
    oFragColor = vColor;
}
#endif
