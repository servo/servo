#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define PST_TOP_LEFT     0
#define PST_TOP          1
#define PST_TOP_RIGHT    2
#define PST_RIGHT        3
#define PST_BOTTOM_RIGHT 4
#define PST_BOTTOM       5
#define PST_BOTTOM_LEFT  6
#define PST_LEFT         7

#define BORDER_LEFT      0
#define BORDER_TOP       1
#define BORDER_RIGHT     2
#define BORDER_BOTTOM    3

#define UV_NORMALIZED    uint(0)
#define UV_PIXEL         uint(1)

// Border styles as defined in webrender_traits/types.rs
#define BORDER_STYLE_NONE         0
#define BORDER_STYLE_SOLID        1
#define BORDER_STYLE_DOUBLE       2
#define BORDER_STYLE_DOTTED       3
#define BORDER_STYLE_DASHED       4
#define BORDER_STYLE_HIDDEN       5
#define BORDER_STYLE_GROOVE       6
#define BORDER_STYLE_RIDGE        7
#define BORDER_STYLE_INSET        8
#define BORDER_STYLE_OUTSET       9

#define MAX_STOPS_PER_ANGLE_GRADIENT 8

uniform sampler2DArray sCache;

#ifdef WR_VERTEX_SHADER

#define VECS_PER_LAYER             13
#define VECS_PER_RENDER_TASK        2
#define VECS_PER_PRIM_GEOM          2

#define GRADIENT_HORIZONTAL     0
#define GRADIENT_VERTICAL       1
#define GRADIENT_ROTATED        2

uniform sampler2D sLayers;
uniform sampler2D sRenderTasks;
uniform sampler2D sPrimGeometry;
uniform sampler2D sClips;

uniform sampler2D sData16;
uniform sampler2D sData32;
uniform sampler2D sData64;
uniform sampler2D sData128;

ivec2 get_fetch_uv(int index, int vecs_per_item) {
    int items_per_row = WR_MAX_VERTEX_TEXTURE_WIDTH / vecs_per_item;
    int y = index / items_per_row;
    int x = vecs_per_item * (index % items_per_row);
    return ivec2(x, y);
}

ivec2 get_fetch_uv_1(int index) {
    return get_fetch_uv(index, 1);
}

ivec2 get_fetch_uv_2(int index) {
    return get_fetch_uv(index, 2);
}

ivec2 get_fetch_uv_4(int index) {
    return get_fetch_uv(index, 4);
}

ivec2 get_fetch_uv_8(int index) {
    return get_fetch_uv(index, 8);
}

struct Layer {
    mat4 transform;
    mat4 inv_transform;
    vec4 local_clip_rect;
    vec4 screen_vertices[4];
};

layout(std140) uniform Data {
    ivec4 int_data[WR_MAX_UBO_VECTORS];
};

Layer fetch_layer(int index) {
    Layer layer;

    // Create a UV base coord for each 8 texels.
    // This is required because trying to use an offset
    // of more than 8 texels doesn't work on some versions
    // of OSX.
    ivec2 uv = get_fetch_uv(index, VECS_PER_LAYER);
    ivec2 uv0 = ivec2(uv.x + 0, uv.y);
    ivec2 uv1 = ivec2(uv.x + 8, uv.y);

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
    layer.screen_vertices[2] = texelFetchOffset(sLayers, uv1, 0, ivec2(3, 0));
    layer.screen_vertices[3] = texelFetchOffset(sLayers, uv1, 0, ivec2(4, 0));

    return layer;
}

struct RenderTaskData {
    vec4 data0;
    vec4 data1;
};

RenderTaskData fetch_render_task(int index) {
    RenderTaskData task;

    ivec2 uv = get_fetch_uv(index, VECS_PER_RENDER_TASK);

    task.data0 = texelFetchOffset(sRenderTasks, uv, 0, ivec2(0, 0));
    task.data1 = texelFetchOffset(sRenderTasks, uv, 0, ivec2(1, 0));

    return task;
}

struct Tile {
    vec4 screen_origin_task_origin;
    vec4 size_target_index;
};

