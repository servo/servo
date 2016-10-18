#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    ImageClip image = fetch_image_clip(gl_InstanceID);

#ifdef WR_FEATURE_TRANSFORM
    TransformVertexInfo vi = write_transform_vertex(image.info);
    vLocalPos = vi.local_pos;
#else
    VertexInfo vi = write_vertex(image.info);
    vLocalPos = vi.local_clamped_pos;
    vLocalRect = image.info.local_rect;
#endif

    write_clip(image.clip);

    vec2 st0 = image.st_rect.xy;
    vec2 st1 = image.st_rect.zw;

    switch (uint(image.uvkind.x)) {
        case UV_NORMALIZED:
            break;
        case UV_PIXEL: {
                vec2 texture_size = vec2(textureSize(sDiffuse, 0));
                st0 /= texture_size;
                st1 /= texture_size;
            }
            break;
    }

    vTextureSize = st1 - st0;
    vTextureOffset = st0;
    vStretchSize = image.stretch_size_and_tile_spacing.xy;
    vTileSpacing = image.stretch_size_and_tile_spacing.zw;
}
