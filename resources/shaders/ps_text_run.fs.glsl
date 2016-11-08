/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
#ifdef WR_FEATURE_SUBPIXEL_AA
    oFragColor = texture(sDiffuse, vUv);
#else
    float a = texture(sDiffuse, vUv).a;
#ifdef WR_FEATURE_TRANSFORM
    float alpha = 0.0;
    init_transform_fs(vLocalPos, vLocalRect, alpha);
    a *= alpha;
#endif
    oFragColor = vec4(vColor.rgb, vColor.a * a);
#endif
}
