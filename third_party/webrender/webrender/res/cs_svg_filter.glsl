/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared,prim_shared

varying vec3 vInput1Uv;
varying vec3 vInput2Uv;
flat varying vec4 vInput1UvRect;
flat varying vec4 vInput2UvRect;
flat varying int vFilterInputCount;
flat varying int vFilterKind;
flat varying ivec4 vData;
flat varying vec4 vFilterData0;
flat varying vec4 vFilterData1;
flat varying float vFloat0;
flat varying mat4 vColorMat;
flat varying int vFuncs[4];

#define FILTER_BLEND                0
#define FILTER_FLOOD                1
#define FILTER_LINEAR_TO_SRGB       2
#define FILTER_SRGB_TO_LINEAR       3
#define FILTER_OPACITY              4
#define FILTER_COLOR_MATRIX         5
#define FILTER_DROP_SHADOW          6
#define FILTER_OFFSET               7
#define FILTER_COMPONENT_TRANSFER   8
#define FILTER_IDENTITY             9
#define FILTER_COMPOSITE            10

#define COMPOSITE_OVER       0
#define COMPOSITE_IN         1
#define COMPOSITE_OUT        2
#define COMPOSITE_ATOP       3
#define COMPOSITE_XOR        4
#define COMPOSITE_LIGHTER    5
#define COMPOSITE_ARITHMETIC 6

#ifdef WR_VERTEX_SHADER

PER_INSTANCE in int aFilterRenderTaskAddress;
PER_INSTANCE in int aFilterInput1TaskAddress;
PER_INSTANCE in int aFilterInput2TaskAddress;
PER_INSTANCE in int aFilterKind;
PER_INSTANCE in int aFilterInputCount;
PER_INSTANCE in int aFilterGenericInt;
PER_INSTANCE in ivec2 aFilterExtraDataAddress;

struct FilterTask {
    RenderTaskCommonData common_data;
    vec3 user_data;
};

FilterTask fetch_filter_task(int address) {
    RenderTaskData task_data = fetch_render_task_data(address);

    FilterTask task = FilterTask(
        task_data.common_data,
        task_data.user_data.xyz
    );

    return task;
}

vec4 compute_uv_rect(RenderTaskCommonData task, vec2 texture_size) {
    RectWithSize task_rect = task.task_rect;

    vec4 uvRect = vec4(task_rect.p0 + vec2(0.5),
                       task_rect.p0 + task_rect.size - vec2(0.5));
    uvRect /= texture_size.xyxy;
    return uvRect;
}

vec3 compute_uv(RenderTaskCommonData task, vec2 texture_size) {
    RectWithSize task_rect = task.task_rect;
    vec3 uv = vec3(0.0, 0.0, task.texture_layer_index);

    vec2 uv0 = task_rect.p0 / texture_size;
    vec2 uv1 = floor(task_rect.p0 + task_rect.size) / texture_size;
    uv.xy = mix(uv0, uv1, aPosition.xy);

    return uv;
}

