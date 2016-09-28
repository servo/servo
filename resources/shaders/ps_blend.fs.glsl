/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

uniform sampler2D sCache;

void main(void) {
    vec4 color = texture(sCache, vUv);
    oFragColor = vec4(color.rgb * vBrightnessOpacity.x, color.a * vBrightnessOpacity.y);
}
