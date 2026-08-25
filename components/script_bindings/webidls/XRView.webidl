/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBXR

// https://immersive-web.github.io/webxr/#xrview-interface

enum XREye {
  "left",
  "right",
  "none",
};

[SecureContext, Exposed=Window, Pref="dom_webxr_enabled"]
interface XRView {
  readonly attribute XREye eye;
  readonly attribute Float32Array projectionMatrix;
  [SameObject] readonly attribute XRRigidTransform transform;
  readonly attribute double? recommendedViewportScale;

  undefined requestViewportScale(double? scale);

  // AR Module
  readonly attribute boolean isFirstPersonObserver;
};
