/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
    Ellipse equation:

    (x-h)^2     (y-k)^2
    -------  +  -------   <=  1
      rx^2        ry^2

 */

float Value(vec2 position) {
    float outer_rx = vBorderRadii.x;
    float outer_ry = vBorderRadii.y;
    float outer_dx = position.x * position.x / (outer_rx * outer_rx);
    float outer_dy = position.y * position.y / (outer_ry * outer_ry);
    if (outer_dx + outer_dy > 1.0)
        return 0.0;

    float inner_rx = vBorderRadii.z;
    float inner_ry = vBorderRadii.w;
    if (inner_rx == 0.0 || inner_ry == 0.0)
        return 1.0;

    float inner_dx = position.x * position.x / (inner_rx * inner_rx);
    float inner_dy = position.y * position.y / (inner_ry * inner_ry);
    return inner_dx + inner_dy >= 1.0 ? 1.0 : 0.0;
}

void main(void)
{
    vec2 position = vPosition - vBorderPosition.xy;
    vec4 pixelBounds = vec4(floor(position.x), floor(position.y),
                            ceil(position.x), ceil(position.y));
    float value = (Value(pixelBounds.xy) + Value(pixelBounds.zy) +
                   Value(pixelBounds.xw) + Value(pixelBounds.zw)) / 4.0;
    SetFragColor(vec4(vColor.rgb, mix(1.0 - vColor.a, vColor.a, value)));
}

