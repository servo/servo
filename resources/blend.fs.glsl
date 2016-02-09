/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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

void main(void)
{
    vec3 Cs = Texture(sDiffuse, vColorTexCoord).xyz;
    vec3 Cb = Texture(sMask, vMaskTexCoord).xyz;

    // TODO: Relies on the ordering of MixBlendMode enum!
    // TODO: May be best to have separate shaders (esp. on Tegra)
    int blend_mode = int(uBlendParams.x);

    // Return yellow if none of the branches match (shouldn't happen).
    vec3 result = vec3(1.0, 1.0, 0.0);

    if (blend_mode == 2) {
        result = Screen(Cb, Cs);
    } else if (blend_mode == 3) {
        result = HardLight(Cs, Cb);        // Overlay is inverse of Hardlight
    } else if (blend_mode == 6) {
        result.r = ColorDodge(Cb.r, Cs.r);
        result.g = ColorDodge(Cb.g, Cs.g);
        result.b = ColorDodge(Cb.b, Cs.b);
    } else if (blend_mode == 7) {
        result.r = ColorBurn(Cb.r, Cs.r);
        result.g = ColorBurn(Cb.g, Cs.g);
        result.b = ColorBurn(Cb.b, Cs.b);
    } else if (blend_mode == 8) {
        result = HardLight(Cb, Cs);
    } else if (blend_mode == 9) {
        result.r = SoftLight(Cb.r, Cs.r);
        result.g = SoftLight(Cb.g, Cs.g);
        result.b = SoftLight(Cb.b, Cs.b);
    } else if (blend_mode == 10) {
        result = Difference(Cb, Cs);
    } else if (blend_mode == 11) {
        result = Exclusion(Cb, Cs);
    } else if (blend_mode == 12) {
        result = Hue(Cb, Cs);
    } else if (blend_mode == 13) {
        result = Saturation(Cb, Cs);
    } else if (blend_mode == 14) {
        result = Color(Cb, Cs);
    } else if (blend_mode == 15) {
        result = Luminosity(Cb, Cs);
    }

    // TODO: Handle output alpha correctly.
    SetFragColor(vec4(result, 1.0));
}

