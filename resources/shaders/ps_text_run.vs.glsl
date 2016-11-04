#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    Primitive prim = load_primitive(gl_InstanceID);
    TextRun text = fetch_text_run(prim.prim_index);
    Glyph glyph = fetch_glyph(prim.sub_index);
    vec4 local_rect = vec4(glyph.offset.xy, (glyph.uv_rect.zw - glyph.uv_rect.xy) / uDevicePixelRatio);

#ifdef WR_FEATURE_TRANSFORM
    TransformVertexInfo vi = write_transform_vertex(local_rect,
                                                    prim.local_clip_rect,
                                                    prim.layer,
                                                    prim.tile);
    vLocalRect = vi.clipped_local_rect;
    vLocalPos = vi.local_pos;
    vec2 f = (vi.local_pos.xy / vi.local_pos.z - local_rect.xy) / local_rect.zw;
#else
    VertexInfo vi = write_vertex(local_rect,
                                 prim.local_clip_rect,
                                 prim.layer,
                                 prim.tile);
    vec2 f = (vi.local_clamped_pos - vi.local_rect.p0) / (vi.local_rect.p1 - vi.local_rect.p0);
#endif

    vec2 texture_size = vec2(textureSize(sDiffuse, 0));
    vec2 st0 = glyph.uv_rect.xy / texture_size;
    vec2 st1 = glyph.uv_rect.zw / texture_size;

    vColor = text.color;
    vUv = mix(st0, st1, f);
}
