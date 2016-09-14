#line 1

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
#ifdef WR_FEATURE_TRANSFORM
    float alpha = 1;
    vec2 local_pos = init_transform_fs(vLocalPos, vLocalRect, alpha);

    // We clamp the texture coordinate calculation here to the local rectangle boundaries,
    // which makes the edge of the texture stretch instead of repeat.
    vec2 uv = clamp(local_pos.xy, vLocalRect.xy, vLocalRect.xy + vLocalRect.zw);

    uv = (uv - vLocalRect.xy) / vStretchSize;
#else
    float alpha = 1;
    vec2 local_pos = vLocalPos;
    vec2 uv = vUv;
#endif

    vec2 st = vTextureOffset + vTextureSize * fract(uv);

    alpha = min(alpha, do_clip(local_pos, vClipRect, vClipRadius));
    oFragColor = texture(sDiffuse, st) * vec4(1, 1, 1, alpha);
}
