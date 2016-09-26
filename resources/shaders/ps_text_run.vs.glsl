#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    vec4 color, uv_rect;
    PrimitiveInfo info = fetch_text_run_glyph(gl_InstanceID, color, uv_rect);

#ifdef WR_FEATURE_TRANSFORM
    TransformVertexInfo vi = write_transform_vertex(info);
    vLocalRect = vi.clipped_local_rect;
    vLocalPos = vi.local_pos;
    vec2 f = (vi.local_pos.xy - info.local_rect.xy) / info.local_rect.zw;
#else
    VertexInfo vi = write_vertex(info);
    vec2 f = (vi.local_clamped_pos - vi.local_rect.p0) / (vi.local_rect.p1 - vi.local_rect.p0);
#endif

    vec2 texture_size = vec2(textureSize(sDiffuse, 0));
    vec2 st0 = uv_rect.xy / texture_size;
    vec2 st1 = uv_rect.zw / texture_size;

    vColor = color;
    vUv = mix(st0, st1, f);
}
