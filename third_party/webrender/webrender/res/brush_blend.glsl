/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define VECS_PER_BLEND_BRUSH 3
#define VECS_PER_SPECIFIC_BRUSH VECS_PER_BLEND_BRUSH

#define WR_BRUSH_VS_FUNCTION blend_brush_vs
#define WR_BRUSH_FS_FUNCTION blend_brush_fs

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

#include shared,prim_shared,brush

// Interpolated UV coordinates to sample.
#define V_UV                varying_vec4_0.zw
#define V_LOCAL_POS         varying_vec4_0.xy

#define V_FLOOD_COLOR       flat_varying_vec4_1

// Normalized bounds of the source image in the texture.
#define V_UV_BOUNDS         flat_varying_vec4_2

#define V_COLOR_OFFSET      flat_varying_vec4_3

// Layer index to sample.
#define V_LAYER             flat_varying_vec4_4.x
// Flag to allow perspective interpolation of UV.
#define V_PERSPECTIVE       flat_varying_vec4_4.y

#define V_AMOUNT            flat_varying_vec4_4.z

#define V_OP                flat_varying_ivec4_0.x
#define V_TABLE_ADDRESS     flat_varying_ivec4_0.y

flat varying mat4 vColorMat;

flat varying int vFuncs[4];

#ifdef WR_VERTEX_SHADER

void blend_brush_vs(
    VertexInfo vi,
    int prim_address,
    RectWithSize local_rect,
    RectWithSize segment_rect,
    ivec4 prim_user_data,
    int specific_resource_address,
    mat4 transform,
    PictureTask pic_task,
    int brush_flags,
    vec4 unused
) {
    ImageResource res = fetch_image_resource(prim_user_data.x);
    vec2 uv0 = res.uv_rect.p0;
    vec2 uv1 = res.uv_rect.p1;

    vec2 texture_size = vec2(textureSize(sColor0, 0).xy);
    vec2 f = (vi.local_pos - local_rect.p0) / local_rect.size;
    f = get_image_quad_uv(prim_user_data.x, f);
    vec2 uv = mix(uv0, uv1, f);
    float perspective_interpolate = (brush_flags & BRUSH_FLAG_PERSPECTIVE_INTERPOLATION) != 0 ? 1.0 : 0.0;

    V_UV = uv / texture_size * mix(vi.world_pos.w, 1.0, perspective_interpolate);
    V_LAYER = res.layer;
    V_PERSPECTIVE = perspective_interpolate;

    // TODO: The image shader treats this differently: deflate the rect by half a pixel on each side and
    // clamp the uv in the frame shader. Does it make sense to do the same here?
    V_UV_BOUNDS = vec4(uv0, uv1) / texture_size.xyxy;
    V_LOCAL_POS = vi.local_pos;

    float lumR = 0.2126;
    float lumG = 0.7152;
    float lumB = 0.0722;
    float oneMinusLumR = 1.0 - lumR;
    float oneMinusLumG = 1.0 - lumG;
    float oneMinusLumB = 1.0 - lumB;

    float amount = float(prim_user_data.z) / 65536.0;
    float invAmount = 1.0 - amount;

    V_OP = prim_user_data.y & 0xffff;
    V_AMOUNT = amount;

    // This assignment is only used for component transfer filters but this
    // assignment has to be done here and not in the component transfer case
    // below because it doesn't get executed on Windows because of a suspected
    // miscompile of this shader on Windows. See
    // https://github.com/servo/webrender/wiki/Driver-issues#bug-1505871---assignment-to-varying-flat-arrays-inside-switch-statement-of-vertex-shader-suspected-miscompile-on-windows
    // default: just to satisfy angle_shader_validation.rs which needs one
    // default: for every switch, even in comments.
    vFuncs[0] = (prim_user_data.y >> 28) & 0xf; // R
    vFuncs[1] = (prim_user_data.y >> 24) & 0xf; // G
    vFuncs[2] = (prim_user_data.y >> 20) & 0xf; // B
    vFuncs[3] = (prim_user_data.y >> 16) & 0xf; // A

    switch (V_OP) {
        case FILTER_GRAYSCALE: {
            vColorMat = mat4(
                vec4(lumR + oneMinusLumR * invAmount, lumR - lumR * invAmount, lumR - lumR * invAmount, 0.0),
                vec4(lumG - lumG * invAmount, lumG + oneMinusLumG * invAmount, lumG - lumG * invAmount, 0.0),
                vec4(lumB - lumB * invAmount, lumB - lumB * invAmount, lumB + oneMinusLumB * invAmount, 0.0),
                vec4(0.0, 0.0, 0.0, 1.0)
            );
            V_COLOR_OFFSET = vec4(0.0);
            break;
        }
        case FILTER_HUE_ROTATE: {
            float c = cos(amount);
            float s = sin(amount);
            vColorMat = mat4(
                vec4(lumR + oneMinusLumR * c - lumR * s, lumR - lumR * c + 0.143 * s, lumR - lumR * c - oneMinusLumR * s, 0.0),
                vec4(lumG - lumG * c - lumG * s, lumG + oneMinusLumG * c + 0.140 * s, lumG - lumG * c + lumG * s, 0.0),
                vec4(lumB - lumB * c + oneMinusLumB * s, lumB - lumB * c - 0.283 * s, lumB + oneMinusLumB * c + lumB * s, 0.0),
                vec4(0.0, 0.0, 0.0, 1.0)
            );
            V_COLOR_OFFSET = vec4(0.0);
            break;
        }
        case FILTER_SATURATE: {
            vColorMat = mat4(
                vec4(invAmount * lumR + amount, invAmount * lumR, invAmount * lumR, 0.0),
                vec4(invAmount * lumG, invAmount * lumG + amount, invAmount * lumG, 0.0),
                vec4(invAmount * lumB, invAmount * lumB, invAmount * lumB + amount, 0.0),
                vec4(0.0, 0.0, 0.0, 1.0)
            );
            V_COLOR_OFFSET = vec4(0.0);
            break;
        }
        case FILTER_SEPIA: {
            vColorMat = mat4(
                vec4(0.393 + 0.607 * invAmount, 0.349 - 0.349 * invAmount, 0.272 - 0.272 * invAmount, 0.0),
                vec4(0.769 - 0.769 * invAmount, 0.686 + 0.314 * invAmount, 0.534 - 0.534 * invAmount, 0.0),
                vec4(0.189 - 0.189 * invAmount, 0.168 - 0.168 * invAmount, 0.131 + 0.869 * invAmount, 0.0),
                vec4(0.0, 0.0, 0.0, 1.0)
            );
            V_COLOR_OFFSET = vec4(0.0);
            break;
        }
        case FILTER_COLOR_MATRIX: {
            vec4 mat_data[4] = fetch_from_gpu_cache_4(prim_user_data.z);
            vec4 offset_data = fetch_from_gpu_cache_1(prim_user_data.z + 4);
            vColorMat = mat4(mat_data[0], mat_data[1], mat_data[2], mat_data[3]);
            V_COLOR_OFFSET = offset_data;
            break;
        }
        case FILTER_COMPONENT_TRANSFER: {
            V_TABLE_ADDRESS = prim_user_data.z;
            break;
        }
        case FILTER_FLOOD: {
            V_FLOOD_COLOR = fetch_from_gpu_cache_1(prim_user_data.z);
            break;
        }
        default: break;
    }
}
#endif

