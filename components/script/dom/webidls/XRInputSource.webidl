/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrinputsource-interface

enum XRHandedness {
  "none",
  "left",
  "right"
};

enum XRTargetRayMode {
  "gaze",
  "tracked-pointer",
  "screen"
};

[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
interface XRInputSource {
  readonly attribute XRHandedness handedness;
  readonly attribute XRTargetRayMode targetRayMode;
  [SameObject] readonly attribute XRSpace targetRaySpace;
  [SameObject] readonly attribute XRSpace? gripSpace;
  // [SameObject] readonly attribute Gamepad? gamepad;
  /* [SameObject] */ readonly attribute /* FrozenArray<DOMString> */ any profiles;

  [Pref="dom.webxr.hands.enabled"]
  readonly attribute XRHand? hand;
};