void main(void) {
    FilterTask filter_task = fetch_filter_task(aFilterRenderTaskAddress);
    RectWithSize target_rect = filter_task.common_data.task_rect;

    vec2 pos = target_rect.p0 + target_rect.size * aPosition.xy;

    RenderTaskCommonData input_1_task;
    if (aFilterInputCount > 0) {
        vec2 texture_size = vec2(textureSize(sColor0, 0).xy);
        input_1_task = fetch_render_task_common_data(aFilterInput1TaskAddress);
        vInput1UvRect = compute_uv_rect(input_1_task, texture_size);
        vInput1Uv = compute_uv(input_1_task, texture_size);
    }

    RenderTaskCommonData input_2_task;
    if (aFilterInputCount > 1) {
        vec2 texture_size = vec2(textureSize(sColor1, 0).xy);
        input_2_task = fetch_render_task_common_data(aFilterInput2TaskAddress);
        vInput2UvRect = compute_uv_rect(input_2_task, texture_size);
        vInput2Uv = compute_uv(input_2_task, texture_size);
    }

    vFilterInputCount = aFilterInputCount;
    vFilterKind = aFilterKind;

    // This assignment is only used for component transfer filters but this
    // assignment has to be done here and not in the component transfer case
    // below because it doesn't get executed on Windows because of a suspected
    // miscompile of this shader on Windows. See
    // https://github.com/servo/webrender/wiki/Driver-issues#bug-1505871---assignment-to-varying-flat-arrays-inside-switch-statement-of-vertex-shader-suspected-miscompile-on-windows
    // default: just to satisfy angle_shader_validation.rs which needs one
    // default: for every switch, even in comments.
    vFuncs[0] = (aFilterGenericInt >> 12) & 0xf; // R
    vFuncs[1] = (aFilterGenericInt >> 8)  & 0xf; // G
    vFuncs[2] = (aFilterGenericInt >> 4)  & 0xf; // B
    vFuncs[3] = (aFilterGenericInt)       & 0xf; // A

    switch (aFilterKind) {
        case FILTER_BLEND:
            vData = ivec4(aFilterGenericInt, 0, 0, 0);
            break;
        case FILTER_FLOOD:
            vFilterData0 = fetch_from_gpu_cache_1_direct(aFilterExtraDataAddress);
            break;
        case FILTER_OPACITY:
            vFloat0 = filter_task.user_data.x;
            break;
        case FILTER_COLOR_MATRIX:
            vec4 mat_data[4] = fetch_from_gpu_cache_4_direct(aFilterExtraDataAddress);
            vColorMat = mat4(mat_data[0], mat_data[1], mat_data[2], mat_data[3]);
            vFilterData0 = fetch_from_gpu_cache_1_direct(aFilterExtraDataAddress + ivec2(4, 0));
            break;
        case FILTER_DROP_SHADOW:
            vFilterData0 = fetch_from_gpu_cache_1_direct(aFilterExtraDataAddress);
            break;
        case FILTER_OFFSET:
            vec2 texture_size = vec2(textureSize(sColor0, 0).xy);
            vFilterData0 = vec4(-filter_task.user_data.xy / texture_size, vec2(0.0));

            RectWithSize task_rect = input_1_task.task_rect;
            vec4 clipRect = vec4(task_rect.p0, task_rect.p0 + task_rect.size);
            clipRect /= texture_size.xyxy;
            vFilterData1 = clipRect;
            break;
        case FILTER_COMPONENT_TRANSFER:
            vData = ivec4(aFilterExtraDataAddress, 0, 0);
            break;
        case FILTER_COMPOSITE:
            vData = ivec4(aFilterGenericInt, 0, 0, 0);
            if (aFilterGenericInt == COMPOSITE_ARITHMETIC) {
              vFilterData0 = fetch_from_gpu_cache_1_direct(aFilterExtraDataAddress);
            }
            break;
        default:
            break;
    }

    gl_Position = uTransform * vec4(pos, 0.0, 1.0);
}
#endif

#ifdef WR_FRAGMENT_SHADER

#define COMPONENT_TRANSFER_IDENTITY 0
#define COMPONENT_TRANSFER_TABLE 1
#define COMPONENT_TRANSFER_DISCRETE 2
#define COMPONENT_TRANSFER_LINEAR 3
#define COMPONENT_TRANSFER_GAMMA 4

vec3 Multiply(vec3 Cb, vec3 Cs) {
    return Cb * Cs;
}

vec3 Screen(vec3 Cb, vec3 Cs) {
    return Cb + Cs - (Cb * Cs);
}

vec3 HardLight(vec3 Cb, vec3 Cs) {
    vec3 m = Multiply(Cb, 2.0 * Cs);
    vec3 s = Screen(Cb, 2.0 * Cs - 1.0);
    vec3 edge = vec3(0.5, 0.5, 0.5);
    return mix(m, s, step(edge, Cs));
}

// TODO: Worth doing with mix/step? Check GLSL output.
float ColorDodge(float Cb, float Cs) {
    if (Cb == 0.0)
        return 0.0;
    else if (Cs == 1.0)
        return 1.0;
    else
        return min(1.0, Cb / (1.0 - Cs));
}

// TODO: Worth doing with mix/step? Check GLSL output.
float ColorBurn(float Cb, float Cs) {
    if (Cb == 1.0)
        return 1.0;
    else if (Cs == 0.0)
        return 0.0;
    else
        return 1.0 - min(1.0, (1.0 - Cb) / Cs);
}

