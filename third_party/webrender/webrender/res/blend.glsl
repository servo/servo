/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define COMPONENT_TRANSFER_IDENTITY 0
#define COMPONENT_TRANSFER_TABLE 1
#define COMPONENT_TRANSFER_DISCRETE 2
#define COMPONENT_TRANSFER_LINEAR 3
#define COMPONENT_TRANSFER_GAMMA 4

// Must be kept in sync with `Filter::as_int` in internal_types.rs
// Not all filters are defined here because some filter use different shaders.
#define FILTER_CONTRAST            0
#define FILTER_GRAYSCALE           1
#define FILTER_HUE_ROTATE          2
#define FILTER_INVERT              3
#define FILTER_SATURATE            4
#define FILTER_SEPIA               5
#define FILTER_BRIGHTNESS          6
#define FILTER_COLOR_MATRIX        7
#define FILTER_SRGB_TO_LINEAR      8
#define FILTER_LINEAR_TO_SRGB      9
#define FILTER_FLOOD               10
#define FILTER_COMPONENT_TRANSFER  11

#ifdef WR_VERTEX_SHADER
void SetupFilterParams(
    int op,
    float amount,
    int gpu_data_address,
    out vec4 color_offset,
    out mat4 color_mat,
    out int table_address
) {
    float lumR = 0.2126;
    float lumG = 0.7152;
    float lumB = 0.0722;
    float oneMinusLumR = 1.0 - lumR;
    float oneMinusLumG = 1.0 - lumG;
    float oneMinusLumB = 1.0 - lumB;
    float invAmount = 1.0 - amount;

    if (op == FILTER_GRAYSCALE) {
        color_mat = mat4(
            vec4(lumR + oneMinusLumR * invAmount, lumR - lumR * invAmount, lumR - lumR * invAmount, 0.0),
            vec4(lumG - lumG * invAmount, lumG + oneMinusLumG * invAmount, lumG - lumG * invAmount, 0.0),
            vec4(lumB - lumB * invAmount, lumB - lumB * invAmount, lumB + oneMinusLumB * invAmount, 0.0),
            vec4(0.0, 0.0, 0.0, 1.0)
        );
        color_offset = vec4(0.0);
    } else if (op ==  FILTER_HUE_ROTATE) {
        float c = cos(amount);
        float s = sin(amount);
        color_mat = mat4(
            vec4(lumR + oneMinusLumR * c - lumR * s, lumR - lumR * c + 0.143 * s, lumR - lumR * c - oneMinusLumR * s, 0.0),
            vec4(lumG - lumG * c - lumG * s, lumG + oneMinusLumG * c + 0.140 * s, lumG - lumG * c + lumG * s, 0.0),
            vec4(lumB - lumB * c + oneMinusLumB * s, lumB - lumB * c - 0.283 * s, lumB + oneMinusLumB * c + lumB * s, 0.0),
            vec4(0.0, 0.0, 0.0, 1.0)
        );
        color_offset = vec4(0.0);
    } else if (op ==   FILTER_SATURATE) {
        color_mat = mat4(
            vec4(invAmount * lumR + amount, invAmount * lumR, invAmount * lumR, 0.0),
            vec4(invAmount * lumG, invAmount * lumG + amount, invAmount * lumG, 0.0),
            vec4(invAmount * lumB, invAmount * lumB, invAmount * lumB + amount, 0.0),
            vec4(0.0, 0.0, 0.0, 1.0)
        );
        color_offset = vec4(0.0);
    } else if (op == FILTER_SEPIA) {
        color_mat = mat4(
            vec4(0.393 + 0.607 * invAmount, 0.349 - 0.349 * invAmount, 0.272 - 0.272 * invAmount, 0.0),
            vec4(0.769 - 0.769 * invAmount, 0.686 + 0.314 * invAmount, 0.534 - 0.534 * invAmount, 0.0),
            vec4(0.189 - 0.189 * invAmount, 0.168 - 0.168 * invAmount, 0.131 + 0.869 * invAmount, 0.0),
            vec4(0.0, 0.0, 0.0, 1.0)
        );
        color_offset = vec4(0.0);
    } else if (op == FILTER_COLOR_MATRIX) {
        vec4 mat_data[4] = fetch_from_gpu_cache_4(gpu_data_address);
        vec4 offset_data = fetch_from_gpu_cache_1(gpu_data_address + 4);
        color_mat = mat4(mat_data[0], mat_data[1], mat_data[2], mat_data[3]);
        color_offset = offset_data;
    } else if (op == FILTER_COMPONENT_TRANSFER) {
        table_address = gpu_data_address;
    } else if (op == FILTER_FLOOD) {
        color_offset = fetch_from_gpu_cache_1(gpu_data_address);
    }
}
#endif

#ifdef WR_FRAGMENT_SHADER
vec3 Contrast(vec3 Cs, float amount) {
    return clamp(Cs.rgb * amount - 0.5 * amount + 0.5, 0.0, 1.0);
}

vec3 Invert(vec3 Cs, float amount) {
    return mix(Cs.rgb, vec3(1.0) - Cs.rgb, amount);
}

