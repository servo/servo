/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared,prim_shared

flat varying vec4 v_color;
flat varying vec3 v_mask_swizzle;
// Normalized bounds of the source image in the texture.
flat varying vec4 v_uv_bounds;

// Interpolated UV coordinates to sample.
varying vec2 v_uv;


#if defined(WR_FEATURE_GLYPH_TRANSFORM) && !defined(SWGL_CLIP_DIST)
varying vec4 v_uv_clip;
#endif

#ifdef WR_VERTEX_SHADER

#define VECS_PER_TEXT_RUN           2
#define GLYPHS_PER_GPU_BLOCK        2U

#ifdef WR_FEATURE_GLYPH_TRANSFORM
RectWithSize transform_rect(RectWithSize rect, mat2 transform) {
    vec2 center = transform * (rect.p0 + rect.size * 0.5);
    vec2 radius = mat2(abs(transform[0]), abs(transform[1])) * (rect.size * 0.5);
    return RectWithSize(center - radius, radius * 2.0);
}

bool rect_inside_rect(RectWithSize little, RectWithSize big) {
    return all(lessThanEqual(vec4(big.p0, little.p0 + little.size),
                             vec4(little.p0, big.p0 + big.size)));
}
#endif //WR_FEATURE_GLYPH_TRANSFORM

struct Glyph {
    vec2 offset;
};

Glyph fetch_glyph(int specific_prim_address,
                  int glyph_index) {
    // Two glyphs are packed in each texel in the GPU cache.
    int glyph_address = specific_prim_address +
                        VECS_PER_TEXT_RUN +
                        int(uint(glyph_index) / GLYPHS_PER_GPU_BLOCK);
    vec4 data = fetch_from_gpu_cache_1(glyph_address);
    // Select XY or ZW based on glyph index.
    vec2 glyph = mix(data.xy, data.zw,
                     bvec2(uint(glyph_index) % GLYPHS_PER_GPU_BLOCK == 1U));

    return Glyph(glyph);
}

struct GlyphResource {
    vec4 uv_rect;
    vec2 offset;
    float scale;
};

GlyphResource fetch_glyph_resource(int address) {
    vec4 data[2] = fetch_from_gpu_cache_2(address);
    return GlyphResource(data[0], data[1].xy, data[1].z);
}

struct TextRun {
    vec4 color;
    vec4 bg_color;
};

TextRun fetch_text_run(int address) {
    vec4 data[2] = fetch_from_gpu_cache_2(address);
    return TextRun(data[0], data[1]);
}

vec2 get_snap_bias(int subpx_dir) {
    // In subpixel mode, the subpixel offset has already been
    // accounted for while rasterizing the glyph. However, we
    // must still round with a subpixel bias rather than rounding
    // to the nearest whole pixel, depending on subpixel direciton.
    switch (subpx_dir) {
        case SUBPX_DIR_NONE:
        default:
            return vec2(0.5);
        case SUBPX_DIR_HORIZONTAL:
            // Glyphs positioned [-0.125, 0.125] get a
            // subpx position of zero. So include that
            // offset in the glyph position to ensure
            // we round to the correct whole position.
            return vec2(0.125, 0.5);
        case SUBPX_DIR_VERTICAL:
            return vec2(0.5, 0.125);
        case SUBPX_DIR_MIXED:
            return vec2(0.125);
    }
}

