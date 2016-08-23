#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct Image {
    PrimitiveInfo info;
    vec4 st_rect;       // Location of the image texture in the texture atlas.
    vec4 stretch_size;  // Size of the actual image.
};

layout(std140) uniform Items {
    Image images[WR_MAX_PRIM_ITEMS];
};

void main(void) {
    Image image = images[gl_InstanceID];

#ifdef WR_FEATURE_TRANSFORM
    TransformVertexInfo vi = write_transform_vertex(image.info);
    vLocalRect = vi.clipped_local_rect;
    vLocalPos = vi.local_pos;
    vStretchSize = image.stretch_size.xy;
#else
    VertexInfo vi = write_vertex(image.info);
    vUv = (vi.local_clamped_pos - vi.local_rect.p0) / image.stretch_size.xy;
#endif

    // vUv will contain how many times this image has wrapped around the image size.
    vTextureSize = image.st_rect.zw - image.st_rect.xy;
    vTextureOffset = image.st_rect.xy;
}
