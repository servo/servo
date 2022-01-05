/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/hit-test/#xrhittestsource-interface

enum XRHitTestTrackableType {
  "point",
  "plane",
  "mesh"
};

dictionary XRHitTestOptionsInit {
  required XRSpace space;
  sequence<XRHitTestTrackableType> entityTypes;
  XRRay offsetRay;
};

[SecureContext, Exposed=Window]
interface XRHitTestSource {
  undefined cancel();
};
