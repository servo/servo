/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define SERVO_GL3

uniform mat4 uTransform;
uniform vec4 uOffsets[32];
uniform vec4 uClipRects[64];
uniform mat4 uMatrixPalette[32];
uniform vec2 uDirection;
uniform vec4 uBlendParams;
uniform vec4 uFilterParams;
uniform float uDevicePixelRatio;
uniform vec4 uTileParams[64];
uniform vec4 uAtlasParams;

in vec3 aPosition;
in vec4 aPositionRect;  // Width can be negative to flip horizontally (for border corners).
in vec4 aColorRectTL;
in vec4 aColorRectTR;
in vec4 aColorRectBR;
in vec4 aColorRectBL;
in vec4 aColorTexCoordRectTop;
in vec4 aColorTexCoordRectBottom;
in vec4 aMaskTexCoordRectTop;
in vec4 aMaskTexCoordRectBottom;
in vec4 aBorderPosition;
in vec4 aBorderRadii;
in vec2 aSourceTextureSize;
in vec2 aDestTextureSize;
in float aBlurRadius;
// x = matrix index; y = clip-in rect; z = clip-out rect; w = tile params index.
//
// A negative w value activates border corner mode. In this mode, the TR and BL colors are ignored,
// the color of the top left corner applies to all vertices of the top left triangle, and the color
// of the bottom right corner applies to all vertices of the bottom right triangle.
in vec4 aMisc;

out vec2 vPosition;
out vec4 vColor;
out vec2 vColorTexCoord;
out vec2 vMaskTexCoord;
out vec4 vBorderPosition;
out vec4 vBorderRadii;
out vec2 vDestTextureSize;
out vec2 vSourceTextureSize;
out float vBlurRadius;
out vec4 vTileParams;
out vec4 vClipInRect;
out vec4 vClipOutRect;

int Bottom7Bits(int value) {
    return value & 0x7f;
}

bool IsBottomTriangle() {
    return gl_VertexID > 2;
}

vec2 SnapToPixels(vec2 pos)
{
    // Snap the vertex to pixel position to guarantee correct texture
    // sampling when using bilinear filtering.

    // Don't use round() because its behavior is implementation-defined on 0.5.
    // TODO: Do we ever get negative coords here?
    return floor(0.5 + pos * uDevicePixelRatio) / uDevicePixelRatio;
}
