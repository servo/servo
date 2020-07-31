/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/layers/#xrmediabindingtype
[SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
interface XRMediaBinding {
  constructor(XRSession session);

//  XRQuadLayer createQuadVideoLayer(HTMLVideoElement video, optional XRMediaLayerInit init = {});
//  XRCylinderLayer createCylinderVideoLayer(HTMLVideoElement video, optional XRMediaLayerInit init = {});
//  XREquirectLayer createEquirectVideoLayer(HTMLVideoElement video, optional XRMediaLayerInit init = {});
};

dictionary XRMediaLayerInit {
  required XRSpace space;
  XRLayerLayout layout = "mono";
  boolean invertStereo = false;
};
