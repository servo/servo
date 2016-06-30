/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#version 110

#define SERVO_ES2

uniform mat4 uTransform;
uniform vec4 uOffsets[32];
uniform vec4 uClipRects[64];
uniform mat4 uMatrixPalette[32];
uniform vec2 uDirection;
uniform vec4 uBlendParams;
uniform vec4 uFilterParams;
uniform float uDevicePixelRatio;
uniform vec4 uTileParams[64];

attribute vec3 aPosition;
attribute vec4 aPositionRect;  // Width can be negative to flip horizontally (for border corners).
attribute vec4 aColorRectTL;
attribute vec4 aColorRectTR;
attribute vec4 aColorRectBR;
attribute vec4 aColorRectBL;
attribute vec4 aColorTexCoordRectTop;
attribute vec4 aColorTexCoordRectBottom;
attribute vec4 aMaskTexCoordRectTop;
attribute vec4 aMaskTexCoordRectBottom;
attribute vec4 aBorderPosition;
attribute vec4 aBorderRadii;
attribute vec2 aSourceTextureSize;
attribute vec2 aDestTextureSize;
attribute float aBlurRadius;
// x = matrix index; y = clip-in rect; z = clip-out rect; w = tile params index.
//
// A negative w value activates border corner mode. In this mode, the TR and BL colors are ignored,
// the color of the top left corner applies to all vertices of the top left triangle, and the color
// of the bottom right corner applies to all vertices of the bottom right triangle.
attribute vec4 aMisc;

varying vec2 vPosition;
varying vec4 vColor;
varying vec2 vColorTexCoord;
varying vec2 vMaskTexCoord;
varying vec4 vBorderPosition;
varying vec4 vBorderRadii;
varying vec2 vDestTextureSize;
varying vec2 vSourceTextureSize;
varying float vBlurRadius;
varying vec4 vTileParams;
varying vec4 vClipInRect;
varying vec4 vClipOutRect;

int Bottom7Bits(int value) {
    return value % 0x80;
}

bool IsBottomTriangle() {
    // FIXME(pcwalton): No gl_VertexID in OpenGL ES 2. We'll need some extra data.
    return false;
}

vec2 SnapToPixels(vec2 pos)
{
    // Snap the vertex to pixel position to guarantee correct texture
    // sampling when using bilinear filtering.

    // TODO(gw): Do we ever get negative coords here?
    return floor(0.5 + pos * uDevicePixelRatio) / uDevicePixelRatio;
}
