#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

flat varying vec4 vClipRect;
flat varying vec4 vClipRadius;
flat varying vec4 vClipMaskUvRect;
flat varying vec4 vClipMaskLocalRect;

#ifdef WR_VERTEX_SHADER
void write_clip(ClipData clip) {
    vClipRect = vec4(clip.rect.rect.xy, clip.rect.rect.xy + clip.rect.rect.zw);
    vClipRadius = vec4(clip.top_left.outer_inner_radius.x,
                       clip.top_right.outer_inner_radius.x,
                       clip.bottom_right.outer_inner_radius.x,
                       clip.bottom_left.outer_inner_radius.x);
    //TODO: interpolate the final mask UV
    vec2 texture_size = textureSize(sMask, 0);
    vClipMaskUvRect = clip.mask_data.uv_rect / texture_size.xyxy;
    vClipMaskLocalRect = clip.mask_data.local_rect; //TODO: transform
}
#endif

#ifdef WR_FRAGMENT_SHADER
float do_clip(vec2 pos) {
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

    float border_alpha = 1.0 - smoothstep(0.0, 1.0, distance_from_border);

    bool repeat_mask = false; //TODO
    vec2 vMaskUv = (pos - vClipMaskLocalRect.xy) / vClipMaskLocalRect.zw;
    vec2 clamped_mask_uv = repeat_mask ? fract(vMaskUv) :
        clamp(vMaskUv, vec2(0.0, 0.0), vec2(1.0, 1.0));
    vec2 source_uv = clamped_mask_uv * vClipMaskUvRect.zw + vClipMaskUvRect.xy;
    float mask_alpha = texture(sMask, source_uv).r; //careful: texture has type A8

    return border_alpha * mask_alpha;
}
#endif
