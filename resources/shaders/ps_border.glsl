#line 1

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// These are not changing.
flat varying vec4 vVerticalColor;     // The vertical color, e.g. top/bottom
flat varying vec4 vHorizontalColor;   // The horizontal color e.g. left/right
flat varying vec4 vRadii;             // The border radius from CSS border-radius
flat varying vec4 vLocalRect; // The rect of the border (x, y, w, h) in local space.

// for corners, this is the beginning of the corner.
// For the lines, this is the top left of the line.
flat varying vec2 vRefPoint;
flat varying int vBorderStyle;
flat varying int vBorderPart; // Which part of the border we're drawing.

flat varying vec4 vPieceRect;

// These are in device space
#ifdef WR_FEATURE_TRANSFORM
varying vec3 vLocalPos;     // The clamped position in local space.
flat varying float vPieceRectHypotenuseLength;
#else
varying vec2 vLocalPos;     // The clamped position in local space.

// These two are interpolated
varying float vDistanceFromMixLine;  // This is the distance from the line where two colors
                                     // meet in border corners.
#endif
