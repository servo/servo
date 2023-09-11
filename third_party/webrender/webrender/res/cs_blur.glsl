/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define WR_FEATURE_TEXTURE_2D

#include shared,prim_shared

varying vec2 vUv;
flat varying vec4 vUvRect;
flat varying vec2 vOffsetScale;
// The number of pixels on each end that we apply the blur filter over.
flat varying int vSupport;
flat varying vec2 vGaussCoefficients;

#ifdef WR_VERTEX_SHADER
// Applies a separable gaussian blur in one direction, as specified
// by the dir field in the blur command.

#define DIR_HORIZONTAL  0
#define DIR_VERTICAL    1

PER_INSTANCE in int aBlurRenderTaskAddress;
PER_INSTANCE in int aBlurSourceTaskAddress;
PER_INSTANCE in int aBlurDirection;

struct BlurTask {
    RectWithSize task_rect;
    float blur_radius;
    vec2 blur_region;
};

BlurTask fetch_blur_task(int address) {
    RenderTaskData task_data = fetch_render_task_data(address);

    BlurTask task = BlurTask(
        task_data.task_rect,
        task_data.user_data.x,
        task_data.user_data.yz
    );

    return task;
}

void calculate_gauss_coefficients(float sigma) {
    // Incremental Gaussian Coefficent Calculation (See GPU Gems 3 pp. 877 - 889)
    vGaussCoefficients = vec2(1.0 / (sqrt(2.0 * 3.14159265) * sigma),
                              exp(-0.5 / (sigma * sigma)));

    // Pre-calculate the coefficient total in the vertex shader so that
    // we can avoid having to do it per-fragment and also avoid division
    // by zero in the degenerate case.
    vec3 gauss_coefficient = vec3(vGaussCoefficients,
                                  vGaussCoefficients.y * vGaussCoefficients.y);
    float gauss_coefficient_total = gauss_coefficient.x;
    for (int i = 1; i <= vSupport; i += 2) {
        gauss_coefficient.xy *= gauss_coefficient.yz;
        float gauss_coefficient_subtotal = gauss_coefficient.x;
        gauss_coefficient.xy *= gauss_coefficient.yz;
        gauss_coefficient_subtotal += gauss_coefficient.x;
        gauss_coefficient_total += 2.0 * gauss_coefficient_subtotal;
    }

    // Scale initial coefficient by total to avoid passing the total separately
    // to the fragment shader.
    vGaussCoefficients.x /= gauss_coefficient_total;
}

void main(void) {
    BlurTask blur_task = fetch_blur_task(aBlurRenderTaskAddress);
    RectWithSize src_rect = fetch_render_task_rect(aBlurSourceTaskAddress);

    RectWithSize target_rect = blur_task.task_rect;

    vec2 texture_size = vec2(TEX_SIZE(sColor0).xy);

    // Ensure that the support is an even number of pixels to simplify the
    // fragment shader logic.
    //
    // TODO(pcwalton): Actually make use of this fact and use the texture
    // hardware for linear filtering.
    vSupport = int(ceil(1.5 * blur_task.blur_radius)) * 2;

    if (vSupport > 0) {
        calculate_gauss_coefficients(blur_task.blur_radius);
    } else {
        // The gauss function gets NaNs when blur radius is zero.
        vGaussCoefficients = vec2(1.0, 1.0);
    }

    switch (aBlurDirection) {
        case DIR_HORIZONTAL:
            vOffsetScale = vec2(1.0 / texture_size.x, 0.0);
            break;
        case DIR_VERTICAL:
            vOffsetScale = vec2(0.0, 1.0 / texture_size.y);
            break;
        default:
            vOffsetScale = vec2(0.0);
    }

    vUvRect = vec4(src_rect.p0 + vec2(0.5),
                   src_rect.p0 + blur_task.blur_region - vec2(0.5));
    vUvRect /= texture_size.xyxy;

    vec2 pos = target_rect.p0 + target_rect.size * aPosition.xy;

    vec2 uv0 = src_rect.p0 / texture_size;
    vec2 uv1 = (src_rect.p0 + src_rect.size) / texture_size;
    vUv = mix(uv0, uv1, aPosition.xy);

    gl_Position = uTransform * vec4(pos, 0.0, 1.0);
}
#endif

#ifdef WR_FRAGMENT_SHADER

#if defined WR_FEATURE_COLOR_TARGET
#define SAMPLE_TYPE vec4
#define SAMPLE_TEXTURE(uv)  texture(sColor0, uv)
#else
#define SAMPLE_TYPE float
#define SAMPLE_TEXTURE(uv)  texture(sColor0, uv).r
#endif

// TODO(gw): Write a fast path blur that handles smaller blur radii
//           with a offset / weight uniform table and a constant
//           loop iteration count!

void main(void) {
    SAMPLE_TYPE original_color = SAMPLE_TEXTURE(vUv);

    // Incremental Gaussian Coefficent Calculation (See GPU Gems 3 pp. 877 - 889)
    vec3 gauss_coefficient = vec3(vGaussCoefficients,
                                  vGaussCoefficients.y * vGaussCoefficients.y);

    SAMPLE_TYPE avg_color = original_color * gauss_coefficient.x;

    // Evaluate two adjacent texels at a time. We can do this because, if c0
    // and c1 are colors of adjacent texels and k0 and k1 are arbitrary
    // factors, this formula:
    //
    //     k0 * c0 + k1 * c1          (Equation 1)
    //
    // is equivalent to:
    //
    //                                 k1
    //     (k0 + k1) * lerp(c0, c1, -------)
    //                              k0 + k1
    //
    // A texture lookup of adjacent texels evaluates this formula:
    //
    //     lerp(c0, c1, t)
    //
    // for some t. So we can let `t = k1/(k0 + k1)` and effectively evaluate
    // Equation 1 with a single texture lookup.

    for (int i = 1; i <= vSupport; i += 2) {
        gauss_coefficient.xy *= gauss_coefficient.yz;

        float gauss_coefficient_subtotal = gauss_coefficient.x;
        gauss_coefficient.xy *= gauss_coefficient.yz;
        gauss_coefficient_subtotal += gauss_coefficient.x;
        float gauss_ratio = gauss_coefficient.x / gauss_coefficient_subtotal;

        vec2 offset = vOffsetScale * (float(i) + gauss_ratio);

        vec2 st0 = max(vUv - offset, vUvRect.xy);
        vec2 st1 = min(vUv + offset, vUvRect.zw);
        avg_color += (SAMPLE_TEXTURE(st0) + SAMPLE_TEXTURE(st1)) *
                     gauss_coefficient_subtotal;
    }

    oFragColor = vec4(avg_color);
}

#ifdef SWGL_DRAW_SPAN
    #ifdef WR_FEATURE_COLOR_TARGET
void swgl_drawSpanRGBA8() {
    swgl_commitGaussianBlurRGBA8(sColor0, vUv, vUvRect, vOffsetScale.x != 0.0,
                                 vSupport, vGaussCoefficients);
}
    #else
void swgl_drawSpanR8() {
    swgl_commitGaussianBlurR8(sColor0, vUv, vUvRect, vOffsetScale.x != 0.0,
                              vSupport, vGaussCoefficients);
}
    #endif
#endif

#endif
