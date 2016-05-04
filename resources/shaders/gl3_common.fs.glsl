/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define SERVO_GL3

precision highp float;

uniform sampler2D sDiffuse;
uniform sampler2D sMask;
uniform vec4 uBlendParams;
uniform vec2 uDirection;
uniform vec4 uFilterParams;

in vec2 vPosition;
in vec4 vColor;
in vec2 vColorTexCoord;
in vec2 vMaskTexCoord;
in vec4 vBorderPosition;
in vec4 vBorderRadii;
in vec2 vDestTextureSize;
in vec2 vSourceTextureSize;
in float vBlurRadius;
in vec4 vTileParams;
in vec4 vClipInRect;
in vec4 vClipOutRect;

out vec4 oFragColor;

vec4 Texture(sampler2D sampler, vec2 texCoord) {
    return texture(sampler, texCoord);
}

float GetAlphaFromMask(vec4 mask) {
    return mask.r;
}

void SetFragColor(vec4 color) {
    oFragColor = color;
}

