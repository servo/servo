#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct Image {
    PrimitiveInfo info;
    vec4 st_rect;       // Location of the image texture in the texture atlas.
    vec4 stretch_size;  // Size of the actual image.
    Clip clip;
};

layout(std140) uniform Items {
    Image images[WR_MAX_PRIM_ITEMS];
};

void main(void) {
    Image image = images[gl_InstanceID];
    VertexInfo vi = write_vertex(image.info);

    vClipRect = vec4(image.clip.rect.xy, image.clip.rect.xy + image.clip.rect.zw);
    vClipRadius = vec4(image.clip.top_left.outer_inner_radius.x,
                       image.clip.top_right.outer_inner_radius.x,
                       image.clip.bottom_right.outer_inner_radius.x,
                       image.clip.bottom_left.outer_inner_radius.x);
    vPos = vi.local_clamped_pos;

    // vUv will contain how many times this image has wrapped around the image size.
    vUv = (vi.local_clamped_pos - image.info.local_rect.xy) / image.stretch_size.xy;
    vTextureSize = image.st_rect.zw - image.st_rect.xy;
    vTextureOffset = image.st_rect.xy;
}
