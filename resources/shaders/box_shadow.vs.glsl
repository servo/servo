/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void)
{
    vPosition = aPosition.xy;
    vColor = aColorRectTL;
    vBorderPosition = aBorderPosition;
    vBorderRadii = aBorderRadii;
    vBlurRadius = aBlurRadius;
    gl_Position = uTransform * vec4(aPosition, 1.0);
}