float SoftLight(float Cb, float Cs) {
    if (Cs <= 0.5) {
        return Cb - (1.0 - 2.0 * Cs) * Cb * (1.0 - Cb);
    } else {
        float D;

        if (Cb <= 0.25)
            D = ((16.0 * Cb - 12.0) * Cb + 4.0) * Cb;
        else
            D = sqrt(Cb);

        return Cb + (2.0 * Cs - 1.0) * (D - Cb);
    }
}

vec3 Difference(vec3 Cb, vec3 Cs) {
    return abs(Cb - Cs);
}

vec3 Exclusion(vec3 Cb, vec3 Cs) {
    return Cb + Cs - 2.0 * Cb * Cs;
}

// These functions below are taken from the spec.
// There's probably a much quicker way to implement
// them in GLSL...
float Sat(vec3 c) {
    return max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b));
}

float Lum(vec3 c) {
    vec3 f = vec3(0.3, 0.59, 0.11);
    return dot(c, f);
}

vec3 ClipColor(vec3 C) {
    float L = Lum(C);
    float n = min(C.r, min(C.g, C.b));
    float x = max(C.r, max(C.g, C.b));

    if (n < 0.0)
        C = L + (((C - L) * L) / (L - n));

    if (x > 1.0)
        C = L + (((C - L) * (1.0 - L)) / (x - L));

    return C;
}

vec3 SetLum(vec3 C, float l) {
    float d = l - Lum(C);
    return ClipColor(C + d);
}

void SetSatInner(inout float Cmin, inout float Cmid, inout float Cmax, float s) {
    if (Cmax > Cmin) {
        Cmid = (((Cmid - Cmin) * s) / (Cmax - Cmin));
        Cmax = s;
    } else {
        Cmid = 0.0;
        Cmax = 0.0;
    }
    Cmin = 0.0;
}

vec3 SetSat(vec3 C, float s) {
    if (C.r <= C.g) {
        if (C.g <= C.b) {
            SetSatInner(C.r, C.g, C.b, s);
        } else {
            if (C.r <= C.b) {
                SetSatInner(C.r, C.b, C.g, s);
            } else {
                SetSatInner(C.b, C.r, C.g, s);
            }
        }
    } else {
        if (C.r <= C.b) {
            SetSatInner(C.g, C.r, C.b, s);
        } else {
            if (C.g <= C.b) {
                SetSatInner(C.g, C.b, C.r, s);
            } else {
                SetSatInner(C.b, C.g, C.r, s);
            }
        }
    }
    return C;
}

vec3 Hue(vec3 Cb, vec3 Cs) {
    return SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb));
}

vec3 Saturation(vec3 Cb, vec3 Cs) {
    return SetLum(SetSat(Cb, Sat(Cs)), Lum(Cb));
}

vec3 Color(vec3 Cb, vec3 Cs) {
    return SetLum(Cs, Lum(Cb));
}

vec3 Luminosity(vec3 Cb, vec3 Cs) {
    return SetLum(Cb, Lum(Cs));
}

const int BlendMode_Normal      = 0;
const int BlendMode_Multiply    = 1;
const int BlendMode_Screen      = 2;
const int BlendMode_Overlay     = 3;
const int BlendMode_Darken      = 4;
const int BlendMode_Lighten     = 5;
const int BlendMode_ColorDodge  = 6;
const int BlendMode_ColorBurn   = 7;
const int BlendMode_HardLight   = 8;
const int BlendMode_SoftLight   = 9;
const int BlendMode_Difference  = 10;
const int BlendMode_Exclusion   = 11;
const int BlendMode_Hue         = 12;
const int BlendMode_Saturation  = 13;
const int BlendMode_Color       = 14;
const int BlendMode_Luminosity  = 15;

