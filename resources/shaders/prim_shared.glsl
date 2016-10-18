#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define PST_TOP_LEFT     uint(0)
#define PST_TOP_RIGHT    uint(1)
#define PST_BOTTOM_LEFT  uint(2)
#define PST_BOTTOM_RIGHT uint(3)
#define PST_TOP          uint(4)
#define PST_LEFT         uint(5)
#define PST_BOTTOM       uint(6)
#define PST_RIGHT        uint(7)

#define UV_NORMALIZED    uint(0)
#define UV_PIXEL         uint(1)

// Border styles as defined in webrender_traits/types.rs
#define BORDER_STYLE_NONE         uint(0)
#define BORDER_STYLE_SOLID        uint(1)
#define BORDER_STYLE_DOUBLE       uint(2)
#define BORDER_STYLE_DOTTED       uint(3)
#define BORDER_STYLE_DASHED       uint(4)
#define BORDER_STYLE_HIDDEN       uint(5)
#define BORDER_STYLE_GROOVE       uint(6)
#define BORDER_STYLE_RIDGE        uint(7)
#define BORDER_STYLE_INSET        uint(8)
#define BORDER_STYLE_OUTSET       uint(9)

#define MAX_STOPS_PER_ANGLE_GRADIENT 8

#ifdef WR_VERTEX_SHADER

#define VECS_PER_LAYER      13
#define LAYERS_PER_ROW      (WR_MAX_VERTEX_TEXTURE_WIDTH / VECS_PER_LAYER)

#define VECS_PER_TILE       2
#define TILES_PER_ROW       (WR_MAX_VERTEX_TEXTURE_WIDTH / VECS_PER_TILE)

uniform sampler2D sLayers;
uniform sampler2D sRenderTasks;

struct Layer {
    mat4 transform;
    mat4 inv_transform;
    vec4 local_clip_rect;
    vec4 screen_vertices[4];
};

layout(std140) uniform Data {
    vec4 data[WR_MAX_UBO_VECTORS];
};

Layer fetch_layer(int index) {
    Layer layer;

    // Create a UV base coord for each 8 texels.
    // This is required because trying to use an offset
    // of more than 8 texels doesn't work on some versions
    // of OSX.
    int y = index / LAYERS_PER_ROW;
    int x = VECS_PER_LAYER * (index % LAYERS_PER_ROW);

    ivec2 uv0 = ivec2(x + 0, y);
    ivec2 uv1 = ivec2(x + 8, y);

    layer.transform[0] = texelFetchOffset(sLayers, uv0, 0, ivec2(0, 0));
    layer.transform[1] = texelFetchOffset(sLayers, uv0, 0, ivec2(1, 0));
    layer.transform[2] = texelFetchOffset(sLayers, uv0, 0, ivec2(2, 0));
    layer.transform[3] = texelFetchOffset(sLayers, uv0, 0, ivec2(3, 0));

    layer.inv_transform[0] = texelFetchOffset(sLayers, uv0, 0, ivec2(4, 0));
    layer.inv_transform[1] = texelFetchOffset(sLayers, uv0, 0, ivec2(5, 0));
    layer.inv_transform[2] = texelFetchOffset(sLayers, uv0, 0, ivec2(6, 0));
    layer.inv_transform[3] = texelFetchOffset(sLayers, uv0, 0, ivec2(7, 0));

    layer.local_clip_rect = texelFetchOffset(sLayers, uv1, 0, ivec2(0, 0));

    layer.screen_vertices[0] = texelFetchOffset(sLayers, uv1, 0, ivec2(1, 0));
    layer.screen_vertices[1] = texelFetchOffset(sLayers, uv1, 0, ivec2(2, 0));
    layer.screen_vertices[2] = texelFetchOffset(sLayers, uv1, 0, ivec2(2, 0));
    layer.screen_vertices[3] = texelFetchOffset(sLayers, uv1, 0, ivec2(3, 0));

    return layer;
}

struct Tile {
    vec4 actual_rect;
    vec4 target_rect;
};

Tile fetch_tile(int index) {
    Tile tile;

    int y = index / TILES_PER_ROW;
    int x = VECS_PER_TILE * (index % TILES_PER_ROW);

    ivec2 uv = ivec2(x + 0, y);

    tile.actual_rect = texelFetchOffset(sRenderTasks, uv, 0, ivec2(0, 0));
    tile.target_rect = texelFetchOffset(sRenderTasks, uv, 0, ivec2(1, 0));

    return tile;
}

struct PrimitiveInfo {
    vec4 layer_tile;
    vec4 local_clip_rect;
    vec4 local_rect;
};

PrimitiveInfo unpack_prim_info(int offset) {
    PrimitiveInfo info;

    info.layer_tile = data[offset + 0];
    info.local_clip_rect = data[offset + 1];
    info.local_rect = data[offset + 2];

    return info;
}

