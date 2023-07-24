/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define VECS_PER_MIX_BLEND_BRUSH 3
#define VECS_PER_SPECIFIC_BRUSH VECS_PER_MIX_BLEND_BRUSH

#define WR_BRUSH_VS_FUNCTION mix_blend_brush_vs
#define WR_BRUSH_FS_FUNCTION mix_blend_brush_fs

#include shared,prim_shared,brush

#define V_SRC_UV            varying_vec4_0.xy
#define V_SRC_LAYER         varying_vec4_0.w

#define V_BACKDROP_UV       varying_vec4_1.xy
#define V_BACKDROP_LAYER    varying_vec4_1.w

#define V_OP                flat_varying_ivec4_0.x

#ifdef WR_VERTEX_SHADER

void mix_blend_brush_vs(
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
    //Note: this is unsafe for `vi.world_pos.w <= 0.0`
    vec2 device_pos = vi.world_pos.xy * pic_task.device_pixel_scale / max(0.0, vi.world_pos.w);
    vec2 texture_size = vec2(textureSize(sPrevPassColor, 0));
    V_OP = prim_user_data.x;

    PictureTask src_task = fetch_picture_task(prim_user_data.z);
    vec2 src_device_pos = vi.world_pos.xy * (src_task.device_pixel_scale / max(0.0, vi.world_pos.w));
    vec2 src_uv = src_device_pos +
                  src_task.common_data.task_rect.p0 -
                  src_task.content_origin;
    V_SRC_UV = src_uv / texture_size;
    V_SRC_LAYER = src_task.common_data.texture_layer_index;

    RenderTaskCommonData backdrop_task = fetch_render_task_common_data(prim_user_data.y);
    float src_to_backdrop_scale = pic_task.device_pixel_scale / src_task.device_pixel_scale;
    vec2 backdrop_uv = device_pos +
                       backdrop_task.task_rect.p0 -
                       src_task.content_origin * src_to_backdrop_scale;
    V_BACKDROP_UV = backdrop_uv / texture_size;
    V_BACKDROP_LAYER = backdrop_task.texture_layer_index;
}
#endif

#ifdef WR_FRAGMENT_SHADER
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

const int MixBlendMode_Multiply    = 1;
const int MixBlendMode_Screen      = 2;
const int MixBlendMode_Overlay     = 3;
const int MixBlendMode_Darken      = 4;
const int MixBlendMode_Lighten     = 5;
const int MixBlendMode_ColorDodge  = 6;
const int MixBlendMode_ColorBurn   = 7;
const int MixBlendMode_HardLight   = 8;
const int MixBlendMode_SoftLight   = 9;
const int MixBlendMode_Difference  = 10;
const int MixBlendMode_Exclusion   = 11;
const int MixBlendMode_Hue         = 12;
const int MixBlendMode_Saturation  = 13;
const int MixBlendMode_Color       = 14;
const int MixBlendMode_Luminosity  = 15;

Fragment mix_blend_brush_fs() {
    vec4 Cb = textureLod(sPrevPassColor, vec3(V_BACKDROP_UV, V_BACKDROP_LAYER), 0.0);
    vec4 Cs = textureLod(sPrevPassColor, vec3(V_SRC_UV, V_SRC_LAYER), 0.0);

    // The mix-blend-mode functions assume no premultiplied alpha
    if (Cb.a != 0.0) {
        Cb.rgb /= Cb.a;
    }

    if (Cs.a != 0.0) {
        Cs.rgb /= Cs.a;
    }

    // Return yellow if none of the branches match (shouldn't happen).
    vec4 result = vec4(1.0, 1.0, 0.0, 1.0);

    switch (V_OP) {
        case MixBlendMode_Multiply:
            result.rgb = Multiply(Cb.rgb, Cs.rgb);
            break;
        case MixBlendMode_Screen:
            result.rgb = Screen(Cb.rgb, Cs.rgb);
            break;
        case MixBlendMode_Overlay:
            // Overlay is inverse of Hardlight
            result.rgb = HardLight(Cs.rgb, Cb.rgb);
            break;
        case MixBlendMode_Darken:
            result.rgb = min(Cs.rgb, Cb.rgb);
            break;
        case MixBlendMode_Lighten:
            result.rgb = max(Cs.rgb, Cb.rgb);
            break;
        case MixBlendMode_ColorDodge:
            result.r = ColorDodge(Cb.r, Cs.r);
            result.g = ColorDodge(Cb.g, Cs.g);
            result.b = ColorDodge(Cb.b, Cs.b);
            break;
        case MixBlendMode_ColorBurn:
            result.r = ColorBurn(Cb.r, Cs.r);
            result.g = ColorBurn(Cb.g, Cs.g);
            result.b = ColorBurn(Cb.b, Cs.b);
            break;
        case MixBlendMode_HardLight:
            result.rgb = HardLight(Cb.rgb, Cs.rgb);
            break;
        case MixBlendMode_SoftLight:
            result.r = SoftLight(Cb.r, Cs.r);
            result.g = SoftLight(Cb.g, Cs.g);
            result.b = SoftLight(Cb.b, Cs.b);
            break;
        case MixBlendMode_Difference:
            result.rgb = Difference(Cb.rgb, Cs.rgb);
            break;
        case MixBlendMode_Exclusion:
            result.rgb = Exclusion(Cb.rgb, Cs.rgb);
            break;
        case MixBlendMode_Hue:
            result.rgb = Hue(Cb.rgb, Cs.rgb);
            break;
        case MixBlendMode_Saturation:
            result.rgb = Saturation(Cb.rgb, Cs.rgb);
            break;
        case MixBlendMode_Color:
            result.rgb = Color(Cb.rgb, Cs.rgb);
            break;
        case MixBlendMode_Luminosity:
            result.rgb = Luminosity(Cb.rgb, Cs.rgb);
            break;
        default: break;
    }

    result.rgb = (1.0 - Cb.a) * Cs.rgb + Cb.a * result.rgb;
    result.a = Cs.a;

    result.rgb *= result.a;

    return Fragment(result);
}
#endif

// Undef macro names that could be re-defined by other shaders.
#undef V_SRC_UV
#undef V_SRC_LAYER
#undef V_BACKDROP_UV
#undef V_BACKDROP_LAYER
#undef V_OP
