/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

varying vec2 vUv;                 // Location within the CSS box to draw.
varying vec2 vPos;
flat varying vec2 vTextureOffset; // Offset of this image into the texture atlas.
flat varying vec2 vTextureSize;   // Size of the image in the texture atlas.
flat varying vec4 vClipRect;
flat varying vec4 vClipRadius;