struct ClipCorner {
    vec4 rect;
    vec4 outer_inner_radius;
};

ClipCorner unpack_clip_corner(int offset) {
    ClipCorner corner;

    corner.rect = data[offset + 0];
    corner.outer_inner_radius = data[offset + 1];

    return corner;
}

struct Clip {
    vec4 rect;
    ClipCorner top_left;
    ClipCorner top_right;
    ClipCorner bottom_left;
    ClipCorner bottom_right;
};

Clip unpack_clip(int offset) {
    Clip clip;

    clip.rect = data[offset + 0];
    clip.top_left = unpack_clip_corner(offset + 1);
    clip.top_right = unpack_clip_corner(offset + 3);
    clip.bottom_left = unpack_clip_corner(offset + 5);
    clip.bottom_right = unpack_clip_corner(offset + 7);

    return clip;
}

bool ray_plane(vec3 normal, vec3 point, vec3 ray_origin, vec3 ray_dir, out float t)
{
    float denom = dot(normal, ray_dir);
    if (denom > 1e-6) {
        vec3 d = point - ray_origin;
        t = dot(d, normal) / denom;
        return t >= 0.0;
    }

    return false;
}

vec4 untransform(vec2 ref, vec3 n, vec3 a, mat4 inv_transform) {
    vec3 p = vec3(ref, -10000.0);
    vec3 d = vec3(0, 0, 1.0);

    float t;
    ray_plane(n, a, p, d, t);
    vec3 c = p + d * t;

    vec4 r = inv_transform * vec4(c, 1.0);
    return r;
}

vec3 get_layer_pos(vec2 pos, Layer layer) {
    vec3 a = layer.screen_vertices[0].xyz / layer.screen_vertices[0].w;
    vec3 b = layer.screen_vertices[3].xyz / layer.screen_vertices[3].w;
    vec3 c = layer.screen_vertices[2].xyz / layer.screen_vertices[2].w;
    vec3 n = normalize(cross(b-a, c-a));
    vec4 local_pos = untransform(pos, n, a, layer.inv_transform);
    return local_pos.xyw;
}

struct Rect {
    vec2 p0;
    vec2 p1;
};

struct VertexInfo {
    Rect local_rect;
    vec2 local_clamped_pos;
    vec2 global_clamped_pos;
};

VertexInfo write_vertex(PrimitiveInfo info) {
    Layer layer = fetch_layer(int(info.layer_tile.x));
    Tile tile = fetch_tile(int(info.layer_tile.y));

    vec2 p0 = floor(0.5 + info.local_rect.xy * uDevicePixelRatio) / uDevicePixelRatio;
    vec2 p1 = floor(0.5 + (info.local_rect.xy + info.local_rect.zw) * uDevicePixelRatio) / uDevicePixelRatio;

    vec2 local_pos = mix(p0, p1, aPosition.xy);

    vec2 cp0 = floor(0.5 + info.local_clip_rect.xy * uDevicePixelRatio) / uDevicePixelRatio;
    vec2 cp1 = floor(0.5 + (info.local_clip_rect.xy + info.local_clip_rect.zw) * uDevicePixelRatio) / uDevicePixelRatio;
    local_pos = clamp(local_pos, cp0, cp1);

    local_pos = clamp(local_pos,
                      vec2(layer.local_clip_rect.xy),
                      vec2(layer.local_clip_rect.xy + layer.local_clip_rect.zw));

    vec4 world_pos = layer.transform * vec4(local_pos, 0, 1);
    world_pos.xyz /= world_pos.w;

    vec2 device_pos = world_pos.xy * uDevicePixelRatio;

    vec2 clamped_pos = clamp(device_pos,
                             vec2(tile.actual_rect.xy),
                             vec2(tile.actual_rect.xy + tile.actual_rect.zw));

    vec4 local_clamped_pos = layer.inv_transform * vec4(clamped_pos / uDevicePixelRatio, world_pos.z, 1);
    local_clamped_pos.xyz /= local_clamped_pos.w;

    vec2 final_pos = clamped_pos + vec2(tile.target_rect.xy) - vec2(tile.actual_rect.xy);

    gl_Position = uTransform * vec4(final_pos, 0, 1);

    VertexInfo vi = VertexInfo(Rect(p0, p1), local_clamped_pos.xy, clamped_pos.xy);
    return vi;
}

struct TransformVertexInfo {
    vec3 local_pos;
    vec4 clipped_local_rect;
};