Tile fetch_tile(int index) {
    RenderTaskData task = fetch_render_task(index);

    Tile tile;
    tile.screen_origin_task_origin = task.data0;
    tile.size_target_index = task.data1;

    return tile;
}

struct Gradient {
    vec4 start_end_point;
    vec4 kind;
};

Gradient fetch_gradient(int index) {
    Gradient gradient;

    ivec2 uv = get_fetch_uv_2(index);

    gradient.start_end_point = texelFetchOffset(sData32, uv, 0, ivec2(0, 0));
    gradient.kind = texelFetchOffset(sData32, uv, 0, ivec2(1, 0));

    return gradient;
}

struct GradientStop {
    vec4 color;
    vec4 offset;
};

GradientStop fetch_gradient_stop(int index) {
    GradientStop stop;

    ivec2 uv = get_fetch_uv_2(index);

    stop.color = texelFetchOffset(sData32, uv, 0, ivec2(0, 0));
    stop.offset = texelFetchOffset(sData32, uv, 0, ivec2(1, 0));

    return stop;
}

struct Glyph {
    vec4 offset;
    vec4 uv_rect;
};

Glyph fetch_glyph(int index) {
    Glyph glyph;

    ivec2 uv = get_fetch_uv_2(index);

    glyph.offset = texelFetchOffset(sData32, uv, 0, ivec2(0, 0));
    glyph.uv_rect = texelFetchOffset(sData32, uv, 0, ivec2(1, 0));

    return glyph;
}

struct Border {
    vec4 style;
    vec4 widths;
    vec4 colors[4];
    vec4 radii[2];
};

Border fetch_border(int index) {
    Border border;

    ivec2 uv = get_fetch_uv_8(index);

    border.style = texelFetchOffset(sData128, uv, 0, ivec2(0, 0));
    border.widths = texelFetchOffset(sData128, uv, 0, ivec2(1, 0));
    border.colors[0] = texelFetchOffset(sData128, uv, 0, ivec2(2, 0));
    border.colors[1] = texelFetchOffset(sData128, uv, 0, ivec2(3, 0));
    border.colors[2] = texelFetchOffset(sData128, uv, 0, ivec2(4, 0));
    border.colors[3] = texelFetchOffset(sData128, uv, 0, ivec2(5, 0));
    border.radii[0] = texelFetchOffset(sData128, uv, 0, ivec2(6, 0));
    border.radii[1] = texelFetchOffset(sData128, uv, 0, ivec2(7, 0));

    return border;
}

vec4 fetch_instance_geometry(int index) {
    ivec2 uv = get_fetch_uv_1(index);

    vec4 rect = texelFetchOffset(sData16, uv, 0, ivec2(0, 0));

    return rect;
}

struct PrimitiveGeometry {
    vec4 local_rect;
    vec4 local_clip_rect;
};

PrimitiveGeometry fetch_prim_geometry(int index) {
    PrimitiveGeometry pg;

    ivec2 uv = get_fetch_uv(index, VECS_PER_PRIM_GEOM);

    pg.local_rect = texelFetchOffset(sPrimGeometry, uv, 0, ivec2(0, 0));
    pg.local_clip_rect = texelFetchOffset(sPrimGeometry, uv, 0, ivec2(1, 0));

    return pg;
}

struct PrimitiveInstance {
    int global_prim_index;
    int specific_prim_index;
    int render_task_index;
    int layer_index;
    int clip_address;
    int sub_index;
    ivec2 user_data;
};

PrimitiveInstance fetch_instance(int index) {
    PrimitiveInstance pi;

    int offset = index * 2;

    ivec4 data0 = int_data[offset + 0];
    ivec4 data1 = int_data[offset + 1];

    pi.global_prim_index = data0.x;
    pi.specific_prim_index = data0.y;
    pi.render_task_index = data0.z;
    pi.layer_index = data0.w;
    pi.clip_address = data1.x;
    pi.sub_index = data1.y;
    pi.user_data = data1.zw;

    return pi;
}

struct CachePrimitiveInstance {
    int global_prim_index;
    int specific_prim_index;
    int render_task_index;
};

