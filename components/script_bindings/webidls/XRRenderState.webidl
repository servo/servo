/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrrenderstate-interface

dictionary XRRenderStateInit {
  double depthNear;
  double depthFar;
  double inlineVerticalFieldOfView;
  XRWebGLLayer? baseLayer;
  sequence<XRLayer>? layers;
};

[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"] interface XRRenderState {
  readonly attribute double depthNear;
  readonly attribute double depthFar;
  readonly attribute double? inlineVerticalFieldOfView;
  readonly attribute XRWebGLLayer? baseLayer;

  // https://immersive-web.github.io/layers/#xrrenderstatechanges
  // workaround until we have FrozenArray
  readonly attribute /* FrozenArray<XRLayer> */ any layers;
};