TransformVertexInfo write_transform_vertex(PrimitiveInfo info) {
    Layer layer = fetch_layer(int(info.layer_tile.x));
    Tile tile = fetch_tile(int(info.layer_tile.y));

    vec2 lp0 = info.local_rect.xy;
    vec2 lp1 = info.local_rect.xy + info.local_rect.zw;

    lp0 = clamp(lp0,
                layer.local_clip_rect.xy,
                layer.local_clip_rect.xy + layer.local_clip_rect.zw);
    lp1 = clamp(lp1,
                layer.local_clip_rect.xy,
                layer.local_clip_rect.xy + layer.local_clip_rect.zw);

    vec4 clipped_local_rect = vec4(lp0, lp1 - lp0);

    vec2 p0 = lp0;
    vec2 p1 = vec2(lp1.x, lp0.y);
    vec2 p2 = vec2(lp0.x, lp1.y);
    vec2 p3 = lp1;

    vec4 t0 = layer.transform * vec4(p0, 0, 1);
    vec4 t1 = layer.transform * vec4(p1, 0, 1);
    vec4 t2 = layer.transform * vec4(p2, 0, 1);
    vec4 t3 = layer.transform * vec4(p3, 0, 1);

    vec2 tp0 = t0.xy / t0.w;
    vec2 tp1 = t1.xy / t1.w;
    vec2 tp2 = t2.xy / t2.w;
    vec2 tp3 = t3.xy / t3.w;

    vec2 min_pos = min(tp0.xy, min(tp1.xy, min(tp2.xy, tp3.xy)));
    vec2 max_pos = max(tp0.xy, max(tp1.xy, max(tp2.xy, tp3.xy)));

    vec2 min_pos_clamped = clamp(min_pos * uDevicePixelRatio,
                                 vec2(tile.actual_rect.xy),
                                 vec2(tile.actual_rect.xy + tile.actual_rect.zw));

    vec2 max_pos_clamped = clamp(max_pos * uDevicePixelRatio,
                                 vec2(tile.actual_rect.xy),
                                 vec2(tile.actual_rect.xy + tile.actual_rect.zw));

    vec2 clamped_pos = mix(min_pos_clamped,
                           max_pos_clamped,
                           aPosition.xy);

    vec3 layer_pos = get_layer_pos(clamped_pos / uDevicePixelRatio, layer);

    vec2 final_pos = clamped_pos + vec2(tile.target_rect.xy) - vec2(tile.actual_rect.xy);

    gl_Position = uTransform * vec4(final_pos, 0, 1);

    return TransformVertexInfo(layer_pos, clipped_local_rect);
}

struct Rectangle {
    PrimitiveInfo info;
    vec4 color;
};

Rectangle fetch_rectangle(int index) {
    Rectangle rect;

    int offset = index * 4;

    rect.info = unpack_prim_info(offset);
    rect.color = data[offset + 3];

    return rect;
}

struct RectangleClip {
    PrimitiveInfo info;
    vec4 color;
    Clip clip;
};

RectangleClip fetch_rectangle_clip(int index) {
    RectangleClip rect;

    int offset = index * 13;

    rect.info = unpack_prim_info(offset);
    rect.color = data[offset + 3];
    rect.clip = unpack_clip(offset + 4);

    return rect;
}

struct Glyph {
    PrimitiveInfo info;
    vec4 color;
    vec4 uv_rect;
};

Glyph fetch_glyph(int index) {
    Glyph glyph;

    int offset = index * 5;

    glyph.info = unpack_prim_info(offset);
    glyph.color = data[offset + 3];
    glyph.uv_rect = data[offset + 4];

    return glyph;
}

struct TextRunGlyph {
    vec4 local_rect;
    vec4 uv_rect;
};

struct TextRun {
    PrimitiveInfo info;
    vec4 color;
    TextRunGlyph glyphs[WR_GLYPHS_PER_TEXT_RUN];
};

PrimitiveInfo fetch_text_run_glyph(int index, out vec4 color, out vec4 uv_rect) {
    int offset = 20 * (index / WR_GLYPHS_PER_TEXT_RUN);
    int glyph_index = index % WR_GLYPHS_PER_TEXT_RUN;
    int glyph_offset = offset + 4 + 2 * glyph_index;

    PrimitiveInfo info;
    info.layer_tile = data[offset + 0];
    info.local_clip_rect = data[offset + 1];
    info.local_rect = data[glyph_offset + 0];

    color = data[offset + 3];
    uv_rect = data[glyph_offset + 1];

    return info;
}

struct Image {
    PrimitiveInfo info;
    vec4 st_rect;                        // Location of the image texture in the texture atlas.
    vec4 stretch_size_and_tile_spacing;  // Size of the actual image and amount of space between
                                         //     tiled instances of this image.
    vec4 uvkind;                         // Type of texture coordinates.
};

Image fetch_image(int index) {
    Image image;

    int offset = index * 6;

    image.info = unpack_prim_info(offset);
    image.st_rect = data[offset + 3];
    image.stretch_size_and_tile_spacing = data[offset + 4];
    image.uvkind = data[offset + 5];

    return image;
}

