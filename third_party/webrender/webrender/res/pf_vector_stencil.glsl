/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared

#ifdef WR_VERTEX_SHADER

PER_INSTANCE in vec2 aFromPosition;
PER_INSTANCE in vec2 aCtrlPosition;
PER_INSTANCE in vec2 aToPosition;
PER_INSTANCE in vec2 aFromNormal;
PER_INSTANCE in vec2 aCtrlNormal;
PER_INSTANCE in vec2 aToNormal;
PER_INSTANCE in int aPathID;
PER_INSTANCE in int aPad;

out vec2 vFrom;
out vec2 vCtrl;
out vec2 vTo;

void main(void) {
    // Unpack.
    int pathID = int(aPathID);

    ivec2 pathAddress = ivec2(0.0, aPathID);
    mat2 transformLinear = mat2(TEXEL_FETCH(sColor1, pathAddress, 0, ivec2(0, 0)));
    vec2 transformTranslation = TEXEL_FETCH(sColor1, pathAddress, 0, ivec2(1, 0)).xy;

    vec4 miscInfo = TEXEL_FETCH(sColor1, pathAddress, 0, ivec2(2, 0));
    float rectHeight = miscInfo.y;
    vec2 emboldenAmount = miscInfo.zw * 0.5;

    // TODO(pcwalton): Hint positions.
    vec2 from = aFromPosition;
    vec2 ctrl = aCtrlPosition;
    vec2 to = aToPosition;

    // Embolden as necessary.
    from -= aFromNormal * emboldenAmount;
    ctrl -= aCtrlNormal * emboldenAmount;
    to -= aToNormal * emboldenAmount;

    // Perform the transform.
    from = transformLinear * from + transformTranslation;
    ctrl = transformLinear * ctrl + transformTranslation;
    to = transformLinear * to + transformTranslation;

    // Choose correct quadrant for rotation.
    vec2 corner = vec2(0.0, rectHeight) + transformTranslation;

    // Compute edge vectors. De Casteljau subdivide if necessary.
    // TODO(pcwalton): Actually do the two-pass rendering.

    // Compute position and dilate. If too thin, discard to avoid artefacts.
    vec2 position;
    if (abs(from.x - to.x) < 0.0001)
        position.x = 0.0;
    else if (aPosition.x < 0.5)
        position.x = floor(min(min(from.x, to.x), ctrl.x));
    else
        position.x = ceil(max(max(from.x, to.x), ctrl.x));
    if (aPosition.y < 0.5)
        position.y = floor(min(min(from.y, to.y), ctrl.y));
    else
        position.y = corner.y;

    // Compute final position and depth.
    vec4 clipPosition = uTransform * vec4(position, aPosition.z, 1.0);

    // Finish up.
    gl_Position = clipPosition;
    vFrom = from - position;
    vCtrl = ctrl - position;
    vTo = to - position;
}

#endif

#ifdef WR_FRAGMENT_SHADER

uniform sampler2D uAreaLUT;

in vec2 vFrom;
in vec2 vCtrl;
in vec2 vTo;

void main(void) {
    // Unpack.
    vec2 from = vFrom, ctrl = vCtrl, to = vTo;

    // Determine winding, and sort into a consistent order so we only need to find one root below.
    bool winding = from.x < to.x;
    vec2 left = winding ? from : to, right = winding ? to : from;
    vec2 v0 = ctrl - left, v1 = right - ctrl;

    // Shoot a vertical ray toward the curve.
    vec2 window = clamp(vec2(from.x, to.x), -0.5, 0.5);
    float offset = mix(window.x, window.y, 0.5) - left.x;
    float t = offset / (v0.x + sqrt(v1.x * offset - v0.x * (offset - v0.x)));

    // Compute position and derivative to form a line approximation.
    float y = mix(mix(left.y, ctrl.y, t), mix(ctrl.y, right.y, t), t);
    float d = mix(v0.y, v1.y, t) / mix(v0.x, v1.x, t);

    // Look up area under that line, and scale horizontally to the window size.
    float dX = window.x - window.y;
    oFragColor = vec4(texture(sColor0, vec2(y + 8.0, abs(d * dX)) / 16.0).r * dX);
}

#endif
