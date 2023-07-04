/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define VECS_PER_IMAGE_BRUSH 3
#define VECS_PER_SPECIFIC_BRUSH VECS_PER_IMAGE_BRUSH

#define WR_BRUSH_VS_FUNCTION image_brush_vs
#define WR_BRUSH_FS_FUNCTION image_brush_fs

#include shared,prim_shared,brush

#ifdef WR_FEATURE_ALPHA_PASS
#define V_LOCAL_POS         varying_vec4_0.xy
#endif

// Interpolated UV coordinates to sample.
#define V_UV                varying_vec4_0.zw

#ifdef WR_FEATURE_ALPHA_PASS
#define V_COLOR             flat_varying_vec4_0
#define V_MASK_SWIZZLE      flat_varying_vec4_1.xy
#define V_TILE_REPEAT       flat_varying_vec4_1.zw
#endif

// Normalized bounds of the source image in the texture.
#define V_UV_BOUNDS         flat_varying_vec4_2
// Normalized bounds of the source image in the texture, adjusted to avoid
// sampling artifacts.
#define V_UV_SAMPLE_BOUNDS  flat_varying_vec4_3
// Layer index to sample.
#define V_LAYER             flat_varying_vec4_4.x
// Flag to allow perspective interpolation of UV.
#define V_PERSPECTIVE       flat_varying_vec4_4.y

#ifdef WR_VERTEX_SHADER

// Must match the AlphaType enum.
#define BLEND_MODE_ALPHA            0
#define BLEND_MODE_PREMUL_ALPHA     1

struct ImageBrushData {
    vec4 color;
    vec4 background_color;
    vec2 stretch_size;
};

ImageBrushData fetch_image_data(int address) {
    vec4[3] raw_data = fetch_from_gpu_cache_3(address);
    ImageBrushData data = ImageBrushData(
        raw_data[0],
        raw_data[1],
        raw_data[2].xy
    );
    return data;
}

