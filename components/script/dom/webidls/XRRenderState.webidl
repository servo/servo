/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrrenderstate-interface

dictionary XRRenderStateInit {
  double depthNear;
  double depthFar;
  XRLayer baseLayer;
};

[SecureContext, Exposed=Window] interface XRRenderState {
  readonly attribute double depthNear;
  readonly attribute double depthFar;
  readonly attribute XRLayer? baseLayer;
};
