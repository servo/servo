#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct Image {
    PrimitiveInfo info;
    vec4 st_rect;
    vec4 stretch_size;  // Size of the actual image.
};

layout(std140) uniform Items {
    Image images[WR_MAX_PRIM_ITEMS];
};

void main(void) {
    Image image = images[gl_InstanceID];

    TransformVertexInfo vi = write_transform_vertex(image.info);

    vLocalRect = image.info.local_rect;
    vLocalPos = vi.local_pos;

    vec2 f = (vi.local_pos.xy - image.info.local_rect.xy) / image.info.local_rect.zw;

    vUv = mix(image.st_rect.xy,
              image.st_rect.zw,
              f);
}
