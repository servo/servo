/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// `vBorderPosition` is the position of the source texture in the atlas.

float gauss(float x, float sigma) {
    return (1.0 / sqrt(6.283185307179586 * sigma * sigma)) * exp(-(x * x) / (2.0 * sigma * sigma));
}

void main(void) {
#ifdef SERVO_ES2
    // TODO(gw): for loops have to be unrollable on es2.
    SetFragColor(vec4(1.0, 0.0, 0.0, 1.0));
#else
    vec2 sideOffsets = (vDestTextureSize - vSourceTextureSize) / 2.0;
    int range = int(vBlurRadius) * 3;
    float sigma = vBlurRadius / 2.0;
    vec4 value = vec4(0.0);
    vec2 sourceTextureUvOrigin = vBorderPosition.xy;
    vec2 sourceTextureUvSize = vBorderPosition.zw - sourceTextureUvOrigin;
    for (int offset = -range; offset <= range; offset++) {
        float offsetF = float(offset);
        vec2 lColorTexCoord = (vColorTexCoord.xy * vDestTextureSize - sideOffsets) /
            vSourceTextureSize;
        lColorTexCoord += vec2(offsetF) / vSourceTextureSize * uDirection;
        vec4 x = lColorTexCoord.x >= 0.0 &&
            lColorTexCoord.x <= 1.0 &&
            lColorTexCoord.y >= 0.0 &&
            lColorTexCoord.y <= 1.0 ?
            texture(sDiffuse, lColorTexCoord * sourceTextureUvSize + sourceTextureUvOrigin) :
            vec4(0.0);

        // Alpha must be premultiplied in order to properly blur the alpha channel.
        value += vec4(x.rgb * x.a, x.a) * gauss(offsetF, sigma);
    }

    // Unpremultiply the alpha.
    value = vec4(value.rgb / value.a, value.a);

    SetFragColor(value);
#endif
}

