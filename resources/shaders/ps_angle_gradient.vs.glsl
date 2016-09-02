#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    AngleGradient gradient = fetch_angle_gradient(gl_InstanceID);
    VertexInfo vi = write_vertex(gradient.info);

    vStopCount = int(gradient.stop_count.x);
    vPos = vi.local_clamped_pos;

    // Snap the start/end points to device pixel units.
    // I'm not sure this is entirely correct, but the
    // old render path does this, and it is needed to
    // make the angle gradient ref tests pass. It might
    // be better to fix this higher up in DL construction
    // and not snap here?
    vStartPoint = floor(0.5 + gradient.start_end_point.xy * uDevicePixelRatio) / uDevicePixelRatio;
    vEndPoint = floor(0.5 + gradient.start_end_point.zw * uDevicePixelRatio) / uDevicePixelRatio;

    for (int i=0 ; i < int(gradient.stop_count.x) ; ++i) {
        vColors[i] = gradient.colors[i];
        vOffsets[i] = gradient.offsets[i];
    }
}
