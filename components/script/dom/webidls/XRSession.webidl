/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrsession-interface

enum XREnvironmentBlendMode {
  "opaque",
  "additive",
  "alpha-blend",
};

callback XRFrameRequestCallback = void (DOMHighResTimeStamp time, XRFrame frame);

[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
interface XRSession : EventTarget {
  // // Attributes
  readonly attribute XRSessionMode mode;
  // readonly attribute XRPresentationContext outputContext;
  readonly attribute XREnvironmentBlendMode environmentBlendMode;

  readonly attribute XRRenderState renderState;

  // // Methods
  Promise<XRReferenceSpace> requestReferenceSpace(XRReferenceSpaceType type);

  // workaround until we have FrozenArray
  // see https://github.com/servo/servo/issues/10427#issuecomment-449593626
  // FrozenArray<XRInputSource> getInputSources();
  sequence<XRInputSource> getInputSources();

  Promise<void> updateRenderState(optional XRRenderStateInit state);
  long requestAnimationFrame(XRFrameRequestCallback callback);
  void cancelAnimationFrame(long handle);

  // Promise<void> end();

  // // Events
  // attribute EventHandler onblur;
  // attribute EventHandler onfocus;
  // attribute EventHandler onend;
  // attribute EventHandler onselect;
  // attribute EventHandler oninputsourceschange;
  // attribute EventHandler onselectstart;
  // attribute EventHandler onselectend;
};
