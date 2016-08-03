#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct Glyph {
    PrimitiveInfo info;
    vec4 color;
    vec4 st_rect;
};

layout(std140) uniform Items {
    Glyph glyphs[WR_MAX_PRIM_ITEMS];
};

void main(void) {
    Glyph glyph = glyphs[gl_InstanceID];
    VertexInfo vi = write_vertex(glyph.info);

    vec2 f = (vi.local_clamped_pos - vi.local_rect.p0) / (vi.local_rect.p1 - vi.local_rect.p0);

    vColor = glyph.color;
    vUv = mix(glyph.st_rect.xy,
              glyph.st_rect.zw,
              f);
}
