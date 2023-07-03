/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Initialize the pixel local storage area by reading the current
// framebuffer color. We might be able to skip this in future by
// making the opaque pass also write to pixel local storage.

#define PLS_WRITEONLY

#include shared

#ifdef WR_VERTEX_SHADER
PER_INSTANCE in vec4 aRect;

void main(void) {
    vec2 pos = aRect.xy + aPosition.xy * aRect.zw;
    gl_Position = uTransform * vec4(pos, 0.0, 1.0);
}
#endif

#ifdef WR_FRAGMENT_SHADER
void main(void) {
    // Store current framebuffer color in our custom PLS struct.
	PLS.color = gl_LastFragColorARM;
}
#endif
