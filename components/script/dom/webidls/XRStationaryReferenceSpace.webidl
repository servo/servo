/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrstationaryreferencespace-interface

enum XRStationaryReferenceSpaceSubtype {
  "eye-level",
  "floor-level",
  "position-disabled"
};

[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
interface XRStationaryReferenceSpace: XRReferenceSpace {
  // readonly attribute XRStationaryReferenceSpaceSubtype subtype;
};
