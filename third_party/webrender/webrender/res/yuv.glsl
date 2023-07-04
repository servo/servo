/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared

#define YUV_FORMAT_NV12             0
#define YUV_FORMAT_PLANAR           1
#define YUV_FORMAT_INTERLEAVED      2

#ifdef WR_VERTEX_SHADER

#ifdef WR_FEATURE_TEXTURE_RECT
    #define TEX_SIZE(sampler) vec2(1.0)
#else
    #define TEX_SIZE(sampler) vec2(textureSize(sampler, 0).xy)
#endif

#define YUV_COLOR_SPACE_REC601      0
#define YUV_COLOR_SPACE_REC709      1
#define YUV_COLOR_SPACE_REC2020     2

// The constants added to the Y, U and V components are applied in the fragment shader.

// From Rec601:
// [R]   [1.1643835616438356,  0.0,                 1.5960267857142858   ]   [Y -  16]
// [G] = [1.1643835616438358, -0.3917622900949137, -0.8129676472377708   ] x [U - 128]
// [B]   [1.1643835616438356,  2.017232142857143,   8.862867620416422e-17]   [V - 128]
//
// For the range [0,1] instead of [0,255].
//
// The matrix is stored in column-major.
const mat3 YuvColorMatrixRec601 = mat3(
    1.16438,  1.16438, 1.16438,
    0.0,     -0.39176, 2.01723,
    1.59603, -0.81297, 0.0
);

// From Rec709:
// [R]   [1.1643835616438356,  0.0,                    1.7927410714285714]   [Y -  16]
// [G] = [1.1643835616438358, -0.21324861427372963,   -0.532909328559444 ] x [U - 128]
// [B]   [1.1643835616438356,  2.1124017857142854,     0.0               ]   [V - 128]
//
// For the range [0,1] instead of [0,255]:
//
// The matrix is stored in column-major.
const mat3 YuvColorMatrixRec709 = mat3(
    1.16438,  1.16438,  1.16438,
    0.0    , -0.21325,  2.11240,
    1.79274, -0.53291,  0.0
);

// From Re2020:
// [R]   [1.16438356164384,  0.0,                    1.678674107142860 ]   [Y -  16]
// [G] = [1.16438356164384, -0.187326104219343,     -0.650424318505057 ] x [U - 128]
// [B]   [1.16438356164384,  2.14177232142857,       0.0               ]   [V - 128]
//
// For the range [0,1] instead of [0,255]:
//
// The matrix is stored in column-major.
const mat3 YuvColorMatrixRec2020 = mat3(
    1.16438356164384 ,  1.164383561643840,  1.16438356164384,
    0.0              , -0.187326104219343,  2.14177232142857,
    1.67867410714286 , -0.650424318505057,  0.0
);

mat3 get_yuv_color_matrix(int color_space) {
    switch (color_space) {
        case YUV_COLOR_SPACE_REC601:
            return YuvColorMatrixRec601;
        case YUV_COLOR_SPACE_REC709:
            return YuvColorMatrixRec709;
        default:
            return YuvColorMatrixRec2020;
    }
}

void write_uv_rect(
    vec2 uv0,
    vec2 uv1,
    vec2 f,
    vec2 texture_size,
    out vec2 uv,
    out vec4 uv_bounds
) {
    uv = mix(uv0, uv1, f);

    uv_bounds = vec4(uv0 + vec2(0.5), uv1 - vec2(0.5));

    #ifndef WR_FEATURE_TEXTURE_RECT
        uv /= texture_size;
        uv_bounds /= texture_size.xyxy;
    #endif
}
#endif

#ifdef WR_FRAGMENT_SHADER

vec4 sample_yuv(
    int format,
    mat3 yuv_color_matrix,
    float coefficient,
    vec3 yuv_layers,
    vec2 in_uv_y,
    vec2 in_uv_u,
    vec2 in_uv_v,
    vec4 uv_bounds_y,
    vec4 uv_bounds_u,
    vec4 uv_bounds_v
) {
    vec3 yuv_value;

    switch (format) {
        case YUV_FORMAT_PLANAR:
            {
                // The yuv_planar format should have this third texture coordinate.
                vec2 uv_y = clamp(in_uv_y, uv_bounds_y.xy, uv_bounds_y.zw);
                vec2 uv_u = clamp(in_uv_u, uv_bounds_u.xy, uv_bounds_u.zw);
                vec2 uv_v = clamp(in_uv_v, uv_bounds_v.xy, uv_bounds_v.zw);
                yuv_value.x = TEX_SAMPLE(sColor0, vec3(uv_y, yuv_layers.x)).r;
                yuv_value.y = TEX_SAMPLE(sColor1, vec3(uv_u, yuv_layers.y)).r;
                yuv_value.z = TEX_SAMPLE(sColor2, vec3(uv_v, yuv_layers.z)).r;
            }
            break;

        case YUV_FORMAT_NV12:
            {
                vec2 uv_y = clamp(in_uv_y, uv_bounds_y.xy, uv_bounds_y.zw);
                vec2 uv_uv = clamp(in_uv_u, uv_bounds_u.xy, uv_bounds_u.zw);
                yuv_value.x = TEX_SAMPLE(sColor0, vec3(uv_y, yuv_layers.x)).r;
                yuv_value.yz = TEX_SAMPLE(sColor1, vec3(uv_uv, yuv_layers.y)).rg;
            }
            break;

        case YUV_FORMAT_INTERLEAVED:
            {
                // "The Y, Cb and Cr color channels within the 422 data are mapped into
                // the existing green, blue and red color channels."
                // https://www.khronos.org/registry/OpenGL/extensions/APPLE/APPLE_rgb_422.txt
                vec2 uv_y = clamp(in_uv_y, uv_bounds_y.xy, uv_bounds_y.zw);
                yuv_value = TEX_SAMPLE(sColor0, vec3(uv_y, yuv_layers.x)).gbr;
            }
            break;

        default:
            yuv_value = vec3(0.0);
            break;
    }

    // See the YuvColorMatrix definition for an explanation of where the constants come from.
    vec3 rgb = yuv_color_matrix * (yuv_value * coefficient - vec3(0.06275, 0.50196, 0.50196));
    vec4 color = vec4(rgb, 1.0);

    return color;
}
#endif
