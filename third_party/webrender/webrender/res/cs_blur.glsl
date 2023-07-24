/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared,prim_shared

varying vec3 vUv;
flat varying vec4 vUvRect;
flat varying vec2 vOffsetScale;
flat varying float vSigma;
// The number of pixels on each end that we apply the blur filter over.
flat varying int vSupport;

#ifdef WR_VERTEX_SHADER
// Applies a separable gaussian blur in one direction, as specified
// by the dir field in the blur command.

#define DIR_HORIZONTAL  0
#define DIR_VERTICAL    1

PER_INSTANCE in int aBlurRenderTaskAddress;
PER_INSTANCE in int aBlurSourceTaskAddress;
PER_INSTANCE in int aBlurDirection;

struct BlurTask {
    RenderTaskCommonData common_data;
    float blur_radius;
    vec2 blur_region;
};

BlurTask fetch_blur_task(int address) {
    RenderTaskData task_data = fetch_render_task_data(address);

    BlurTask task = BlurTask(
        task_data.common_data,
        task_data.user_data.x,
        task_data.user_data.yz
    );

    return task;
}

void main(void) {
    BlurTask blur_task = fetch_blur_task(aBlurRenderTaskAddress);
    RenderTaskCommonData src_task = fetch_render_task_common_data(aBlurSourceTaskAddress);

    RectWithSize src_rect = src_task.task_rect;
    RectWithSize target_rect = blur_task.common_data.task_rect;

#if defined WR_FEATURE_COLOR_TARGET
    vec2 texture_size = vec2(textureSize(sPrevPassColor, 0).xy);
#else
    vec2 texture_size = vec2(textureSize(sPrevPassAlpha, 0).xy);
#endif
    vUv.z = src_task.texture_layer_index;
    vSigma = blur_task.blur_radius;

    // Ensure that the support is an even number of pixels to simplify the
    // fragment shader logic.
    //
    // TODO(pcwalton): Actually make use of this fact and use the texture
    // hardware for linear filtering.
    vSupport = int(ceil(1.5 * blur_task.blur_radius)) * 2;

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
    vUv.xy = mix(uv0, uv1, aPosition.xy);

    gl_Position = uTransform * vec4(pos, 0.0, 1.0);
}
#endif

#ifdef WR_FRAGMENT_SHADER

#if defined WR_FEATURE_COLOR_TARGET
#define SAMPLE_TYPE vec4
#define SAMPLE_TEXTURE(uv)  texture(sPrevPassColor, uv)
#else
#define SAMPLE_TYPE float
#define SAMPLE_TEXTURE(uv)  texture(sPrevPassAlpha, uv).r
#endif

// TODO(gw): Write a fast path blur that handles smaller blur radii
//           with a offset / weight uniform table and a constant
//           loop iteration count!

void main(void) {
    SAMPLE_TYPE original_color = SAMPLE_TEXTURE(vUv);

    // TODO(gw): The gauss function gets NaNs when blur radius
    //           is zero. In the future, detect this earlier
    //           and skip the blur passes completely.
    if (vSupport == 0) {
        oFragColor = vec4(original_color);
        return;
    }

    // Incremental Gaussian Coefficent Calculation (See GPU Gems 3 pp. 877 - 889)
    vec3 gauss_coefficient;
    gauss_coefficient.x = 1.0 / (sqrt(2.0 * 3.14159265) * vSigma);
    gauss_coefficient.y = exp(-0.5 / (vSigma * vSigma));
    gauss_coefficient.z = gauss_coefficient.y * gauss_coefficient.y;

    float gauss_coefficient_total = gauss_coefficient.x;
    SAMPLE_TYPE avg_color = original_color * gauss_coefficient.x;
    gauss_coefficient.xy *= gauss_coefficient.yz;

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
        float gauss_coefficient_subtotal = gauss_coefficient.x;
        gauss_coefficient.xy *= gauss_coefficient.yz;
        gauss_coefficient_subtotal += gauss_coefficient.x;
        float gauss_ratio = gauss_coefficient.x / gauss_coefficient_subtotal;

        vec2 offset = vOffsetScale * (float(i) + gauss_ratio);

        vec2 st0 = clamp(vUv.xy - offset, vUvRect.xy, vUvRect.zw);
        avg_color += SAMPLE_TEXTURE(vec3(st0, vUv.z)) * gauss_coefficient_subtotal;

        vec2 st1 = clamp(vUv.xy + offset, vUvRect.xy, vUvRect.zw);
        avg_color += SAMPLE_TEXTURE(vec3(st1, vUv.z)) * gauss_coefficient_subtotal;

        gauss_coefficient_total += 2.0 * gauss_coefficient_subtotal;
        gauss_coefficient.xy *= gauss_coefficient.yz;
    }

    oFragColor = vec4(avg_color) / gauss_coefficient_total;
}
#endif
