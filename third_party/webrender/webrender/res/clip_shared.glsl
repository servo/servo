/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include rect,render_task,gpu_cache,transform

#ifdef WR_VERTEX_SHADER

PER_INSTANCE in ivec2 aTransformIds;
PER_INSTANCE in ivec4 aClipDataResourceAddress;
PER_INSTANCE in vec2 aClipLocalPos;
PER_INSTANCE in vec4 aClipTileRect;
PER_INSTANCE in vec4 aClipDeviceArea;
PER_INSTANCE in vec4 aClipOrigins;
PER_INSTANCE in float aDevicePixelScale;

struct ClipMaskInstance {
    int clip_transform_id;
    int prim_transform_id;
    ivec2 clip_data_address;
    ivec2 resource_address;
    vec2 local_pos;
    RectWithSize tile_rect;
    RectWithSize sub_rect;
    vec2 task_origin;
    vec2 screen_origin;
    float device_pixel_scale;
};

ClipMaskInstance fetch_clip_item() {
    ClipMaskInstance cmi;

    cmi.clip_transform_id = aTransformIds.x;
    cmi.prim_transform_id = aTransformIds.y;
    cmi.clip_data_address = aClipDataResourceAddress.xy;
    cmi.resource_address = aClipDataResourceAddress.zw;
    cmi.local_pos = aClipLocalPos;
    cmi.tile_rect = RectWithSize(aClipTileRect.xy, aClipTileRect.zw);
    cmi.sub_rect = RectWithSize(aClipDeviceArea.xy, aClipDeviceArea.zw);
    cmi.task_origin = aClipOrigins.xy;
    cmi.screen_origin = aClipOrigins.zw;
    cmi.device_pixel_scale = aDevicePixelScale;

    return cmi;
}

struct ClipVertexInfo {
    vec4 local_pos;
    RectWithSize clipped_local_rect;
};

RectWithSize intersect_rect(RectWithSize a, RectWithSize b) {
    vec4 p = clamp(vec4(a.p0, a.p0 + a.size), b.p0.xyxy, b.p0.xyxy + b.size.xyxy);
    return RectWithSize(p.xy, max(vec2(0.0), p.zw - p.xy));
}


// The transformed vertex function that always covers the whole clip area,
// which is the intersection of all clip instances of a given primitive
ClipVertexInfo write_clip_tile_vertex(RectWithSize local_clip_rect,
                                      Transform prim_transform,
                                      Transform clip_transform,
                                      RectWithSize sub_rect,
                                      vec2 task_origin,
                                      vec2 screen_origin,
                                      float device_pixel_scale) {
    vec2 device_pos = screen_origin + sub_rect.p0 + aPosition.xy * sub_rect.size;
    vec2 world_pos = device_pos / device_pixel_scale;

    vec4 pos = prim_transform.m * vec4(world_pos, 0.0, 1.0);
    pos.xyz /= pos.w;

    vec4 p = get_node_pos(pos.xy, clip_transform);
    vec4 local_pos = p * pos.w;

    //TODO: Interpolate in clip space, where "local_pos.w" contains
    // the W of the homogeneous transform *from* clip space into the world.
    //    float interpolate_w = 1.0 / local_pos.w;
    // This is problematic today, because the W<=0 hemisphere is going to be
    // clipped, while we currently want this shader to fill out the whole rect.
    // We can therefore simplify this when the clip construction is rewritten
    // to only affect the areas touched by a clip.
    vec4 vertex_pos = vec4(
        task_origin + sub_rect.p0 + aPosition.xy * sub_rect.size,
        0.0,
        1.0
    );

    gl_Position = uTransform * vertex_pos;

    init_transform_vs(vec4(local_clip_rect.p0, local_clip_rect.p0 + local_clip_rect.size));

    ClipVertexInfo vi = ClipVertexInfo(local_pos, local_clip_rect);
    return vi;
}

#endif //WR_VERTEX_SHADER
