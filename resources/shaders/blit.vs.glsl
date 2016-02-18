/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void)
{
    vColor = aColorRectTL / 255.0;
    vColorTexCoord = aColorTexCoordRectTop.xy;
    vec4 pos = vec4(aPosition, 1.0);
    pos.xy = SnapToPixels(pos.xy);
    gl_Position = uTransform * pos;
}
