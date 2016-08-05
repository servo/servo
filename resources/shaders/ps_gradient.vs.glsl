#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define DIR_HORIZONTAL      uint(0)
#define DIR_VERTICAL        uint(1)

struct Gradient {
    PrimitiveInfo info;
    vec4 color0;
    vec4 color1;
    uvec4 dir;
    Clip clip;
};

layout(std140) uniform Items {
    Gradient gradients[WR_MAX_PRIM_ITEMS];
};

void main(void) {
    Gradient gradient = gradients[gl_InstanceID];
    VertexInfo vi = write_vertex(gradient.info);

    vec2 f = (vi.local_clamped_pos - gradient.info.local_rect.xy) / gradient.info.local_rect.zw;

    switch (gradient.dir.x) {
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
    vPos = vi.local_clamped_pos;

    vColor0 = gradient.color0;
    vColor1 = gradient.color1;
}