vec3 Brightness(vec3 Cs, float amount) {
    // Apply the brightness factor.
    // Resulting color needs to be clamped to output range
    // since we are pre-multiplying alpha in the shader.
    return clamp(Cs.rgb * amount, vec3(0.0), vec3(1.0));
}

// Based on the Gecko's implementation in
// https://hg.mozilla.org/mozilla-central/file/91b4c3687d75/gfx/src/FilterSupport.cpp#l24
// These could be made faster by sampling a lookup table stored in a float texture
// with linear interpolation.

vec3 SrgbToLinear(vec3 color) {
    vec3 c1 = color / 12.92;
    vec3 c2 = pow(color / 1.055 + vec3(0.055 / 1.055), vec3(2.4));
    return if_then_else(lessThanEqual(color, vec3(0.04045)), c1, c2);
}

vec3 LinearToSrgb(vec3 color) {
    vec3 c1 = color * 12.92;
    vec3 c2 = vec3(1.055) * pow(color, vec3(1.0 / 2.4)) - vec3(0.055);
    return if_then_else(lessThanEqual(color, vec3(0.0031308)), c1, c2);
}

// This function has to be factored out due to the following issue:
// https://github.com/servo/webrender/wiki/Driver-issues#bug-1532245---switch-statement-inside-control-flow-inside-switch-statement-fails-to-compile-on-some-android-phones
// (and now the words "default: default:" so angle_shader_validation.rs passes)
vec4 ComponentTransfer(vec4 colora, ivec4 vfuncs, int table_address) {
    // We push a different amount of data to the gpu cache depending on the
    // function type.
    // Identity => 0 blocks
    // Table/Discrete => 64 blocks (256 values)
    // Linear => 1 block (2 values)
    // Gamma => 1 block (3 values)
    // We loop through the color components and increment the offset (for the
    // next color component) into the gpu cache based on how many blocks that
    // function type put into the gpu cache.
    // Table/Discrete use a 256 entry look up table.
    // Linear/Gamma are a simple calculation.
    int offset = 0;
    vec4 texel;
    int k;

    // Dynamically indexing a vector is buggy on some platforms, so use a temporary array
    int[4] funcs = int[4](vfuncs.r, vfuncs.g, vfuncs.b, vfuncs.a);
    for (int i = 0; i < 4; i++) {
        switch (funcs[i]) {
            case COMPONENT_TRANSFER_IDENTITY:
                break;
            case COMPONENT_TRANSFER_TABLE:
            case COMPONENT_TRANSFER_DISCRETE: {
                // fetch value from lookup table
                k = int(floor(colora[i]*255.0));
                texel = fetch_from_gpu_cache_1(table_address + offset + k/4);
                colora[i] = clamp(texel[k % 4], 0.0, 1.0);
                // offset plus 256/4 blocks
                offset = offset + 64;
                break;
            }
            case COMPONENT_TRANSFER_LINEAR: {
                // fetch the two values for use in the linear equation
                texel = fetch_from_gpu_cache_1(table_address + offset);
                colora[i] = clamp(texel[0] * colora[i] + texel[1], 0.0, 1.0);
                // offset plus 1 block
                offset = offset + 1;
                break;
            }
            case COMPONENT_TRANSFER_GAMMA: {
                // fetch the three values for use in the gamma equation
                texel = fetch_from_gpu_cache_1(table_address + offset);
                colora[i] = clamp(texel[0] * pow(colora[i], texel[1]) + texel[2], 0.0, 1.0);
                // offset plus 1 block
                offset = offset + 1;
                break;
            }
            default:
                // shouldn't happen
                break;
        }
    }
    return colora;
}

void CalculateFilter(
    vec4 Cs,
    int op,
    float amount,
    int table_address,
    vec4 color_offset,
    mat4 color_mat,
    ivec4 v_funcs,
    out vec3 color,
    out float alpha
) {
    // Un-premultiply the input.
    alpha = Cs.a;
    color = alpha != 0.0 ? Cs.rgb / alpha : Cs.rgb;

    switch (op) {
        case FILTER_CONTRAST:
            color = Contrast(color, amount);
            break;
        case FILTER_INVERT:
            color = Invert(color, amount);
            break;
        case FILTER_BRIGHTNESS:
            color = Brightness(color, amount);
            break;
        case FILTER_SRGB_TO_LINEAR:
            color = SrgbToLinear(color);
            break;
        case FILTER_LINEAR_TO_SRGB:
            color = LinearToSrgb(color);
            break;
        case FILTER_COMPONENT_TRANSFER: {
            // Get the unpremultiplied color with alpha.
            vec4 colora = vec4(color, alpha);
            colora = ComponentTransfer(colora, v_funcs, table_address);
            color = colora.rgb;
            alpha = colora.a;
            break;
        }
        case FILTER_FLOOD:
            color = color_offset.rgb;
            alpha = color_offset.a;
            break;
        default:
            // Color matrix type filters (sepia, hue-rotate, etc...)
            vec4 result = color_mat * vec4(color, alpha) + color_offset;
            result = clamp(result, vec4(0.0), vec4(1.0));
            color = result.rgb;
            alpha = result.a;
    }
}
#endif
