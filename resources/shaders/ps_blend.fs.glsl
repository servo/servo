/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
    switch (sector) {
        case 0:
            return vec3(c.z, pqt.z, pqt.x);
        case 1:
            return vec3(pqt.y, c.z, pqt.x);
        case 2:
            return vec3(pqt.x, c.z, pqt.z);
        case 3:
            return vec3(pqt.x, pqt.y, c.z);
        case 4:
            return vec3(pqt.z, pqt.x, c.z);
        default:
            return vec3(c.z, pqt.x, pqt.y);
    }
}

vec4 Blur(float radius, vec2 direction) {
    // TODO(gw): Support blur in WR2!
    return vec4(1.0);
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

vec4 Brightness(vec4 Cs, float amount) {
    return vec4(Cs.rgb * amount, Cs.a);
}

vec4 Opacity(vec4 Cs, float amount) {
    return vec4(Cs.rgb, Cs.a * amount);
}

void main(void) {
    vec4 Cs = texture(sCache, vUv);

    if (Cs.a == 0.0) {
        discard;
    }

    switch (vOp) {
        case 0:
            // Gaussian blur is specially handled:
            oFragColor = Cs;// Blur(vAmount, vec2(0,0));
            break;
        case 1:
            oFragColor = Contrast(Cs, vAmount);
            break;
        case 2:
            oFragColor = Grayscale(Cs, vAmount);
            break;
        case 3:
            oFragColor = HueRotate(Cs, vAmount);
            break;
        case 4:
            oFragColor = Invert(Cs, vAmount);
            break;
        case 5:
            oFragColor = Saturate(Cs, vAmount);
            break;
        case 6:
            oFragColor = Sepia(Cs, vAmount);
            break;
        case 7:
            oFragColor = Brightness(Cs, vAmount);
            break;
        case 8:
            oFragColor = Opacity(Cs, vAmount);
            break;
    }
}
