#line 1

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define COMPOSITE_KIND_MIX_BLEND_MODE   0
#define COMPOSITE_KIND_FILTER           1

uniform sampler2D sCache;

vec3 rgbToHsv(vec3 c) {
    float value = max(max(c.r, c.g), c.b);

    float chroma = value - min(min(c.r, c.g), c.b);
    if (chroma == 0.0) {
        return vec3(0.0);
    }
    float saturation = chroma / value;

    float hue;
    if (c.r == value)
        hue = (c.g - c.b) / chroma;
    else if (c.g == value)
        hue = 2.0 + (c.b - c.r) / chroma;
    else // if (c.b == value)
        hue = 4.0 + (c.r - c.g) / chroma;

    hue *= 1.0/6.0;
    if (hue < 0.0)
        hue += 1.0;
    return vec3(hue, saturation, value);
}

vec3 hsvToRgb(vec3 c) {
    if (c.s == 0.0) {
        return vec3(c.z);
    }

    float hue = c.x * 6.0;
    int sector = int(hue);
    float residualHue = hue - float(sector);

    vec3 pqt = c.z * vec3(1.0 - c.y, 1.0 - c.y * residualHue, 1.0 - c.y * (1.0 - residualHue));
    if (sector == 0)
        return vec3(c.z, pqt.z, pqt.x);
    if (sector == 1)
        return vec3(pqt.y, c.z, pqt.x);
    if (sector == 2)
        return vec3(pqt.x, c.z, pqt.z);
    if (sector == 3)
        return vec3(pqt.x, pqt.y, c.z);
    if (sector == 4)
        return vec3(pqt.z, pqt.x, c.z);
    return vec3(c.z, pqt.x, pqt.y);
}

float gauss(float x, float sigma) {
    if (sigma == 0.0)
        return 1.0;
    return (1.0 / sqrt(6.283185307179586 * sigma * sigma)) * exp(-(x * x) / (2.0 * sigma * sigma));
}

vec4 Blur(float radius, vec2 direction) {
    // TODO(gw): Support blur in WR2!
    return vec4(1, 1, 1, 1);
}

vec4 Contrast(vec4 Cs, float amount) {
    return vec4(Cs.rgb * amount - 0.5 * amount + 0.5, 1.0);
}

vec4 Grayscale(vec4 Cs, float amount) {
    float ia = 1.0 - amount;
    return mat4(vec4(0.2126 + 0.7874 * ia, 0.2126 - 0.2126 * ia, 0.2126 - 0.2126 * ia, 0.0),
                vec4(0.7152 - 0.7152 * ia, 0.7152 + 0.2848 * ia, 0.7152 - 0.7152 * ia, 0.0),
                vec4(0.0722 - 0.0722 * ia, 0.0722 - 0.0722 * ia, 0.0722 + 0.9278 * ia, 0.0),
                vec4(0.0, 0.0, 0.0, 1.0)) * Cs;
}

vec4 HueRotate(vec4 Cs, float amount) {
    vec3 CsHsv = rgbToHsv(Cs.rgb);
    CsHsv.x = mod(CsHsv.x + amount / 6.283185307179586, 1.0);
    return vec4(hsvToRgb(CsHsv), Cs.a);
}

vec4 Invert(vec4 Cs, float amount) {
    return mix(Cs, vec4(1.0, 1.0, 1.0, Cs.a) - vec4(Cs.rgb, 0.0), amount);
}

vec4 Saturate(vec4 Cs, float amount) {
    return vec4(hsvToRgb(min(vec3(1.0, amount, 1.0) * rgbToHsv(Cs.rgb), vec3(1.0))), Cs.a);
}

vec4 Sepia(vec4 Cs, float amount) {
    float ia = 1.0 - amount;
    return mat4(vec4(0.393 + 0.607 * ia, 0.349 - 0.349 * ia, 0.272 - 0.272 * ia, 0.0),
                vec4(0.769 - 0.769 * ia, 0.686 + 0.314 * ia, 0.534 - 0.534 * ia, 0.0),
                vec4(0.189 - 0.189 * ia, 0.168 - 0.168 * ia, 0.131 + 0.869 * ia, 0.0),
                vec4(0.0, 0.0, 0.0, 1.0)) * Cs;
}

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

void main(void) {
    vec4 Cs = texture(sCache, vUv1);
    vec4 Cb = texture(sCache, vUv0);

    // TODO(gw): This is a hack that's (probably) wrong.
    //           Instead of drawing the tile rect, draw the
    //           stacking context bounds instead?
    if (Cs.a == 0.0) {
        oFragColor = Cb;
        return;
    }

    int kind = vInfo.x;
    int op = vInfo.y;
    float amount = vAmount;

    // Return yellow if none of the branches match (shouldn't happen).
    vec4 result = vec4(1.0, 1.0, 0.0, 1.0);

    switch (kind) {
        case COMPOSITE_KIND_MIX_BLEND_MODE:
            if (op == 2) {
                result.rgb = Screen(Cb.rgb, Cs.rgb);
            } else if (op == 3) {
                result.rgb = HardLight(Cs.rgb, Cb.rgb);        // Overlay is inverse of Hardlight
            } else if (op == 6) {
                result.r = ColorDodge(Cb.r, Cs.r);
                result.g = ColorDodge(Cb.g, Cs.g);
                result.b = ColorDodge(Cb.b, Cs.b);
            } else if (op == 7) {
                result.r = ColorBurn(Cb.r, Cs.r);
                result.g = ColorBurn(Cb.g, Cs.g);
                result.b = ColorBurn(Cb.b, Cs.b);
            } else if (op == 8) {
                result.rgb = HardLight(Cb.rgb, Cs.rgb);
            } else if (op == 9) {
                result.r = SoftLight(Cb.r, Cs.r);
                result.g = SoftLight(Cb.g, Cs.g);
                result.b = SoftLight(Cb.b, Cs.b);
            } else if (op == 10) {
                result.rgb = Difference(Cb.rgb, Cs.rgb);
            } else if (op == 11) {
                result.rgb = Exclusion(Cb.rgb, Cs.rgb);
            } else if (op == 12) {
                result.rgb = Hue(Cb.rgb, Cs.rgb);
            } else if (op == 13) {
                result.rgb = Saturation(Cb.rgb, Cs.rgb);
            } else if (op == 14) {
                result.rgb = Color(Cb.rgb, Cs.rgb);
            } else if (op == 15) {
                result.rgb = Luminosity(Cb.rgb, Cs.rgb);
            }
            break;
        case COMPOSITE_KIND_FILTER:
            if (op == 0) {
                // Gaussian blur is specially handled:
                result = Cs;// Blur(amount, vec2(0,0));
            } else {
                if (op == 1) {
                    result = Contrast(Cs, amount);
                } else if (op == 2) {
                    result = Grayscale(Cs, amount);
                } else if (op == 3) {
                    result = HueRotate(Cs, amount);
                } else if (op == 4) {
                    result = Invert(Cs, amount);
                } else if (op == 5) {
                    result = Saturate(Cs, amount);
                } else if (op == 6) {
                    result = Sepia(Cs, amount);
                }
            }
            break;
    }

    oFragColor = result;
}