void image_brush_vs(
    VertexInfo vi,
    int prim_address,
    RectWithSize prim_rect,
    RectWithSize segment_rect,
    ivec4 prim_user_data,
    int specific_resource_address,
    mat4 transform,
    PictureTask pic_task,
    int brush_flags,
    vec4 segment_data
) {
    ImageBrushData image_data = fetch_image_data(prim_address);

    // If this is in WR_FEATURE_TEXTURE_RECT mode, the rect and size use
    // non-normalized texture coordinates.
#ifdef WR_FEATURE_TEXTURE_RECT
    vec2 texture_size = vec2(1, 1);
#else
    vec2 texture_size = vec2(textureSize(sColor0, 0));
#endif

    ImageResource res = fetch_image_resource(specific_resource_address);
    vec2 uv0 = res.uv_rect.p0;
    vec2 uv1 = res.uv_rect.p1;

    RectWithSize local_rect = prim_rect;
    vec2 stretch_size = image_data.stretch_size;
    if (stretch_size.x < 0.0) {
        stretch_size = local_rect.size;
    }

    // If this segment should interpolate relative to the
    // segment, modify the parameters for that.
    if ((brush_flags & BRUSH_FLAG_SEGMENT_RELATIVE) != 0) {
        local_rect = segment_rect;
        stretch_size = local_rect.size;

        // If the extra data is a texel rect, modify the UVs.
        if ((brush_flags & BRUSH_FLAG_TEXEL_RECT) != 0) {
            vec2 uv_size = res.uv_rect.p1 - res.uv_rect.p0;
            uv0 = res.uv_rect.p0 + segment_data.xy * uv_size;
            uv1 = res.uv_rect.p0 + segment_data.zw * uv_size;

            // Size of the uv rect of the segment we are considering when computing
            // the repetitions. In most case it is the current segment, but for the
            // middle area we look at the border size instead.
            vec2 segment_uv_size = uv1 - uv0;
            #ifdef WR_FEATURE_REPETITION
            // Value of the stretch size with repetition. We have to compute it for
            // both axis even if we only repeat on one axis because the value for
            // each axis depends on what the repeated value would have been for the
            // other axis.
            vec2 repeated_stretch_size = stretch_size;
            // The repetition parameters for the middle area of a nine-patch are based
            // on the size of the border segments rather than the middle segment itself,
            // taking top and left by default, falling back to bottom and right when a
            // size is empty.
            // TODO(bug 1609893): Move this logic to the CPU as well as other sources of
            // branchiness in this shader.
            if ((brush_flags & BRUSH_FLAG_SEGMENT_NINEPATCH_MIDDLE) != 0) {
                segment_uv_size = uv0 - res.uv_rect.p0;
                repeated_stretch_size = segment_rect.p0 - prim_rect.p0;
                float epsilon = 0.001;

                if (segment_uv_size.x < epsilon || repeated_stretch_size.x < epsilon) {
                    segment_uv_size.x = res.uv_rect.p1.x - uv1.x;
                    repeated_stretch_size.x = prim_rect.p0.x + prim_rect.size.x
                        - segment_rect.p0.x - segment_rect.size.x;
                }

                if (segment_uv_size.y < epsilon || repeated_stretch_size.y < epsilon) {
                    segment_uv_size.y = res.uv_rect.p1.y - uv1.y;
                    repeated_stretch_size.y = prim_rect.p0.y + prim_rect.size.y
                        - segment_rect.p0.y - segment_rect.size.y;
                }
            }

            if ((brush_flags & BRUSH_FLAG_SEGMENT_REPEAT_X) != 0) {
              stretch_size.x = repeated_stretch_size.y / segment_uv_size.y * segment_uv_size.x;
            }
            if ((brush_flags & BRUSH_FLAG_SEGMENT_REPEAT_Y) != 0) {
              stretch_size.y = repeated_stretch_size.x / segment_uv_size.x * segment_uv_size.y;
            }
            #endif

        } else {
            #ifdef WR_FEATURE_REPETITION
            if ((brush_flags & BRUSH_FLAG_SEGMENT_REPEAT_X) != 0) {
                stretch_size.x = segment_data.z - segment_data.x;
            }
            if ((brush_flags & BRUSH_FLAG_SEGMENT_REPEAT_Y) != 0) {
                stretch_size.y = segment_data.w - segment_data.y;
            }
            #endif
        }

        #ifdef WR_FEATURE_REPETITION
        if ((brush_flags & BRUSH_FLAG_SEGMENT_REPEAT_X_ROUND) != 0) {
            float nx = max(1.0, round(segment_rect.size.x / stretch_size.x));
            stretch_size.x = segment_rect.size.x / nx;
        }
        if ((brush_flags & BRUSH_FLAG_SEGMENT_REPEAT_Y_ROUND) != 0) {
            float ny = max(1.0, round(segment_rect.size.y / stretch_size.y));
            stretch_size.y = segment_rect.size.y / ny;
        }
        #endif
    }

    float perspective_interpolate = (brush_flags & BRUSH_FLAG_PERSPECTIVE_INTERPOLATION) != 0 ? 1.0 : 0.0;
    V_LAYER = res.layer;
    V_PERSPECTIVE = perspective_interpolate;

    // Handle case where the UV coords are inverted (e.g. from an
    // external image).
    vec2 min_uv = min(uv0, uv1);
    vec2 max_uv = max(uv0, uv1);

    V_UV_SAMPLE_BOUNDS = vec4(
        min_uv + vec2(0.5),
        max_uv - vec2(0.5)
    ) / texture_size.xyxy;

    vec2 f = (vi.local_pos - local_rect.p0) / local_rect.size;

#ifdef WR_FEATURE_ALPHA_PASS
    int color_mode = prim_user_data.x & 0xffff;
    int blend_mode = prim_user_data.x >> 16;
    int raster_space = prim_user_data.y;

    if (color_mode == COLOR_MODE_FROM_PASS) {
        color_mode = uMode;
    }

    // Derive the texture coordinates for this image, based on
    // whether the source image is a local-space or screen-space
    // image.
    switch (raster_space) {
        case RASTER_SCREEN: {
            // Since the screen space UVs specify an arbitrary quad, do
            // a bilinear interpolation to get the correct UV for this
            // local position.
            f = get_image_quad_uv(specific_resource_address, f);
            break;
        }
        default:
            break;
    }
#endif

    // Offset and scale V_UV here to avoid doing it in the fragment shader.
    vec2 repeat = local_rect.size / stretch_size;
    V_UV = mix(uv0, uv1, f) - min_uv;
    V_UV /= texture_size;
    V_UV *= repeat.xy;
    if (perspective_interpolate == 0.0) {
        V_UV *= vi.world_pos.w;
    }

#ifdef WR_FEATURE_TEXTURE_RECT
    V_UV_BOUNDS = vec4(0.0, 0.0, vec2(textureSize(sColor0)));
#else
    V_UV_BOUNDS = vec4(min_uv, max_uv) / texture_size.xyxy;
#endif

#ifdef WR_FEATURE_ALPHA_PASS
    V_TILE_REPEAT = repeat.xy;

    float opacity = float(prim_user_data.z) / 65535.0;
    switch (blend_mode) {
        case BLEND_MODE_ALPHA:
            image_data.color.a *= opacity;
            break;
        case BLEND_MODE_PREMUL_ALPHA:
        default:
            image_data.color *= opacity;
            break;
    }

    switch (color_mode) {
        case COLOR_MODE_ALPHA:
        case COLOR_MODE_BITMAP:
            V_MASK_SWIZZLE = vec2(0.0, 1.0);
            V_COLOR = image_data.color;
            break;
        case COLOR_MODE_SUBPX_BG_PASS2:
        case COLOR_MODE_SUBPX_DUAL_SOURCE:
        case COLOR_MODE_IMAGE:
            V_MASK_SWIZZLE = vec2(1.0, 0.0);
            V_COLOR = image_data.color;
            break;
        case COLOR_MODE_SUBPX_CONST_COLOR:
        case COLOR_MODE_SUBPX_BG_PASS0:
        case COLOR_MODE_COLOR_BITMAP:
            V_MASK_SWIZZLE = vec2(1.0, 0.0);
            V_COLOR = vec4(image_data.color.a);
            break;
        case COLOR_MODE_SUBPX_BG_PASS1:
            V_MASK_SWIZZLE = vec2(-1.0, 1.0);
            V_COLOR = vec4(image_data.color.a) * image_data.background_color;
            break;
        default:
            V_MASK_SWIZZLE = vec2(0.0);
            V_COLOR = vec4(1.0);
    }

    V_LOCAL_POS = vi.local_pos;
#endif
}
#endif