#ifdef WR_FRAGMENT_SHADER
vec3 Contrast(vec3 Cs, float amount) {
    return Cs.rgb * amount - 0.5 * amount + 0.5;
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
vec4 ComponentTransfer(vec4 colora) {
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

    for (int i = 0; i < 4; i++) {
        switch (vFuncs[i]) {
            case COMPONENT_TRANSFER_IDENTITY:
                break;
            case COMPONENT_TRANSFER_TABLE:
            case COMPONENT_TRANSFER_DISCRETE: {
                // fetch value from lookup table
                k = int(floor(colora[i]*255.0));
                texel = fetch_from_gpu_cache_1(V_TABLE_ADDRESS + offset + k/4);
                colora[i] = clamp(texel[k % 4], 0.0, 1.0);
                // offset plus 256/4 blocks
                offset = offset + 64;
                break;
            }
            case COMPONENT_TRANSFER_LINEAR: {
                // fetch the two values for use in the linear equation
                texel = fetch_from_gpu_cache_1(V_TABLE_ADDRESS + offset);
                colora[i] = clamp(texel[0] * colora[i] + texel[1], 0.0, 1.0);
                // offset plus 1 block
                offset = offset + 1;
                break;
            }
            case COMPONENT_TRANSFER_GAMMA: {
                // fetch the three values for use in the gamma equation
                texel = fetch_from_gpu_cache_1(V_TABLE_ADDRESS + offset);
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

Fragment blend_brush_fs() {
    float perspective_divisor = mix(gl_FragCoord.w, 1.0, V_PERSPECTIVE);
    vec2 uv = V_UV * perspective_divisor;
    vec4 Cs = texture(sColor0, vec3(uv, V_LAYER));

    // Un-premultiply the input.
    float alpha = Cs.a;
    vec3 color = alpha != 0.0 ? Cs.rgb / alpha : Cs.rgb;

    switch (V_OP) {
        case FILTER_CONTRAST:
            color = Contrast(color, V_AMOUNT);
            break;
        case FILTER_INVERT:
            color = Invert(color, V_AMOUNT);
            break;
        case FILTER_BRIGHTNESS:
            color = Brightness(color, V_AMOUNT);
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
            colora = ComponentTransfer(colora);
            color = colora.rgb;
            alpha = colora.a;
            break;
        }
        case FILTER_FLOOD:
            color = V_FLOOD_COLOR.rgb;
            alpha = V_FLOOD_COLOR.a;
            break;
        default:
            // Color matrix type filters (sepia, hue-rotate, etc...)
            vec4 result = vColorMat * vec4(color, alpha) + V_COLOR_OFFSET;
            result = clamp(result, vec4(0.0), vec4(1.0));
            color = result.rgb;
            alpha = result.a;
    }

    // Fail-safe to ensure that we don't sample outside the rendered
    // portion of a blend source.
    alpha *= min(point_inside_rect(uv, V_UV_BOUNDS.xy, V_UV_BOUNDS.zw),
                 init_transform_fs(V_LOCAL_POS));

    // Pre-multiply the alpha into the output value.
    return Fragment(alpha * vec4(color, 1.0));
}
#endif

// Undef macro names that could be re-defined by other shaders.
#undef V_UV
#undef V_LOCAL_POS
#undef V_FLOOD_COLOR
#undef V_UV_BOUNDS
#undef V_COLOR_OFFSET
#undef V_AMOUNT
#undef V_LAYER
#undef V_PERSPECTIVE
#undef V_OP
#undef V_TABLE_ADDRESS