void main() {
    Instance instance = decode_instance_attributes();
    PrimitiveHeader ph = fetch_prim_header(instance.prim_header_address);
    Transform transform = fetch_transform(ph.transform_id);
    ClipArea clip_area = fetch_clip_area(instance.clip_address);
    PictureTask task = fetch_picture_task(instance.picture_task_address);

    int glyph_index = instance.segment_index;
    int subpx_dir = (instance.flags >> 8) & 0xff;
    int color_mode = instance.flags & 0xff;

    // Note that the reference frame relative offset is stored in the prim local
    // rect size during batching, instead of the actual size of the primitive.
    TextRun text = fetch_text_run(ph.specific_prim_address);
    vec2 text_offset = ph.local_rect.size;

    if (color_mode == COLOR_MODE_FROM_PASS) {
        color_mode = uMode;
    }

    // Note that the unsnapped reference frame relative offset has already
    // been subtracted from the prim local rect origin during batching.
    // It was done this way to avoid pushing both the snapped and the
    // unsnapped offsets to the shader.
    Glyph glyph = fetch_glyph(ph.specific_prim_address, glyph_index);
    glyph.offset += ph.local_rect.p0;

    GlyphResource res = fetch_glyph_resource(instance.resource_address);

    vec2 snap_bias = get_snap_bias(subpx_dir);

    // Glyph space refers to the pixel space used by glyph rasterization during frame
    // building. If a non-identity transform was used, WR_FEATURE_GLYPH_TRANSFORM will
    // be set. Otherwise, regardless of whether the raster space is LOCAL or SCREEN,
    // we ignored the transform during glyph rasterization, and need to snap just using
    // the device pixel scale and the raster scale.
#ifdef WR_FEATURE_GLYPH_TRANSFORM
    // Transform from local space to glyph space.
    mat2 glyph_transform = mat2(transform.m) * task.device_pixel_scale;
    vec2 glyph_translation = transform.m[3].xy * task.device_pixel_scale;

    // Transform from glyph space back to local space.
    mat2 glyph_transform_inv = inverse(glyph_transform);

    // Glyph raster pixels include the impact of the transform. This path can only be
    // entered for 3d transforms that can be coerced into a 2d transform; they have no
    // perspective, and have a 2d inverse. This is a looser condition than axis aligned
    // transforms because it also allows 2d rotations.
    vec2 raster_glyph_offset = floor(glyph_transform * glyph.offset + snap_bias);

    // We want to eliminate any subpixel translation in device space to ensure glyph
    // snapping is stable for equivalent glyph subpixel positions. Note that we must take
    // into account the translation from the transform for snapping purposes.
    vec2 raster_text_offset = floor(glyph_transform * text_offset + glyph_translation + 0.5) - glyph_translation;

    // Compute the glyph rect in glyph space.
    RectWithSize glyph_rect = RectWithSize(res.offset + raster_glyph_offset + raster_text_offset,
                                           res.uv_rect.zw - res.uv_rect.xy);

    // The glyph rect is in glyph space, so transform it back to local space.
    RectWithSize local_rect = transform_rect(glyph_rect, glyph_transform_inv);

    // Select the corner of the glyph's local space rect that we are processing.
    vec2 local_pos = local_rect.p0 + local_rect.size * aPosition.xy;

    // If the glyph's local rect would fit inside the local clip rect, then select a corner from
    // the device space glyph rect to reduce overdraw of clipped pixels in the fragment shader.
    // Otherwise, fall back to clamping the glyph's local rect to the local clip rect.
    if (rect_inside_rect(local_rect, ph.local_clip_rect)) {
        local_pos = glyph_transform_inv * (glyph_rect.p0 + glyph_rect.size * aPosition.xy);
    }
#else
    float raster_scale = float(ph.user_data.x) / 65535.0;

    // Scale in which the glyph is snapped when rasterized.
    float glyph_raster_scale = raster_scale * task.device_pixel_scale;

    // Scale from glyph space to local space.
    float glyph_scale_inv = res.scale / glyph_raster_scale;

    // Glyph raster pixels do not include the impact of the transform. Instead it was
    // replaced with an identity transform during glyph rasterization. As such only the
    // impact of the raster scale (if in local space) and the device pixel scale (for both
    // local and screen space) are included.
    //
    // This implies one or more of the following conditions:
    // - The transform is an identity. In that case, setting WR_FEATURE_GLYPH_TRANSFORM
    //   should have the same output result as not. We just distingush which path to use
    //   based on the transform used during glyph rasterization. (Screen space).
    // - The transform contains an animation. We will imply local raster space in such
    //   cases to avoid constantly rerasterizing the glyphs.
    // - The transform has perspective or does not have a 2d inverse (Screen or local space).
    // - The transform's scale will result in result in very large rasterized glyphs and
    //   we clamped the size. This will imply local raster space.
    vec2 raster_glyph_offset = floor(glyph.offset * glyph_raster_scale + snap_bias) / res.scale;

    // Compute the glyph rect in local space.
    //
    // The transform may be animated, so we don't want to do any snapping here for the
    // text offset to avoid glyphs wiggling. The text offset should have been snapped
    // already for axis aligned transforms excluding any animations during frame building.
    RectWithSize glyph_rect = RectWithSize(glyph_scale_inv * (res.offset + raster_glyph_offset) + text_offset,
                                           glyph_scale_inv * (res.uv_rect.zw - res.uv_rect.xy));

    // Select the corner of the glyph rect that we are processing.
    vec2 local_pos = glyph_rect.p0 + glyph_rect.size * aPosition.xy;
#endif

    VertexInfo vi = write_vertex(
        local_pos,
        ph.local_clip_rect,
        ph.z,
        transform,
        task
    );

#ifdef WR_FEATURE_GLYPH_TRANSFORM
    vec2 f = (glyph_transform * vi.local_pos - glyph_rect.p0) / glyph_rect.size;
    #ifdef SWGL_CLIP_DIST
        gl_ClipDistance[0] = f.x;
        gl_ClipDistance[1] = f.y;
        gl_ClipDistance[2] = 1.0 - f.x;
        gl_ClipDistance[3] = 1.0 - f.y;
    #else
        v_uv_clip = vec4(f, 1.0 - f);
    #endif
#else
    vec2 f = (vi.local_pos - glyph_rect.p0) / glyph_rect.size;
#endif

    write_clip(vi.world_pos, clip_area, task);

    switch (color_mode) {
        case COLOR_MODE_ALPHA:
            v_mask_swizzle = vec3(0.0, 1.0, 1.0);
            v_color = text.color;
            break;
        case COLOR_MODE_BITMAP_SHADOW:
            #ifdef SWGL_BLEND
                swgl_blendDropShadow(text.color);
                v_mask_swizzle = vec3(1.0, 0.0, 0.0);
                v_color = vec4(1.0);
            #else
                v_mask_swizzle = vec3(0.0, 1.0, 0.0);
                v_color = text.color;
            #endif
            break;
        case COLOR_MODE_SUBPX_BG_PASS2:
            v_mask_swizzle = vec3(1.0, 0.0, 0.0);
            v_color = text.color;
            break;
        case COLOR_MODE_SUBPX_CONST_COLOR:
        case COLOR_MODE_SUBPX_BG_PASS0:
        case COLOR_MODE_COLOR_BITMAP:
            v_mask_swizzle = vec3(1.0, 0.0, 0.0);
            v_color = vec4(text.color.a);
            break;
        case COLOR_MODE_SUBPX_BG_PASS1:
            v_mask_swizzle = vec3(-1.0, 1.0, 0.0);
            v_color = vec4(text.color.a) * text.bg_color;
            break;
        case COLOR_MODE_SUBPX_DUAL_SOURCE:
            #ifdef SWGL_BLEND
                swgl_blendSubpixelText(text.color);
                v_mask_swizzle = vec3(1.0, 0.0, 0.0);
                v_color = vec4(1.0);
            #else
                v_mask_swizzle = vec3(text.color.a, 0.0, 0.0);
                v_color = text.color;
            #endif
            break;
        default:
            v_mask_swizzle = vec3(0.0, 0.0, 0.0);
            v_color = vec4(1.0);
    }

    vec2 texture_size = vec2(TEX_SIZE(sColor0));
    vec2 st0 = res.uv_rect.xy / texture_size;
    vec2 st1 = res.uv_rect.zw / texture_size;

    v_uv = mix(st0, st1, f);
    v_uv_bounds = (res.uv_rect + vec4(0.5, 0.5, -0.5, -0.5)) / texture_size.xyxy;
}