#ifdef WR_FRAGMENT_SHADER

vec2 compute_repeated_uvs(float perspective_divisor) {
    vec2 uv_size = V_UV_BOUNDS.zw - V_UV_BOUNDS.xy;

#ifdef WR_FEATURE_ALPHA_PASS
    // This prevents the uv on the top and left parts of the primitive that was inflated
    // for anti-aliasing purposes from going beyound the range covered by the regular
    // (non-inflated) primitive.
    vec2 local_uv = max(V_UV * perspective_divisor, vec2(0.0));

    // Handle horizontal and vertical repetitions.
    vec2 repeated_uv = mod(local_uv, uv_size) + V_UV_BOUNDS.xy;

    // This takes care of the bottom and right inflated parts.
    // We do it after the modulo because the latter wraps around the values exactly on
    // the right and bottom edges, which we do not want.
    if (local_uv.x >= V_TILE_REPEAT.x * uv_size.x) {
        repeated_uv.x = V_UV_BOUNDS.z;
    }
    if (local_uv.y >= V_TILE_REPEAT.y * uv_size.y) {
        repeated_uv.y = V_UV_BOUNDS.w;
    }
#else
    vec2 repeated_uv = mod(V_UV * perspective_divisor, uv_size) + V_UV_BOUNDS.xy;
#endif

    return repeated_uv;
}

Fragment image_brush_fs() {
    float perspective_divisor = mix(gl_FragCoord.w, 1.0, V_PERSPECTIVE);

#ifdef WR_FEATURE_REPETITION
    vec2 repeated_uv = compute_repeated_uvs(perspective_divisor);
#else
    vec2 repeated_uv = V_UV * perspective_divisor + V_UV_BOUNDS.xy;
#endif

    // Clamp the uvs to avoid sampling artifacts.
    vec2 uv = clamp(repeated_uv, V_UV_SAMPLE_BOUNDS.xy, V_UV_SAMPLE_BOUNDS.zw);

    vec4 texel = TEX_SAMPLE(sColor0, vec3(uv, V_LAYER));

    Fragment frag;

#ifdef WR_FEATURE_ALPHA_PASS
    #ifdef WR_FEATURE_ANTIALIASING
        float alpha = init_transform_fs(V_LOCAL_POS);
    #else
        float alpha = 1.0;
    #endif
    texel.rgb = texel.rgb * V_MASK_SWIZZLE.x + texel.aaa * V_MASK_SWIZZLE.y;

    vec4 alpha_mask = texel * alpha;
    frag.color = V_COLOR * alpha_mask;

    #ifdef WR_FEATURE_DUAL_SOURCE_BLENDING
        frag.blend = alpha_mask * V_COLOR.a;
    #endif
#else
    frag.color = texel;
#endif

    return frag;
}
#endif

// Undef macro names that could be re-defined by other shaders.
#undef V_LOCAL_POS
#undef V_UV
#undef V_COLOR
#undef V_MASK_SWIZZLE
#undef V_TILE_REPEAT
#undef V_UV_BOUNDS
#undef V_UV_SAMPLE_BOUNDS
#undef V_LAYER
#undef V_PERSPECTIVE