CachePrimitiveInstance fetch_cache_instance(int index) {
    CachePrimitiveInstance cpi;

    int offset = index * 1;

    ivec4 data0 = int_data[offset + 0];

    cpi.global_prim_index = data0.x;
    cpi.specific_prim_index = data0.y;
    cpi.render_task_index = data0.z;

    return cpi;
}

struct Primitive {
    Layer layer;
    Tile tile;
    vec4 local_rect;
    vec4 local_clip_rect;
    int prim_index;
    int clip_index;
    // when sending multiple primitives of the same type (e.g. border segments)
    // this index allows the vertex shader to recognize the difference
    int sub_index;
    ivec2 user_data;
};

Primitive load_primitive(int index) {
    Primitive prim;

    PrimitiveInstance pi = fetch_instance(index);

    prim.layer = fetch_layer(pi.layer_index);
    prim.tile = fetch_tile(pi.render_task_index);

    PrimitiveGeometry pg = fetch_prim_geometry(pi.global_prim_index);
    prim.local_rect = pg.local_rect;
    prim.local_clip_rect = pg.local_clip_rect;

    prim.prim_index = pi.specific_prim_index;
    prim.clip_index = pi.clip_address;
    prim.sub_index = pi.sub_index;
    prim.user_data = pi.user_data;

    return prim;
}

struct ClipRect {
    vec4 rect;
    vec4 dummy;
};

ClipRect fetch_clip_rect(int index) {
    ClipRect rect;

    ivec2 uv = get_fetch_uv_2(index);

    rect.rect = texelFetchOffset(sData32, uv, 0, ivec2(0, 0));
    //rect.dummy = texelFetchOffset(sData32, uv, 0, ivec2(1, 0));
    rect.dummy = vec4(0.0, 0.0, 0.0, 0.0);

    return rect;
}

struct ImageMaskData {
    vec4 uv_rect;
    vec4 local_rect;
};

ImageMaskData fetch_mask_data(int index) {
    ImageMaskData info;

    ivec2 uv = get_fetch_uv_2(index);

    info.uv_rect = texelFetchOffset(sData32, uv, 0, ivec2(0, 0));
    info.local_rect = texelFetchOffset(sData32, uv, 0, ivec2(1, 0));

    return info;
}

struct ClipCorner {
    vec4 rect;
    vec4 outer_inner_radius;
};

ClipCorner fetch_clip_corner(int index) {
    ClipCorner corner;

    ivec2 uv = get_fetch_uv_2(index);

    corner.rect = texelFetchOffset(sData32, uv, 0, ivec2(0, 0));
    corner.outer_inner_radius = texelFetchOffset(sData32, uv, 0, ivec2(1, 0));

    return corner;
}

struct ClipData {
    ClipRect rect;
    ClipCorner top_left;
    ClipCorner top_right;
    ClipCorner bottom_left;
    ClipCorner bottom_right;
    ImageMaskData mask_data;
};

ClipData fetch_clip(int index) {
    ClipData clip;

    clip.rect = fetch_clip_rect(index + 0);
    clip.top_left = fetch_clip_corner(index + 1);
    clip.top_right = fetch_clip_corner(index + 2);
    clip.bottom_left = fetch_clip_corner(index + 3);
    clip.bottom_right = fetch_clip_corner(index + 4);
    clip.mask_data = fetch_mask_data(index + 5);

    return clip;
}

// Return the intersection of the plane (set up by "normal" and "point")
// with the ray (set up by "ray_origin" and "ray_dir"),
// writing the resulting scaler into "t".
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

// Apply the inverse transform "inv_transform"
// to the reference point "ref" in CSS space,
// producing a local point on a layer plane,
// set by a base point "a" and a normal "n".
vec4 untransform(vec2 ref, vec3 n, vec3 a, mat4 inv_transform) {
    vec3 p = vec3(ref, -10000.0);
    vec3 d = vec3(0, 0, 1.0);

    float t = 0.0;
    // get an intersection of the layer plane with Z axis vector,
    // originated from the "ref" point
    ray_plane(n, a, p, d, t);
    float z = p.z + d.z * t; // Z of the visible point on the layer

    vec4 r = inv_transform * vec4(ref, z, 1.0);
    return r;
}

