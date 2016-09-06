#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define DIR_HORIZONTAL      uint(0)
#define DIR_VERTICAL        uint(1)

void main(void) {
    AlignedGradient gradient = fetch_aligned_gradient(gl_InstanceID);

#ifdef WR_FEATURE_TRANSFORM
    TransformVertexInfo vi = write_transform_vertex(gradient.info);
    vLocalRect = vi.clipped_local_rect;
    vLocalPos = vi.local_pos;
    vec2 f = (vi.local_pos.xy - gradient.info.local_rect.xy) / gradient.info.local_rect.zw;
#else
    VertexInfo vi = write_vertex(gradient.info);
    vec2 f = (vi.local_clamped_pos - gradient.info.local_rect.xy) / gradient.info.local_rect.zw;
    vPos = vi.local_clamped_pos;
#endif

    switch (uint(gradient.dir.x)) {
        case DIR_HORIZONTAL:
            vF = f.x;
            break;
        case DIR_VERTICAL:
            vF = f.y;
            break;
    }

    vClipRect = vec4(gradient.clip.rect.xy, gradient.clip.rect.xy + gradient.clip.rect.zw);
    vClipRadius = vec4(gradient.clip.top_left.outer_inner_radius.x,
                       gradient.clip.top_right.outer_inner_radius.x,
                       gradient.clip.bottom_right.outer_inner_radius.x,
                       gradient.clip.bottom_left.outer_inner_radius.x);

    vColor0 = gradient.color0;
    vColor1 = gradient.color1;
}
