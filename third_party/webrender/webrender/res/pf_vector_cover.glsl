/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared

#ifdef WR_VERTEX_SHADER

PER_INSTANCE in ivec4 aTargetRect;
PER_INSTANCE in ivec2 aStencilOrigin;
PER_INSTANCE in int aSubpixel;
PER_INSTANCE in int aPad;

out vec2 vStencilUV;
flat out int vSubpixel;

void main(void) {
    vec4 targetRect = vec4(aTargetRect);
    vec2 stencilOrigin = vec2(aStencilOrigin);

    vec2 targetOffset = mix(vec2(0.0), targetRect.zw, aPosition.xy);
    vec2 targetPosition = targetRect.xy + targetOffset;
    vec2 stencilOffset = targetOffset * vec2(aSubpixel == 0 ? 1.0 : 3.0, 1.0);
    vec2 stencilPosition = stencilOrigin + stencilOffset;

    gl_Position = uTransform * vec4(targetPosition, aPosition.z, 1.0);
    vStencilUV = stencilPosition;
    vSubpixel = aSubpixel;
}

#endif

#ifdef WR_FRAGMENT_SHADER

#define LCD_FILTER_FACTOR_0     (86.0 / 255.0)
#define LCD_FILTER_FACTOR_1     (77.0 / 255.0)
#define LCD_FILTER_FACTOR_2     (8.0  / 255.0)

in vec2 vStencilUV;
flat in int vSubpixel;

/// Applies a slight horizontal blur to reduce color fringing on LCD screens
/// when performing subpixel AA.
///
/// The algorithm should be identical to that of FreeType:
/// https://www.freetype.org/freetype2/docs/reference/ft2-lcd_filtering.html
float lcdFilter(float shadeL2, float shadeL1, float shade0, float shadeR1, float shadeR2) {
    return LCD_FILTER_FACTOR_2 * shadeL2 +
        LCD_FILTER_FACTOR_1 * shadeL1 +
        LCD_FILTER_FACTOR_0 * shade0 +
        LCD_FILTER_FACTOR_1 * shadeR1 +
        LCD_FILTER_FACTOR_2 * shadeR2;
}

void main(void) {
    ivec2 stencilUV = ivec2(vStencilUV);
    float shade0 = abs(TEXEL_FETCH(sColor0, stencilUV, 0, ivec2(0, 0)).r);

    if (vSubpixel == 0) {
        oFragColor = vec4(shade0);
        return;
    }

    vec3 shadeL = abs(vec3(TEXEL_FETCH(sColor0, stencilUV, 0, ivec2(-1, 0)).r,
                           TEXEL_FETCH(sColor0, stencilUV, 0, ivec2(-2, 0)).r,
                           TEXEL_FETCH(sColor0, stencilUV, 0, ivec2(-3, 0)).r));
    vec3 shadeR = abs(vec3(TEXEL_FETCH(sColor0, stencilUV, 0, ivec2(1, 0)).r,
                           TEXEL_FETCH(sColor0, stencilUV, 0, ivec2(2, 0)).r,
                           TEXEL_FETCH(sColor0, stencilUV, 0, ivec2(3, 0)).r));

    oFragColor = vec4(lcdFilter(shadeL.z, shadeL.y, shadeL.x, shade0,   shadeR.x),
                      lcdFilter(shadeL.y, shadeL.x, shade0,   shadeR.x, shadeR.y),
                      lcdFilter(shadeL.x, shade0,   shadeR.x, shadeR.y, shadeR.z),
                      1.0);
}

#endif