vec4 blend(vec4 Cs, vec4 Cb, int mode) {
    vec4 result = vec4(1.0, 0.0, 0.0, 1.0);

    switch (mode) {
        case BlendMode_Normal:
            result.rgb = Cs.rgb;
            break;
        case BlendMode_Multiply:
            result.rgb = Multiply(Cb.rgb, Cs.rgb);
            break;
        case BlendMode_Screen:
            result.rgb = Screen(Cb.rgb, Cs.rgb);
            break;
        case BlendMode_Overlay:
            // Overlay is inverse of Hardlight
            result.rgb = HardLight(Cs.rgb, Cb.rgb);
            break;
        case BlendMode_Darken:
            result.rgb = min(Cs.rgb, Cb.rgb);
            break;
        case BlendMode_Lighten:
            result.rgb = max(Cs.rgb, Cb.rgb);
            break;
        case BlendMode_ColorDodge:
            result.r = ColorDodge(Cb.r, Cs.r);
            result.g = ColorDodge(Cb.g, Cs.g);
            result.b = ColorDodge(Cb.b, Cs.b);
            break;
        case BlendMode_ColorBurn:
            result.r = ColorBurn(Cb.r, Cs.r);
            result.g = ColorBurn(Cb.g, Cs.g);
            result.b = ColorBurn(Cb.b, Cs.b);
            break;
        case BlendMode_HardLight:
            result.rgb = HardLight(Cb.rgb, Cs.rgb);
            break;
        case BlendMode_SoftLight:
            result.r = SoftLight(Cb.r, Cs.r);
            result.g = SoftLight(Cb.g, Cs.g);
            result.b = SoftLight(Cb.b, Cs.b);
            break;
        case BlendMode_Difference:
            result.rgb = Difference(Cb.rgb, Cs.rgb);
            break;
        case BlendMode_Exclusion:
            result.rgb = Exclusion(Cb.rgb, Cs.rgb);
            break;
        case BlendMode_Hue:
            result.rgb = Hue(Cb.rgb, Cs.rgb);
            break;
        case BlendMode_Saturation:
            result.rgb = Saturation(Cb.rgb, Cs.rgb);
            break;
        case BlendMode_Color:
            result.rgb = Color(Cb.rgb, Cs.rgb);
            break;
        case BlendMode_Luminosity:
            result.rgb = Luminosity(Cb.rgb, Cs.rgb);
            break;
        default: break;
    }
    vec3 rgb = (1.0 - Cb.a) * Cs.rgb + Cb.a * result.rgb;
    result = mix(vec4(Cb.rgb * Cb.a, Cb.a), vec4(rgb, 1.0), Cs.a);
    return result;
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
            case COMPONENT_TRANSFER_DISCRETE:
                // fetch value from lookup table
                k = int(floor(colora[i]*255.0));
                texel = fetch_from_gpu_cache_1_direct(vData.xy + ivec2(offset + k/4, 0));
                colora[i] = clamp(texel[k % 4], 0.0, 1.0);
                // offset plus 256/4 blocks
                offset = offset + 64;
                break;
            case COMPONENT_TRANSFER_LINEAR:
                // fetch the two values for use in the linear equation
                texel = fetch_from_gpu_cache_1_direct(vData.xy + ivec2(offset, 0));
                colora[i] = clamp(texel[0] * colora[i] + texel[1], 0.0, 1.0);
                // offset plus 1 block
                offset = offset + 1;
                break;
            case COMPONENT_TRANSFER_GAMMA:
                // fetch the three values for use in the gamma equation
                texel = fetch_from_gpu_cache_1_direct(vData.xy + ivec2(offset, 0));
                colora[i] = clamp(texel[0] * pow(colora[i], texel[1]) + texel[2], 0.0, 1.0);
                // offset plus 1 block
                offset = offset + 1;
                break;
            default:
                // shouldn't happen
                break;
        }
    }
    return colora;
}

// Composite Filter

