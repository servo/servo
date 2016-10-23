#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

flat varying vec2 vTextureOffset; // Offset of this image into the texture atlas.
flat varying vec2 vTextureSize;   // Size of the image in the texture atlas.
flat varying vec2 vTileSpacing;   // Amount of space between tiled instances of this image.
flat varying vec2 vStretchSize;
flat varying vec4 vLocalRect;

#ifdef WR_FEATURE_TRANSFORM
varying vec3 vLocalPos;
#else
varying vec2 vLocalPos;
varying vec2 vUv;                 // Location within the CSS box to draw.
#endif
