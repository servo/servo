/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrreferencespace-interface

enum XRReferenceSpaceType {
  "stationary",
  "bounded",
  "unbounded"
};

dictionary XRReferenceSpaceOptions {
  required XRReferenceSpaceType type;
};

[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
interface XRReferenceSpace : XRSpace {
  // attribute XRRigidTransform originOffset;
  // attribute EventHandler onreset;
};
