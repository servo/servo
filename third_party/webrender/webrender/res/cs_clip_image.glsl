/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared,clip_shared

varying vec2 vLocalPos;
varying vec2 vClipMaskImageUv;

flat varying vec4 vClipMaskUvInnerRect;

#ifdef WR_VERTEX_SHADER

PER_INSTANCE in vec4 aClipTileRect;
PER_INSTANCE in ivec2 aClipDataResourceAddress;
PER_INSTANCE in vec4 aClipLocalRect;

struct ClipMaskInstanceImage {
    ClipMaskInstanceCommon base;
    RectWithSize tile_rect;
    ivec2 resource_address;
    RectWithSize local_rect;
};

ClipMaskInstanceImage fetch_clip_item() {
    ClipMaskInstanceImage cmi;

    cmi.base = fetch_clip_item_common();

    cmi.tile_rect = RectWithSize(aClipTileRect.xy, aClipTileRect.zw);
    cmi.resource_address = aClipDataResourceAddress;
    cmi.local_rect = RectWithSize(aClipLocalRect.xy, aClipLocalRect.zw);

    return cmi;
}

struct ClipImageVertexInfo {
    vec2 local_pos;
    vec4 world_pos;
};

// This differs from write_clip_tile_vertex in that we forward transform the
// primitive's local-space tile rect into the target space. We use scissoring
// to ensure that the primitive does not draw outside the target bounds.
ClipImageVertexInfo write_clip_image_vertex(RectWithSize tile_rect,
                                            RectWithSize local_clip_rect,
                                            Transform prim_transform,
                                            Transform clip_transform,
                                            RectWithSize sub_rect,
                                            vec2 task_origin,
                                            vec2 screen_origin,
                                            float device_pixel_scale) {
    vec2 local_pos = clamp_rect(tile_rect.p0 + aPosition.xy * tile_rect.size, local_clip_rect);
    vec4 world_pos = prim_transform.m * vec4(local_pos, 0.0, 1.0);
    vec4 final_pos = vec4(
        world_pos.xy * device_pixel_scale + (task_origin - screen_origin) * world_pos.w,
        0.0,
        world_pos.w
    );
    gl_Position = uTransform * final_pos;

    init_transform_vs(
        clip_transform.is_axis_aligned
            ? vec4(vec2(-1.0e16), vec2(1.0e16))
            : vec4(local_clip_rect.p0, local_clip_rect.p0 + local_clip_rect.size));

    ClipImageVertexInfo vi = ClipImageVertexInfo(local_pos, world_pos);
    return vi;
}

void main(void) {
    ClipMaskInstanceImage cmi = fetch_clip_item();
    Transform clip_transform = fetch_transform(cmi.base.clip_transform_id);
    Transform prim_transform = fetch_transform(cmi.base.prim_transform_id);
    ImageSource res = fetch_image_source_direct(cmi.resource_address);

    ClipImageVertexInfo vi = write_clip_image_vertex(
        cmi.tile_rect,
        cmi.local_rect,
        prim_transform,
        clip_transform,
        cmi.base.sub_rect,
        cmi.base.task_origin,
        cmi.base.screen_origin,
        cmi.base.device_pixel_scale
    );
    vLocalPos = vi.local_pos;
    vec2 uv = (vi.local_pos - cmi.tile_rect.p0) / cmi.tile_rect.size;

    vec2 texture_size = vec2(TEX_SIZE(sColor0));
    vec4 uv_rect = vec4(res.uv_rect.p0, res.uv_rect.p1);
    vClipMaskImageUv = mix(uv_rect.xy, uv_rect.zw, uv) / texture_size;

    // applying a half-texel offset to the UV boundaries to prevent linear samples from the outside
    vClipMaskUvInnerRect = (uv_rect + vec4(0.5, 0.5, -0.5, -0.5)) / texture_size.xyxy;
}
#endif

#ifdef WR_FRAGMENT_SHADER
void main(void) {
    float alpha = init_transform_rough_fs(vLocalPos);
    vec2 source_uv = clamp(vClipMaskImageUv, vClipMaskUvInnerRect.xy, vClipMaskUvInnerRect.zw);
    float clip_alpha = texture(sColor0, source_uv).r; //careful: texture has type A8
    oFragColor = vec4(mix(1.0, clip_alpha, alpha), 0.0, 0.0, 1.0);
}

#ifdef SWGL_DRAW_SPAN
void swgl_drawSpanR8() {
    if (has_valid_transform_bounds()) {
        return;
    }

    swgl_commitTextureLinearR8(sColor0, vClipMaskImageUv, vClipMaskUvInnerRect);
}
#endif

#endif