vec4 composite(vec4 Cs, vec4 Cb, int mode) {
    vec4 Cr = vec4(0.0, 1.0, 0.0, 1.0);
    switch (mode) {
        case COMPOSITE_OVER:
            Cr.rgb = Cs.a * Cs.rgb + Cb.a * Cb.rgb * (1.0 - Cs.a);
            Cr.a = Cs.a + Cb.a * (1.0 - Cs.a);
            break;
        case COMPOSITE_IN:
            Cr.rgb = Cs.a * Cs.rgb * Cb.a;
            Cr.a = Cs.a * Cb.a;
            break;
        case COMPOSITE_OUT:
            Cr.rgb = Cs.a * Cs.rgb * (1.0 - Cb.a);
            Cr.a = Cs.a * (1.0 - Cb.a);
            break;
        case COMPOSITE_ATOP:
            Cr.rgb = Cs.a * Cs.rgb * Cb.a + Cb.a * Cb.rgb * (1.0 - Cs.a);
            Cr.a = Cs.a * Cb.a + Cb.a * (1.0 - Cs.a);
            break;
        case COMPOSITE_XOR:
            Cr.rgb = Cs.a * Cs.rgb * (1.0 - Cb.a) + Cb.a * Cb.rgb * (1.0 - Cs.a);
            Cr.a = Cs.a * (1.0 - Cb.a) + Cb.a * (1.0 - Cs.a);
            break;
        case COMPOSITE_LIGHTER:
            Cr.rgb = Cs.a * Cs.rgb + Cb.a * Cb.rgb;
            Cr.a = Cs.a + Cb.a;
            Cr = clamp(Cr, vec4(0.0), vec4(1.0));
            break;
        case COMPOSITE_ARITHMETIC:
            Cr = vec4(vFilterData0.x) * Cs * Cb + vec4(vFilterData0.y) * Cs + vec4(vFilterData0.z) * Cb + vec4(vFilterData0.w);
            Cr = clamp(Cr, vec4(0.0), vec4(1.0));
            break;
        default:
            break;
    }
    return Cr;
}

vec4 sampleInUvRect(sampler2DArray sampler, vec3 uv, vec4 uvRect) {
    vec2 clamped = clamp(uv.xy, uvRect.xy, uvRect.zw);
    return texture(sampler, vec3(clamped, uv.z));
}

void main(void) {
    vec4 Ca = vec4(0.0, 0.0, 0.0, 0.0);
    vec4 Cb = vec4(0.0, 0.0, 0.0, 0.0);
    if (vFilterInputCount > 0) {
        Ca = sampleInUvRect(sColor0, vInput1Uv, vInput1UvRect);
        if (Ca.a != 0.0) {
            Ca.rgb /= Ca.a;
        }
    }
    if (vFilterInputCount > 1) {
        Cb = sampleInUvRect(sColor1, vInput2Uv, vInput2UvRect);
        if (Cb.a != 0.0) {
            Cb.rgb /= Cb.a;
        }
    }

    vec4 result = vec4(1.0, 0.0, 0.0, 1.0);

    bool needsPremul = true;

    switch (vFilterKind) {
        case FILTER_BLEND:
            result = blend(Ca, Cb, vData.x);
            needsPremul = false;
            break;
        case FILTER_FLOOD:
            result = vFilterData0;
            needsPremul = false;
            break;
        case FILTER_LINEAR_TO_SRGB:
            result.rgb = LinearToSrgb(Ca.rgb);
            result.a = Ca.a;
            break;
        case FILTER_SRGB_TO_LINEAR:
            result.rgb = SrgbToLinear(Ca.rgb);
            result.a = Ca.a;
            break;
        case FILTER_OPACITY:
            result.rgb = Ca.rgb;
            result.a = Ca.a * vFloat0;
            break;
        case FILTER_COLOR_MATRIX:
            result = vColorMat * Ca + vFilterData0;
            result = clamp(result, vec4(0.0), vec4(1.0));
            break;
        case FILTER_DROP_SHADOW:
            vec4 shadow = vec4(vFilterData0.rgb, Cb.a * vFilterData0.a);
            // Normal blend + source-over coposite
            result = blend(Ca, shadow, BlendMode_Normal);
            needsPremul = false;
            break;
        case FILTER_OFFSET:
            vec2 offsetUv = vInput1Uv.xy + vFilterData0.xy;
            result = sampleInUvRect(sColor0, vec3(offsetUv, vInput1Uv.z), vInput1UvRect);
            result *= point_inside_rect(offsetUv, vFilterData1.xy, vFilterData1.zw);
            needsPremul = false;
            break;
        case FILTER_COMPONENT_TRANSFER:
            result = ComponentTransfer(Ca);
            break;
        case FILTER_IDENTITY:
            result = Ca;
            break;
        case FILTER_COMPOSITE:
            result = composite(Ca, Cb, vData.x);
            needsPremul = false;
        default:
            break;
    }

    if (needsPremul) {
        result.rgb *= result.a;
    }

    oFragColor = result;
}
#endif
