#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct BoxShadow {
	PrimitiveInfo info;
	vec4 color;
    vec4 border_radii_blur_radius_inverted;
    vec4 bs_rect;
    vec4 src_rect;
};

layout(std140) uniform Items {
    BoxShadow boxshadows[WR_MAX_PRIM_ITEMS];
};

void main(void) {
    BoxShadow bs = boxshadows[gl_InstanceID];
    VertexInfo vi = write_vertex(bs.info);

    vPos = vi.local_clamped_pos;
    vColor = bs.color;
    vBorderRadii = bs.border_radii_blur_radius_inverted.xy;
    vBlurRadius = bs.border_radii_blur_radius_inverted.z;
    vBoxShadowRect = vec4(bs.bs_rect.xy, bs.bs_rect.xy + bs.bs_rect.zw);
    vSrcRect = vec4(bs.src_rect.xy, bs.src_rect.xy + bs.src_rect.zw);
    vInverted = bs.border_radii_blur_radius_inverted.w;
}
