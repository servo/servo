/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared,clip_shared

varying vec4 vLocalPos;
varying vec2 vUv;
flat varying vec4 vUvBounds;
flat varying vec4 vEdge;
flat varying vec4 vUvBounds_NoClamp;
#if defined(PLATFORM_ANDROID) && !defined(SWGL)
// Work around Adreno 3xx driver bug. See the v_perspective comment in
// brush_image or bug 1630356 for details.
flat varying vec2 vClipModeVec;
#define vClipMode vClipModeVec.x
#else
flat varying float vClipMode;
#endif

#define MODE_STRETCH        0
#define MODE_SIMPLE         1

#ifdef WR_VERTEX_SHADER

PER_INSTANCE in ivec2 aClipDataResourceAddress;
PER_INSTANCE in vec2 aClipSrcRectSize;
PER_INSTANCE in int aClipMode;
PER_INSTANCE in ivec2 aStretchMode;
PER_INSTANCE in vec4 aClipDestRect;

struct ClipMaskInstanceBoxShadow {
    ClipMaskInstanceCommon base;
    ivec2 resource_address;
};

ClipMaskInstanceBoxShadow fetch_clip_item() {
    ClipMaskInstanceBoxShadow cmi;

    cmi.base = fetch_clip_item_common();
    cmi.resource_address = aClipDataResourceAddress;

    return cmi;
}

struct BoxShadowData {
    vec2 src_rect_size;
    int clip_mode;
    int stretch_mode_x;
    int stretch_mode_y;
    RectWithSize dest_rect;
};

BoxShadowData fetch_data() {
    BoxShadowData bs_data = BoxShadowData(
        aClipSrcRectSize,
        aClipMode,
        aStretchMode.x,
        aStretchMode.y,
        RectWithSize(aClipDestRect.xy, aClipDestRect.zw)
    );
    return bs_data;
}

void main(void) {
    ClipMaskInstanceBoxShadow cmi = fetch_clip_item();
    Transform clip_transform = fetch_transform(cmi.base.clip_transform_id);
    Transform prim_transform = fetch_transform(cmi.base.prim_transform_id);
    BoxShadowData bs_data = fetch_data();
    ImageSource res = fetch_image_source_direct(cmi.resource_address);

    RectWithSize dest_rect = bs_data.dest_rect;

    ClipVertexInfo vi = write_clip_tile_vertex(
        dest_rect,
        prim_transform,
        clip_transform,
        cmi.base.sub_rect,
        cmi.base.task_origin,
        cmi.base.screen_origin,
        cmi.base.device_pixel_scale
    );
    vClipMode = float(bs_data.clip_mode);

    vec2 texture_size = vec2(TEX_SIZE(sColor0));
    vec2 local_pos = vi.local_pos.xy / vi.local_pos.w;
    vLocalPos = vi.local_pos;

    switch (bs_data.stretch_mode_x) {
        case MODE_STRETCH: {
            vEdge.x = 0.5;
            vEdge.z = (dest_rect.size.x / bs_data.src_rect_size.x) - 0.5;
            vUv.x = (local_pos.x - dest_rect.p0.x) / bs_data.src_rect_size.x;
            break;
        }
        case MODE_SIMPLE:
        default: {
            vEdge.xz = vec2(1.0);
            vUv.x = (local_pos.x - dest_rect.p0.x) / dest_rect.size.x;
            break;
        }
    }

    switch (bs_data.stretch_mode_y) {
        case MODE_STRETCH: {
            vEdge.y = 0.5;
            vEdge.w = (dest_rect.size.y / bs_data.src_rect_size.y) - 0.5;
            vUv.y = (local_pos.y - dest_rect.p0.y) / bs_data.src_rect_size.y;
            break;
        }
        case MODE_SIMPLE:
        default: {
            vEdge.yw = vec2(1.0);
            vUv.y = (local_pos.y - dest_rect.p0.y) / dest_rect.size.y;
            break;
        }
    }

    vUv *= vi.local_pos.w;
    vec2 uv0 = res.uv_rect.p0;
    vec2 uv1 = res.uv_rect.p1;
    vUvBounds = vec4(uv0 + vec2(0.5), uv1 - vec2(0.5)) / texture_size.xyxy;
    vUvBounds_NoClamp = vec4(uv0, uv1) / texture_size.xyxy;
}
#endif

#ifdef WR_FRAGMENT_SHADER
void main(void) {
    vec2 uv_linear = vUv / vLocalPos.w;
    vec2 uv = clamp(uv_linear, vec2(0.0), vEdge.xy);
    uv += max(vec2(0.0), uv_linear - vEdge.zw);
    uv = mix(vUvBounds_NoClamp.xy, vUvBounds_NoClamp.zw, uv);
    uv = clamp(uv, vUvBounds.xy, vUvBounds.zw);

    float in_shadow_rect = init_transform_rough_fs(vLocalPos.xy / vLocalPos.w);

    float texel = TEX_SAMPLE(sColor0, uv).r;

    float alpha = mix(texel, 1.0 - texel, vClipMode);
    float result = vLocalPos.w > 0.0 ? mix(vClipMode, alpha, in_shadow_rect) : 0.0;

    oFragColor = vec4(result);
}

