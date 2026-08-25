/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBXR

// https://immersive-web.github.io/layers/#xrmediabindingtype
[SecureContext, Exposed=Window, Pref="dom_webxr_layers_enabled"]
interface XRMediaBinding {
  [Throws] constructor(XRSession session);

  [Throws] XRQuadLayer createQuadLayer(HTMLVideoElement video, XRMediaLayerInit init);
  [Throws] XRCylinderLayer createCylinderLayer(HTMLVideoElement video, XRMediaLayerInit init);
  [Throws] XREquirectLayer createEquirectLayer(HTMLVideoElement video, XRMediaLayerInit init);
};

dictionary XRMediaLayerInit {
  required XRSpace space;
  XRLayerLayout layout = "mono";
  boolean invertStereo = false;
};
