#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct Rectangle {
    PrimitiveInfo info;
    vec4 color;
    Clip clip;
};

layout(std140) uniform Items {
    Rectangle rects[WR_MAX_PRIM_ITEMS];
};

void main(void) {
    Rectangle rect = rects[gl_InstanceID];
    VertexInfo vi = write_vertex(rect.info);

    vClipRect = vec4(rect.clip.rect.xy, rect.clip.rect.xy + rect.clip.rect.zw);
    vClipRadius = vec4(rect.clip.top_left.outer_inner_radius.x,
                       rect.clip.top_right.outer_inner_radius.x,
                       rect.clip.bottom_right.outer_inner_radius.x,
                       rect.clip.bottom_left.outer_inner_radius.x);
    vPos = vi.local_clamped_pos;

    vColor = rect.color;
}
