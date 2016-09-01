#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    Image image = fetch_image(gl_InstanceID);

#ifdef WR_FEATURE_TRANSFORM
    TransformVertexInfo vi = write_transform_vertex(image.info);
    vLocalRect = vi.clipped_local_rect;
    vLocalPos = vi.local_pos;
    vStretchSize = image.stretch_size_uvkind.xy;
#else
    VertexInfo vi = write_vertex(image.info);
    vUv = (vi.local_clamped_pos - vi.local_rect.p0) / image.stretch_size_uvkind.xy;
#endif

    // vUv will contain how many times this image has wrapped around the image size.
    vec2 st0 = image.st_rect.xy;
    vec2 st1 = image.st_rect.zw;

    switch (uint(image.stretch_size_uvkind.z)) {
        case UV_NORMALIZED:
            break;
        case UV_PIXEL: {
                vec2 texture_size = textureSize(sDiffuse, 0);
                st0 /= texture_size;
                st1 /= texture_size;
            }
            break;
    }

    vTextureSize = st1 - st0;
    vTextureOffset = st0;
}
