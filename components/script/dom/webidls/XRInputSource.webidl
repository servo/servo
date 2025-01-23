/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBXR

// https://immersive-web.github.io/webxr/#xrinputsource-interface

enum XRHandedness {
  "none",
  "left",
  "right"
};

enum XRTargetRayMode {
  "gaze",
  "tracked-pointer",
  "screen",
  "transient-pointer"
};

[SecureContext, Exposed=Window, Pref="dom_webxr_enabled"]
interface XRInputSource {
  readonly attribute XRHandedness handedness;
  readonly attribute XRTargetRayMode targetRayMode;
  [SameObject] readonly attribute XRSpace targetRaySpace;
  [SameObject] readonly attribute XRSpace? gripSpace;
  /* [SameObject] */ readonly attribute /* FrozenArray<DOMString> */ any profiles;
  readonly attribute boolean skipRendering;

  // WebXR Gamepads Module
  [SameObject] readonly attribute Gamepad? gamepad;

  // Hand Input
  [Pref="dom_webxr_hands_enabled"]
  readonly attribute XRHand? hand;
};
