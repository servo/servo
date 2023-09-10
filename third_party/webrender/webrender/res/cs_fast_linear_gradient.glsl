/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared

varying float vPos;
flat varying vec4 vColor0;
flat varying vec4 vColor1;

#ifdef WR_VERTEX_SHADER

PER_INSTANCE in vec4 aTaskRect;
PER_INSTANCE in vec4 aColor0;
PER_INSTANCE in vec4 aColor1;
PER_INSTANCE in float aAxisSelect;

void main(void) {
    vPos = mix(0.0, 1.0, mix(aPosition.x, aPosition.y, aAxisSelect));

    vColor0 = aColor0;
    vColor1 = aColor1;

    gl_Position = uTransform * vec4(aTaskRect.xy + aTaskRect.zw * aPosition.xy, 0.0, 1.0);
}
#endif

#ifdef WR_FRAGMENT_SHADER
void main(void) {
    oFragColor = mix(vColor0, vColor1, vPos);
}
#endif
