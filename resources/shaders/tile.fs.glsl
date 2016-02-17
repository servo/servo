/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    vec2 textureSize = vBorderPosition.zw - vBorderPosition.xy;
    vec2 colorTexCoord = vBorderPosition.xy + mod(vColorTexCoord.xy, 1.0) * textureSize;
    vec4 diffuse = Texture(sDiffuse, colorTexCoord);
    SetFragColor(diffuse);
}

