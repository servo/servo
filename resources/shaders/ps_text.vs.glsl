#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct Glyph {
    PrimitiveInfo info;
    vec4 color;
    ivec4 uv_rect;
};

layout(std140) uniform Items {
    Glyph glyphs[WR_MAX_PRIM_ITEMS];
};

void main(void) {
    Glyph glyph = glyphs[gl_InstanceID];
    VertexInfo vi = write_vertex(glyph.info);

    vec2 f = (vi.local_clamped_pos - vi.local_rect.p0) / (vi.local_rect.p1 - vi.local_rect.p0);

    vec2 texture_size = textureSize(sDiffuse, 0);
    vec2 st0 = glyph.uv_rect.xy / texture_size;
    vec2 st1 = glyph.uv_rect.zw / texture_size;

    vColor = glyph.color;
    vUv = mix(st0, st1, f);
}