#ifdef SWGL_DRAW_SPAN
// As with cs_clip_rectangle, this shader spends a lot of time doing clipping and
// combining for every fragment, even if outside of the primitive to initialize
// the clip tile, or inside the inner bounds of the primitive, where the shadow
// is unnecessary. To alleviate this, the span shader attempts to first intersect
// the the local clip bounds, outside of which we can just use a solid fill
// to initialize those clip tile fragments. Once inside the primitive bounds,
// we further intersect with the inner region where no shadow is necessary either
// so that we can commit entire spans of texture within this nine-patch region
// instead of having to do the work of mapping per fragment.
void swgl_drawSpanR8() {
    // If the span is completely outside the Z-range and clipped out, just
    // output clear so we don't need to consider invalid W in the rest of the
    // shader.
    float w = swgl_forceScalar(vLocalPos.w);
    if (w <= 0.0) {
        swgl_commitSolidR8(0.0);
        return;
    }

    // To start, we evaluate the box shadow in both UV and local space relative
    // to the local-space position. This will be interpolated across the span to
    // track whether we intersect the nine-patch.
    w = 1.0 / w;
    vec2 uv_linear = vUv * w;
    vec2 uv_linear0 = swgl_forceScalar(uv_linear);
    vec2 uv_linear_step = swgl_interpStep(vUv).xy * w;
    vec2 local_pos = vLocalPos.xy * w;
    vec2 local_pos0 = swgl_forceScalar(local_pos);
    vec2 local_step = swgl_interpStep(vLocalPos).xy * w;

    // We need to compute the local-space distance to the bounding box and then
    // figure out how many processing steps that maps to. If we are stepping in
    // a negative direction on an axis, we need to swap the sides of the box
    // which we consider as the start or end. If there is no local-space step
    // on an axis (i.e. constant Y), we need to take care to force the steps to
    // either the start or end of the span depending on if we are inside or
    // outside of the bounding box.
    vec4 clip_dist =
        mix(vTransformBounds, vTransformBounds.zwxy, lessThan(local_step, vec2(0.0)).xyxy)
            - local_pos0.xyxy;
    clip_dist =
        mix(1.0e6 * step(0.0, clip_dist),
            clip_dist * recip(local_step).xyxy,
            notEqual(local_step, vec2(0.0)).xyxy);

    // Find the start and end of the shadowed region on this span.
    float shadow_start = max(clip_dist.x, clip_dist.y);
    float shadow_end = min(clip_dist.z, clip_dist.w);

    // Flip the offsets from the start of the span so we can compare against the
    // remaining span length which automatically deducts as we commit fragments.
    ivec2 shadow_steps = ivec2(clamp(
        swgl_SpanLength - swgl_StepSize * vec2(floor(shadow_start), ceil(shadow_end)),
        0.0, swgl_SpanLength));
    int shadow_start_len = shadow_steps.x;
    int shadow_end_len = shadow_steps.y;

    // Likewise, once inside the primitive bounds, we also need to track which
    // sector of the nine-patch we are in which requires intersecting against
    // the inner box instead of the outer box.
    vec4 opaque_dist =
        mix(vEdge, vEdge.zwxy, lessThan(uv_linear_step, vec2(0.0)).xyxy)
            - uv_linear0.xyxy;
    opaque_dist =
        mix(1.0e6 * step(0.0, opaque_dist),
            opaque_dist * recip(uv_linear_step).xyxy,
            notEqual(uv_linear_step, vec2(0.0)).xyxy);

    // Unlike for the shadow clipping bounds, here we need to rather find the floor of all
    // the offsets so that we don't accidentally process any chunks in the transitional areas
    // between sectors of the nine-patch.
    ivec4 opaque_steps = ivec4(clamp(
        swgl_SpanLength -
            swgl_StepSize *
                vec4(floor(opaque_dist.x), floor(opaque_dist.y), floor(opaque_dist.z), floor(opaque_dist.w)),
        shadow_end_len, swgl_SpanLength));

    // Fill any initial sections of the span that are clipped out based on clip mode.
    if (swgl_SpanLength > shadow_start_len) {
        int num_before = swgl_SpanLength - shadow_start_len;
        swgl_commitPartialSolidR8(num_before, vClipMode);
        float steps_before = float(num_before / swgl_StepSize);
        uv_linear += steps_before * uv_linear_step;
        local_pos += steps_before * local_step;
    }

    // This loop tries to repeatedly process entire spans of the nine-patch that map
    // to a contiguous spans of texture in the source box shadow. First, we process
    // a chunk with per-fragment clipping and mapping in case we're starting on a
    // transitional region between sectors of the nine-patch which may need to map
    // to different spans of texture per-fragment. After, we find the largest span
    // within the current sector before we hit the next transitional region, and
    // attempt to commit an entire span of texture therein.
    while (swgl_SpanLength > 0) {
        // Here we might be in a transitional chunk, so do everything per-fragment.
        {
            vec2 uv = clamp(uv_linear, vec2(0.0), vEdge.xy);
            uv += max(vec2(0.0), uv_linear - vEdge.zw);
            uv = mix(vUvBounds_NoClamp.xy, vUvBounds_NoClamp.zw, uv);
            uv = clamp(uv, vUvBounds.xy, vUvBounds.zw);

            float in_shadow_rect = init_transform_rough_fs(local_pos);

            float texel = TEX_SAMPLE(sColor0, uv).r;

            float alpha = mix(texel, 1.0 - texel, vClipMode);
            float result = mix(vClipMode, alpha, in_shadow_rect);
            swgl_commitColorR8(result);

            uv_linear += uv_linear_step;
            local_pos += local_step;
        }
        // If we now hit the end of the clip bounds, just bail out since there is
        // no more shadow to map.
        if (swgl_SpanLength <= shadow_end_len) {
            break;
        }
        // By here we've determined to be still inside the nine-patch. We need to
        // compare against the inner rectangle thresholds to see which sector of
        // the nine-patch to use and thus how to map the box shadow texture. Stop
        // at least one step before the end of the shadow region to properly clip
        // on the boundary.
        int num_inside = swgl_SpanLength - swgl_StepSize - shadow_end_len;
        vec4 uv_bounds = vUvBounds;
        if (swgl_SpanLength >= opaque_steps.y) {
            // We're in the top Y band of the nine-patch.
            num_inside = min(num_inside, swgl_SpanLength - opaque_steps.y);
        } else if (swgl_SpanLength >= opaque_steps.w) {
            // We're in the middle Y band of the nine-patch. Set the UV clamp bounds
            // to the vertical center texel of the box shadow.
            num_inside = min(num_inside, swgl_SpanLength - opaque_steps.w);
            uv_bounds.yw = vec2(clamp(mix(vUvBounds_NoClamp.y, vUvBounds_NoClamp.w, vEdge.y),
                                      vUvBounds.y, vUvBounds.w));
        }
        if (swgl_SpanLength >= opaque_steps.x) {
            // We're in the left X column of the nine-patch.
            num_inside = min(num_inside, swgl_SpanLength - opaque_steps.x);
        } else if (swgl_SpanLength >= opaque_steps.z) {
            // We're in the middle X band of the nine-patch. Set the UV clamp bounds
            // to the horizontal center texel of the box shadow.
            num_inside = min(num_inside, swgl_SpanLength - opaque_steps.z);
            uv_bounds.xz = vec2(clamp(mix(vUvBounds_NoClamp.x, vUvBounds_NoClamp.z, vEdge.x),
                                      vUvBounds.x, vUvBounds.z));
        }
        if (num_inside > 0) {
            // We have a non-zero span of fragments within the sector. Map to the UV
            // start offset of the sector and the UV offset within the sector.
            vec2 uv = clamp(uv_linear, vec2(0.0), vEdge.xy);
            uv += max(vec2(0.0), uv_linear - vEdge.zw);
            uv = mix(vUvBounds_NoClamp.xy, vUvBounds_NoClamp.zw, uv);
            // If we're in the center sector of the nine-patch, then we only need to
            // sample from a single texel of the box shadow. Just sample that single
            // texel once and output it for the entire span. Otherwise, we just need
            // to commit an actual span of texture from the box shadow. Depending on
            // if we are in clip-out mode, we may need to invert the source texture.
            if (uv_bounds.xy == uv_bounds.zw) {
                uv = clamp(uv, uv_bounds.xy, uv_bounds.zw);
                float texel = TEX_SAMPLE(sColor0, uv).r;
                float alpha = mix(texel, 1.0 - texel, vClipMode);
                swgl_commitPartialSolidR8(num_inside, alpha);
            } else if (vClipMode != 0.0) {
                swgl_commitPartialTextureLinearInvertR8(num_inside, sColor0, uv, uv_bounds);
            } else {
                swgl_commitPartialTextureLinearR8(num_inside, sColor0, uv, uv_bounds);
            }
            float steps_inside = float(num_inside / swgl_StepSize);
            uv_linear += steps_inside * uv_linear_step;
            local_pos += steps_inside * local_step;
        }
        // By here we're probably in a transitional chunk of the nine-patch that
        // requires per-fragment processing, so loop around again to the handler
        // for that case.
    }

    // Fill any remaining sections of the span that are clipped out.
    if (swgl_SpanLength > 0) {
        swgl_commitPartialSolidR8(swgl_SpanLength, vClipMode);
    }
}
#endif

#endif
