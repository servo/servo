/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

flat varying HIGHP_FS_ADDRESS int v_gradient_address;

#if defined(PLATFORM_ANDROID) && !defined(SWGL)
// Work around Adreno 3xx driver bug. See the v_perspective comment in
// brush_image or bugs 1630356 and  for details.
flat varying vec2 v_gradient_repeat_vec;
#define v_gradient_repeat v_gradient_repeat_vec.x
#else
// Repetition along the gradient stops.
flat varying float v_gradient_repeat;
#endif

#ifdef WR_FRAGMENT_SHADER

#ifdef WR_FEATURE_DITHERING
vec4 dither(vec4 color) {
    const int matrix_mask = 7;

    ivec2 pos = ivec2(gl_FragCoord.xy) & ivec2(matrix_mask);
    float noise_normalized = (texelFetch(sDither, pos, 0).r * 255.0 + 0.5) / 64.0;
    float noise = (noise_normalized - 0.5) / 256.0; // scale down to the unit length

    return color + vec4(noise, noise, noise, 0);
}
#else
vec4 dither(vec4 color) {
    return color;
}
#endif //WR_FEATURE_DITHERING

#define GRADIENT_ENTRIES 128.0

float clamp_gradient_entry(float offset) {
    // Calculate the color entry index to use for this offset:
    //     offsets < 0 use the first color entry, 0
    //     offsets from [0, 1) use the color entries in the range of [1, N-1)
    //     offsets >= 1 use the last color entry, N-1
    //     so transform the range [0, 1) -> [1, N-1)

    // TODO(gw): In the future we might consider making the size of the
    // LUT vary based on number / distribution of stops in the gradient.
    // Ensure we don't fetch outside the valid range of the LUT.
    return clamp(1.0 + offset * GRADIENT_ENTRIES, 0.0, 1.0 + GRADIENT_ENTRIES);
}

vec4 sample_gradient(float offset) {
    // Modulo the offset if the gradient repeats.
    offset -= floor(offset) * v_gradient_repeat;

    // Calculate the texel to index into the gradient color entries:
    //     floor(x) is the gradient color entry index
    //     fract(x) is the linear filtering factor between start and end
    float x = clamp_gradient_entry(offset);
    float entry_index = floor(x);
    float entry_fract = x - entry_index;

    // Fetch the start and end color. There is a [start, end] color per entry.
    vec4 texels[2] = fetch_from_gpu_cache_2(v_gradient_address + 2 * int(entry_index));

    // Finally interpolate and apply dithering
    return dither(texels[0] + texels[1] * entry_fract);
}

#endif //WR_FRAGMENT_SHADER