// Given a CSS space position, transform it back into the layer space.
vec4 get_layer_pos(vec2 pos, Layer layer) {
    // get 3 of the layer corners in CSS space
    vec3 a = layer.screen_vertices[0].xyz / layer.screen_vertices[0].w;
    vec3 b = layer.screen_vertices[3].xyz / layer.screen_vertices[3].w;
    vec3 c = layer.screen_vertices[2].xyz / layer.screen_vertices[2].w;
    // get the normal to the layer plane
    vec3 n = normalize(cross(b-a, c-a));
    return untransform(pos, n, a, layer.inv_transform);
}

vec2 clamp_rect(vec2 point, vec4 rect) {
    return clamp(point, rect.xy, rect.xy + rect.zw);
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

VertexInfo write_vertex(vec4 instance_rect,
                        vec4 local_clip_rect,
                        Layer layer,
                        Tile tile) {
    vec2 p0 = floor(0.5 + instance_rect.xy * uDevicePixelRatio) / uDevicePixelRatio;
    vec2 p1 = floor(0.5 + (instance_rect.xy + instance_rect.zw) * uDevicePixelRatio) / uDevicePixelRatio;

    vec2 local_pos = mix(p0, p1, aPosition.xy);

    vec2 cp0 = floor(0.5 + local_clip_rect.xy * uDevicePixelRatio) / uDevicePixelRatio;
    vec2 cp1 = floor(0.5 + (local_clip_rect.xy + local_clip_rect.zw) * uDevicePixelRatio) / uDevicePixelRatio;
    local_pos = clamp(local_pos, cp0, cp1);

    local_pos = clamp_rect(local_pos, layer.local_clip_rect);

    vec4 world_pos = layer.transform * vec4(local_pos, 0, 1);
    world_pos.xyz /= world_pos.w;

    vec2 device_pos = world_pos.xy * uDevicePixelRatio;

    vec2 clamped_pos = clamp(device_pos,
                             tile.screen_origin_task_origin.xy,
                             tile.screen_origin_task_origin.xy + tile.size_target_index.xy);

    vec4 local_clamped_pos = layer.inv_transform * vec4(clamped_pos / uDevicePixelRatio, world_pos.z, 1);
    local_clamped_pos.xyz /= local_clamped_pos.w;

    vec2 final_pos = clamped_pos + tile.screen_origin_task_origin.zw - tile.screen_origin_task_origin.xy;

    gl_Position = uTransform * vec4(final_pos, 0, 1);

    VertexInfo vi = VertexInfo(Rect(p0, p1), local_clamped_pos.xy, clamped_pos.xy);
    return vi;
}

#ifdef WR_FEATURE_TRANSFORM

struct TransformVertexInfo {
    vec3 local_pos;
    vec4 clipped_local_rect;
};

TransformVertexInfo write_transform_vertex(vec4 instance_rect,
                                           vec4 local_clip_rect,
                                           Layer layer,
                                           Tile tile) {
    vec2 lp0_base = instance_rect.xy;
    vec2 lp1_base = instance_rect.xy + instance_rect.zw;

    vec2 lp0 = clamp_rect(clamp_rect(lp0_base, local_clip_rect),
                          layer.local_clip_rect);
    vec2 lp1 = clamp_rect(clamp_rect(lp1_base, local_clip_rect),
                          layer.local_clip_rect);

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

    // compute a CSS space aligned bounding box
    vec2 min_pos = min(min(tp0.xy, tp1.xy), min(tp2.xy, tp3.xy));
    vec2 max_pos = max(max(tp0.xy, tp1.xy), max(tp2.xy, tp3.xy));

    // clamp to the tile boundaries, in device space
    vec2 min_pos_clamped = clamp(min_pos * uDevicePixelRatio,
                                 tile.screen_origin_task_origin.xy,
                                 tile.screen_origin_task_origin.xy + tile.size_target_index.xy);

    vec2 max_pos_clamped = clamp(max_pos * uDevicePixelRatio,
                                 tile.screen_origin_task_origin.xy,
                                 tile.screen_origin_task_origin.xy + tile.size_target_index.xy);

    // compute the device space position of this vertex
    vec2 clamped_pos = mix(min_pos_clamped,
                           max_pos_clamped,
                           aPosition.xy);

    // compute the point position in side the layer, in CSS space
    vec4 layer_pos = get_layer_pos(clamped_pos / uDevicePixelRatio, layer);

    // apply the task offset
    vec2 final_pos = clamped_pos + tile.screen_origin_task_origin.zw - tile.screen_origin_task_origin.xy;

    gl_Position = uTransform * vec4(final_pos, 0, 1);

    return TransformVertexInfo(layer_pos.xyw, clipped_local_rect);
}

#endif //WR_FEATURE_TRANSFORM

struct Rectangle {
    vec4 color;
};

Rectangle fetch_rectangle(int index) {
    Rectangle rect;

    ivec2 uv = get_fetch_uv_1(index);

    rect.color = texelFetchOffset(sData16, uv, 0, ivec2(0, 0));

    return rect;
}

struct TextRun {
    vec4 color;
};

TextRun fetch_text_run(int index) {
    TextRun text;

    ivec2 uv = get_fetch_uv_1(index);

    text.color = texelFetchOffset(sData16, uv, 0, ivec2(0, 0));

    return text;
}

struct Image {
    vec4 st_rect;                        // Location of the image texture in the texture atlas.
    vec4 stretch_size_and_tile_spacing;  // Size of the actual image and amount of space between
                                         //     tiled instances of this image.
};

Image fetch_image(int index) {
    Image image;

    ivec2 uv = get_fetch_uv_2(index);

    image.st_rect = texelFetchOffset(sData32, uv, 0, ivec2(0, 0));
    image.stretch_size_and_tile_spacing = texelFetchOffset(sData32, uv, 0, ivec2(1, 0));

    return image;
}

struct BoxShadow {
    vec4 src_rect;
    vec4 bs_rect;
    vec4 color;
    vec4 border_radius_edge_size_blur_radius_inverted;
};

BoxShadow fetch_boxshadow(int index) {
    BoxShadow bs;

    ivec2 uv = get_fetch_uv_4(index);

    bs.src_rect = texelFetchOffset(sData64, uv, 0, ivec2(0, 0));
    bs.bs_rect = texelFetchOffset(sData64, uv, 0, ivec2(1, 0));
    bs.color = texelFetchOffset(sData64, uv, 0, ivec2(2, 0));
    bs.border_radius_edge_size_blur_radius_inverted = texelFetchOffset(sData64, uv, 0, ivec2(3, 0));

    return bs;
}

struct Blend {
    ivec4 src_id_target_id_op_amount;
};

Blend fetch_blend(int index) {
    Blend blend;

    int offset = index * 1;
    blend.src_id_target_id_op_amount = int_data[offset + 0];

    return blend;
}

struct Composite {
    ivec4 src0_src1_target_id_op;
};

Composite fetch_composite(int index) {
    Composite composite;

    int offset = index * 1;

    composite.src0_src1_target_id_op = int_data[offset + 0];

    return composite;
}
#endif

#ifdef WR_FRAGMENT_SHADER
float distance_from_rect(vec2 p, vec2 origin, vec2 size) {
    vec2 clamped = clamp(p, origin, origin + size);
    return distance(clamped, p);
}

vec2 init_transform_fs(vec3 local_pos, vec4 local_rect, out float fragment_alpha) {
    fragment_alpha = 1.0;
    vec2 pos = local_pos.xy / local_pos.z;

    float border_distance = distance_from_rect(pos, local_rect.xy, local_rect.zw);
    if (border_distance != 0.0) {
        float delta = length(fwidth(local_pos.xy));
        fragment_alpha = 1.0 - smoothstep(0.0, 1.0, border_distance / delta * 2.0);
    }

    return pos;
}
#endif
