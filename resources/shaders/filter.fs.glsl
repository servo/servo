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
#ifdef SERVO_ES2
    // TODO(gw): for loops have to be unrollable on es2.
    return vec4(1.0, 0.0, 0.0, 1.0);
#else
    int range = int(radius) * 3;
    float sigma = radius / 2.0;
    vec4 color = vec4(0.0);
    for (int offset = -range; offset <= range; offset++) {
        float offsetF = float(offset);

        // Here, we use the vMaskTexCoord.xy (i.e. the muv) to store the texture size.
        vec2 texCoord = vColorTexCoord.xy + vec2(offsetF) / vMaskTexCoord.xy * direction;
        vec4 x = texCoord.x >= 0.0 &&
            texCoord.x <= 1.0 &&
            texCoord.y >= 0.0 &&
            texCoord.y <= 1.0 ?
            Texture(sDiffuse, texCoord) :
            vec4(0.0);
        color += x * gauss(offsetF, sigma);
    }
    return color;
#endif
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

void main(void)
{
    // TODO: May be best to have separate shaders (esp. on Tegra)
    int filterOp = int(uFilterParams.x);
    float amount = uFilterParams.y;

    // Return yellow if none of the branches match (shouldn't happen).
    vec4 result = vec4(1.0, 1.0, 0.0, 1.0);

    if (filterOp == 0) {
        // Gaussian blur is specially handled:
        result = Blur(amount, uFilterParams.zw);
    } else {
        vec4 Cs = Texture(sDiffuse, vColorTexCoord);

        if (filterOp == 1) {
            result = Contrast(Cs, amount);
        } else if (filterOp == 2) {
            result = Grayscale(Cs, amount);
        } else if (filterOp == 3) {
            result = HueRotate(Cs, amount);
        } else if (filterOp == 4) {
            result = Invert(Cs, amount);
        } else if (filterOp == 5) {
            result = Saturate(Cs, amount);
        } else if (filterOp == 6) {
            result = Sepia(Cs, amount);
        }
    }

    SetFragColor(result);
}

