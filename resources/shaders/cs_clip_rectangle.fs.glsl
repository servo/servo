/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

float rounded_rect(vec2 pos) {
    vec2 ref_tl = vClipRect.xy + vec2( vClipRadius.x,  vClipRadius.x);
    vec2 ref_tr = vClipRect.zy + vec2(-vClipRadius.y,  vClipRadius.y);
    vec2 ref_br = vClipRect.zw + vec2(-vClipRadius.z, -vClipRadius.z);
    vec2 ref_bl = vClipRect.xw + vec2( vClipRadius.w, -vClipRadius.w);

    float d_tl = distance(pos, ref_tl);
    float d_tr = distance(pos, ref_tr);
    float d_br = distance(pos, ref_br);
    float d_bl = distance(pos, ref_bl);

    float pixels_per_fragment = length(fwidth(pos.xy));
    float nudge = 0.5 * pixels_per_fragment;
    vec4 distances = vec4(d_tl, d_tr, d_br, d_bl) - vClipRadius + nudge;

    bvec4 is_out = bvec4(pos.x < ref_tl.x && pos.y < ref_tl.y,
                         pos.x > ref_tr.x && pos.y < ref_tr.y,
                         pos.x > ref_br.x && pos.y > ref_br.y,
                         pos.x < ref_bl.x && pos.y > ref_bl.y);

    float distance_from_border = dot(vec4(is_out),
                                     max(vec4(0.0, 0.0, 0.0, 0.0), distances));

    // Move the distance back into pixels.
    distance_from_border /= pixels_per_fragment;
    // Apply a more gradual fade out to transparent.
    //distance_from_border -= 0.5;

    return 1.0 - smoothstep(0.0, 1.0, distance_from_border);
}


void main(void) {
    float alpha = 1.f;
    vec2 local_pos = init_transform_fs(vPos, vLocalRect, alpha);

    float clip_alpha = rounded_rect(local_pos);

    oFragColor = vec4(1.0, 1.0, 1.0, min(alpha, clip_alpha));
}