#endif // WR_VERTEX_SHADER

#ifdef WR_FRAGMENT_SHADER

Fragment text_fs(void) {
    Fragment frag;

    vec2 tc = clamp(v_uv, v_uv_bounds.xy, v_uv_bounds.zw);
    vec4 mask = texture(sColor0, tc);
    // v_mask_swizzle.z != 0 means we are using an R8 texture as alpha,
    // and therefore must swizzle from the r channel to all channels.
    mask = mix(mask, mask.rrrr, bvec4(v_mask_swizzle.z != 0.0));
    #ifndef WR_FEATURE_DUAL_SOURCE_BLENDING
        mask.rgb = mask.rgb * v_mask_swizzle.x + mask.aaa * v_mask_swizzle.y;
    #endif

    #if defined(WR_FEATURE_GLYPH_TRANSFORM) && !defined(SWGL_CLIP_DIST)
        mask *= float(all(greaterThanEqual(v_uv_clip, vec4(0.0))));
    #endif

    frag.color = v_color * mask;

    #if defined(WR_FEATURE_DUAL_SOURCE_BLENDING) && !defined(SWGL_BLEND)
        frag.blend = mask * v_mask_swizzle.x + mask.aaaa * v_mask_swizzle.y;
    #endif

    return frag;
}


void main() {
    Fragment frag = text_fs();

    float clip_mask = do_clip();
    frag.color *= clip_mask;

    #if defined(WR_FEATURE_DEBUG_OVERDRAW)
        oFragColor = WR_DEBUG_OVERDRAW_COLOR;
    #elif defined(WR_FEATURE_DUAL_SOURCE_BLENDING) && !defined(SWGL_BLEND)
        oFragColor = frag.color;
        oFragBlend = frag.blend * clip_mask;
    #else
        write_output(frag.color);
    #endif
}

#if defined(SWGL_DRAW_SPAN) && defined(SWGL_BLEND) && defined(SWGL_CLIP_DIST)
void swgl_drawSpanRGBA8() {
    // Only support simple swizzles for now. More complex swizzles must either
    // be handled by blend overrides or the slow path.
    if (v_mask_swizzle.x != 0.0 && v_mask_swizzle.x != 1.0) {
        return;
    }

    #ifdef WR_FEATURE_DUAL_SOURCE_BLENDING
        swgl_commitTextureLinearRGBA8(sColor0, v_uv, v_uv_bounds);
    #else
        if (swgl_isTextureR8(sColor0)) {
            swgl_commitTextureLinearColorR8ToRGBA8(sColor0, v_uv, v_uv_bounds, v_color);
        } else {
            swgl_commitTextureLinearColorRGBA8(sColor0, v_uv, v_uv_bounds, v_color);
        }
    #endif
}
#endif

#endif // WR_FRAGMENT_SHADER
