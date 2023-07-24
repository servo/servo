/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include base

varying vec4 vData;

#ifdef WR_VERTEX_SHADER
in vec4 aValue;
in vec2 aPosition;

void main() {
    vData = aValue;
    gl_Position = vec4(aPosition * 2.0 - 1.0, 0.0, 1.0);
    gl_PointSize = 1.0;
}

#endif //WR_VERTEX_SHADER

#ifdef WR_FRAGMENT_SHADER
out vec4 oValue;

void main() {
    oValue = vData;
}
#endif //WR_FRAGMENT_SHADER