struct ImageClip {
    PrimitiveInfo info;
    vec4 st_rect;                        // Location of the image texture in the texture atlas.
    vec4 stretch_size_and_tile_spacing;  // Size of the actual image and amount of space between
                                         //     tiled instances of this image.
    vec4 uvkind;                         // Type of texture coordinates.
    Clip clip;
};

ImageClip fetch_image_clip(int index) {
    ImageClip image;

    int offset = index * 15;

    image.info = unpack_prim_info(offset);
    image.st_rect = data[offset + 3];
    image.stretch_size_and_tile_spacing = data[offset + 4];
    image.uvkind = data[offset + 5];
    image.clip = unpack_clip(offset + 6);

    return image;
}

struct Border {
    PrimitiveInfo info;
    vec4 verticalColor;
    vec4 horizontalColor;
    vec4 radii;
    vec4 border_style_trbl;
    vec4 part;
};

Border fetch_border(int index) {
    Border border;

    int offset = index * 8;

    border.info = unpack_prim_info(offset);
    border.verticalColor = data[offset + 3];
    border.horizontalColor = data[offset + 4];
    border.radii = data[offset + 5];
    border.border_style_trbl = data[offset + 6];
    border.part = data[offset + 7];

    return border;
}

struct BoxShadow {
    PrimitiveInfo info;
    vec4 color;
    vec4 border_radii_blur_radius_inverted;
    vec4 bs_rect;
    vec4 src_rect;
};

BoxShadow fetch_boxshadow(int index) {
    BoxShadow bs;

    int offset = index * 7;

    bs.info = unpack_prim_info(offset);
    bs.color = data[offset + 3];
    bs.border_radii_blur_radius_inverted = data[offset + 4];
    bs.bs_rect = data[offset + 5];
    bs.src_rect = data[offset + 6];

    return bs;
}

struct AlignedGradient {
    PrimitiveInfo info;
    vec4 color0;
    vec4 color1;
    vec4 dir;
    Clip clip;
};

AlignedGradient fetch_aligned_gradient(int index) {
    AlignedGradient gradient;

    int offset = index * 15;

    gradient.info = unpack_prim_info(offset);
    gradient.color0 = data[offset + 3];
    gradient.color1 = data[offset + 4];
    gradient.dir = data[offset + 5];
    gradient.clip = unpack_clip(offset + 6);

    return gradient;
}

struct AngleGradient {
    PrimitiveInfo info;
    vec4 start_end_point;
    vec4 stop_count;
    vec4 colors[MAX_STOPS_PER_ANGLE_GRADIENT];
    vec4 offsets[MAX_STOPS_PER_ANGLE_GRADIENT/4];
};

AngleGradient fetch_angle_gradient(int index) {
    AngleGradient gradient;

    int offset = index * 15;

    gradient.info = unpack_prim_info(offset);
    gradient.start_end_point = data[offset + 3];
    gradient.stop_count = data[offset + 4];

    for (int i=0 ; i < MAX_STOPS_PER_ANGLE_GRADIENT ; ++i) {
        gradient.colors[i] = data[offset + 5 + i];
    }

    for (int i=0 ; i < MAX_STOPS_PER_ANGLE_GRADIENT/4 ; ++i) {
        gradient.offsets[i] = data[offset + 5 + MAX_STOPS_PER_ANGLE_GRADIENT + i];
    }

    return gradient;
}

struct Blend {
    vec4 src_id_target_id_opacity;
};

Blend fetch_blend(int index) {
    Blend blend;

    int offset = index * 1;

    blend.src_id_target_id_opacity = data[offset + 0];

    return blend;
}

struct Composite {
    vec4 src0_src1_target_id;
    vec4 info_amount;
};

Composite fetch_composite(int index) {
    Composite composite;

    int offset = index * 2;

    composite.src0_src1_target_id = data[offset + 0];
    composite.info_amount = data[offset + 1];

    return composite;
}
#endif

#ifdef WR_FRAGMENT_SHADER
float squared_distance_from_rect(vec2 p, vec2 origin, vec2 size) {
    vec2 clamped = clamp(p, origin, origin + size);
    return distance(clamped, p);
}

vec2 init_transform_fs(vec3 local_pos, vec4 local_rect, out float fragment_alpha) {
    fragment_alpha = 1.0;
    vec2 pos = local_pos.xy / local_pos.z;

    float squared_distance = squared_distance_from_rect(pos, local_rect.xy, local_rect.zw);
    if (squared_distance != 0.0) {
        float delta = length(fwidth(local_pos.xy));
        fragment_alpha = smoothstep(1.0, 0.0, squared_distance / delta * 2.0);
    }

    return pos;
}
#endif
