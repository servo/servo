#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    Primitive prim = load_primitive(gl_InstanceID);
    BoxShadow bs = fetch_boxshadow(prim.prim_index);
    vec4 segment_rect = fetch_instance_geometry(prim.user_data.x + prim.user_data.y);

    VertexInfo vi = write_vertex(segment_rect,
                                 prim.local_clip_rect,
                                 prim.layer,
                                 prim.tile);

    vPos = vi.local_clamped_pos;
    vColor = bs.color;
    vBorderRadii = bs.border_radii_blur_radius_inverted.xy;
    vBlurRadius = bs.border_radii_blur_radius_inverted.z;
    vBoxShadowRect = vec4(bs.bs_rect.xy, bs.bs_rect.xy + bs.bs_rect.zw);
    vSrcRect = vec4(bs.src_rect.xy, bs.src_rect.xy + bs.src_rect.zw);
    vInverted = bs.border_radii_blur_radius_inverted.w;
}
