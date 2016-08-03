#line 1

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// These two are interpolated
varying float vF;   // This is a weighting as we get closer to the bottom right corner?

// These are not changing.
flat varying vec4 vVerticalColor;     // The vertical color, e.g. top/bottom
flat varying vec4 vHorizontalColor;   // The horizontal color e.g. left/right
flat varying vec4 vRadii;             // The border radius from CSS border-radius

// These are in device space
varying vec2 vLocalPos;     // The clamped position in local space.
varying vec2 vDevicePos;    // The clamped position in device space.
flat varying vec4 vBorders; // the rect of the border in (x, y, width, height) form

// for corners, this is the beginning of the corner.
// For the lines, this is the top left of the line.
flat varying vec2 vRefPoint;
flat varying uint vBorderStyle;
flat varying uint vBorderPart; // Which part of the border we're drawing.
