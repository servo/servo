#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    Primitive prim = load_primitive(gl_InstanceID);
    Image image = fetch_image(prim.prim_index);

#ifdef WR_FEATURE_TRANSFORM
    TransformVertexInfo vi = write_transform_vertex(prim.local_rect,
                                                    prim.local_clip_rect,
                                                    prim.layer,
                                                    prim.tile);
    vLocalRect = vi.clipped_local_rect;
    vLocalPos = vi.local_pos;
#else
    VertexInfo vi = write_vertex(prim.local_rect,
                                 prim.local_clip_rect,
                                 prim.layer,
                                 prim.tile);
    vLocalPos = vi.local_clamped_pos - vi.local_rect.p0;
#endif

    // vUv will contain how many times this image has wrapped around the image size.
    vec2 texture_size = vec2(textureSize(sDiffuse, 0));
    vec2 st0 = image.st_rect.xy / texture_size;
    vec2 st1 = image.st_rect.zw / texture_size;

    vTextureSize = st1 - st0;
    vTextureOffset = st0;
    vTileSpacing = image.stretch_size_and_tile_spacing.zw;
    vStretchSize = image.stretch_size_and_tile_spacing.xy;
}
