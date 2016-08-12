#line 1

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
#ifdef WR_FEATURE_TRANSFORM
    float alpha = 0;
    vec2 pos = init_transform_fs(vLocalPos, vLocalRect, alpha);
    vec2 uv = (pos - vLocalRect.xy) / vStretchSize;
#else
    vec2 uv = vUv;
#endif
    vec2 st = vTextureOffset + vTextureSize * fract(uv);
#ifdef WR_FEATURE_TRANSFORM
    oFragColor = vec4(1, 1, 1, alpha) * texture(sDiffuse, st);
#else
    oFragColor = texture(sDiffuse, st);
#endif
}
