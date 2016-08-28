#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct TextRunGlyph {
    vec4 local_rect;
    ivec4 uv_rect;
};

struct TextRun {
    PrimitiveInfo info;
    TextRunGlyph glyphs[WR_GLYPHS_PER_TEXT_RUN];
    vec4 color;
};

layout(std140) uniform Items {
    TextRun text_runs[WR_MAX_PRIM_ITEMS];
};

void main(void) {
    TextRun text_run = text_runs[gl_InstanceID / WR_GLYPHS_PER_TEXT_RUN];
    TextRunGlyph glyph = text_run.glyphs[gl_InstanceID % WR_GLYPHS_PER_TEXT_RUN];
    text_run.info.local_rect = glyph.local_rect;
    ivec4 uv_rect = glyph.uv_rect;

#ifdef WR_FEATURE_TRANSFORM
    TransformVertexInfo vi = write_transform_vertex(text_run.info);
    vLocalRect = vi.clipped_local_rect;
    vLocalPos = vi.local_pos;
    vec2 f = (vi.local_pos.xy - text_run.info.local_rect.xy) / text_run.info.local_rect.zw;
#else
    VertexInfo vi = write_vertex(text_run.info);
    vec2 f = (vi.local_clamped_pos - vi.local_rect.p0) / (vi.local_rect.p1 - vi.local_rect.p0);
#endif

    vec2 texture_size = textureSize(sDiffuse, 0);
    vec2 st0 = uv_rect.xy / texture_size;
    vec2 st1 = uv_rect.zw / texture_size;

    vColor = text_run.color;
    vUv = mix(st0, st1, f);
}
