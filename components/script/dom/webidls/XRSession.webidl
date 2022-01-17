/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrsession-interface

enum XREnvironmentBlendMode {
  "opaque",
  "additive",
  "alpha-blend",
};

enum XRVisibilityState {
  "visible",
  "visible-blurred",
  "hidden",
};

callback XRFrameRequestCallback = undefined (DOMHighResTimeStamp time, XRFrame frame);

[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
interface XRSession : EventTarget {
  // // Attributes
  readonly attribute XREnvironmentBlendMode environmentBlendMode;

  readonly attribute XRVisibilityState visibilityState;
  [SameObject] readonly attribute XRRenderState renderState;
  [SameObject] readonly attribute XRInputSourceArray inputSources;

  // // Methods
  [Throws] undefined updateRenderState(optional XRRenderStateInit state = {});
  Promise<XRReferenceSpace> requestReferenceSpace(XRReferenceSpaceType type);

  long requestAnimationFrame(XRFrameRequestCallback callback);
  undefined cancelAnimationFrame(long handle);

  Promise<undefined> end();

  // hit test module
  Promise<XRHitTestSource> requestHitTestSource(XRHitTestOptionsInit options);

  // // Events
  attribute EventHandler onend;
  attribute EventHandler onselect;
  attribute EventHandler onsqueeze;
  attribute EventHandler oninputsourceschange;
  attribute EventHandler onselectstart;
  attribute EventHandler onselectend;
  attribute EventHandler onsqueezestart;
  attribute EventHandler onsqueezeend;
  attribute EventHandler onvisibilitychange;
};
