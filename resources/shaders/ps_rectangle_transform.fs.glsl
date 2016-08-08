/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
void main(void) {
    vec2 pos = vPos.xy / vPos.z;

    float squared_distance = squared_distance_from_rect(pos, vRect.xy, vRect.zw);
    if (squared_distance != 0) {
        float delta = length(fwidth(vPos.xy));
        float alpha = smoothstep(1.0, 0.0, squared_distance / delta * 2);
        oFragColor = vColor * vec4(1.0, 1.0, 1.0, alpha);
    } else {
        oFragColor = vColor;
    }
}
