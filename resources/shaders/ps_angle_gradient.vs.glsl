#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    Primitive prim = load_primitive(gl_InstanceID);
    Gradient gradient = fetch_gradient(prim.prim_index);

    VertexInfo vi = write_vertex(prim.local_rect,
                                 prim.local_clip_rect,
                                 prim.layer,
                                 prim.tile);

    vStopCount = int(prim.user_data.x);
    vPos = vi.local_clamped_pos;

    // Snap the start/end points to device pixel units.
    // I'm not sure this is entirely correct, but the
    // old render path does this, and it is needed to
    // make the angle gradient ref tests pass. It might
    // be better to fix this higher up in DL construction
    // and not snap here?
    vStartPoint = floor(0.5 + gradient.start_end_point.xy * uDevicePixelRatio) / uDevicePixelRatio;
    vEndPoint = floor(0.5 + gradient.start_end_point.zw * uDevicePixelRatio) / uDevicePixelRatio;

    for (int i=0 ; i < vStopCount ; ++i) {
        GradientStop stop = fetch_gradient_stop(prim.sub_index + i);
        vColors[i] = stop.color;
        vOffsets[i/4][i%4] = stop.offset.x;
    }
}
