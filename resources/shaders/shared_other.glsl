/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//======================================================================================
// Vertex shader attributes and uniforms
//======================================================================================
#ifdef WR_VERTEX_SHADER
    in vec4 aColorTexCoordRectTop;
    in vec4 aColorRectTL;

    // box-shadow
    in vec4 aBorderPosition;
    in vec4 aBorderRadii;
    in float aBlurRadius;

    // blur
    in vec2 aDestTextureSize;
    in vec2 aSourceTextureSize;
#endif

//======================================================================================
// Fragment shader attributes and uniforms
//======================================================================================
#ifdef WR_FRAGMENT_SHADER
    uniform vec2 uDirection;
#endif

//======================================================================================
// Interpolator definitions
//======================================================================================

// Hacks to be removed (needed for text etc)
varying vec2 vColorTexCoord;
varying vec4 vColor;

// box_shadow
varying vec2 vPosition;
varying vec4 vBorderPosition;
varying vec4 vBorderRadii;
varying float vBlurRadius;

// blur
varying vec2 vSourceTextureSize;
varying vec2 vDestTextureSize;

//======================================================================================
// VS only types and UBOs
//======================================================================================

//======================================================================================
// VS only functions
//======================================================================================

//======================================================================================
// FS only functions
//======================================================================================
#ifdef WR_FRAGMENT_SHADER

void SetFragColor(vec4 color) {
    oFragColor = color;
}

#endif
