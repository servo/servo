/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Write the final value stored in pixel local store out to normal
// fragment outputs. This will be the color that gets resolved out
// to main memory.

#define PLS_READONLY

#include shared

#ifdef WR_VERTEX_SHADER
PER_INSTANCE in vec4 aRect;

void main(void) {
    vec2 pos = aRect.xy + aPosition.xy * aRect.zw;
    gl_Position = uTransform * vec4(pos, 0.0, 1.0);
}
#endif

#ifdef WR_FRAGMENT_SHADER
out vec4 oFragColor;

void main(void) {
	// Write the final color value in pixel local storage out as a fragment color.
    oFragColor = PLS.color;
}
#endif
