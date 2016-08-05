/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

float offset(int index) {
    return vOffsets[index / 4][index % 4];
}

float linearStep(float lo, float hi, float x) {
    float d = hi - lo;
    float v = x - lo;
    if (d != 0.0) {
        v /= d;
    }
    return clamp(v, 0.0, 1.0);
}

void main(void) {
    float angle = atan(-vEndPoint.y + vStartPoint.y,
                        vEndPoint.x - vStartPoint.x);
    float sa = sin(angle);
    float ca = cos(angle);

    float sx = vStartPoint.x * ca - vStartPoint.y * sa;
    float ex = vEndPoint.x * ca - vEndPoint.y * sa;
    float d = ex - sx;

    float x = vPos.x * ca - vPos.y * sa;

    oFragColor = mix(vColors[0],
                     vColors[1],
                     linearStep(sx + d * offset(0), sx + d * offset(1), x));

    for (int i=1 ; i < vStopCount-1 ; ++i) {
        oFragColor = mix(oFragColor,
                         vColors[i+1],
                         linearStep(sx + d * offset(i), sx + d * offset(i+1), x));
    }
}
