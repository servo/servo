/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
#ifdef WR_FEATURE_TRANSFORM
    float alpha = 1;
    vec2 local_pos = init_transform_fs(vPos, vLocalRect, alpha);
#else
    float alpha = 1;
    vec2 local_pos = vPos;
#endif

    alpha = min(alpha, do_clip(local_pos, vClipRect, vClipRadius));
    oFragColor = vColor * vec4(1, 1, 1, alpha);
}
