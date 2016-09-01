/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

flat varying int vStopCount;
flat varying float vAngle;
flat varying vec2 vStartPoint;
flat varying vec2 vEndPoint;
varying vec2 vPos;
flat varying vec4 vColors[MAX_STOPS_PER_ANGLE_GRADIENT];
flat varying vec4 vOffsets[MAX_STOPS_PER_ANGLE_GRADIENT/4];
