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
  Promise<XRReferenceSpace> requestReferenceSpace(XRReferenceSpaceOptions options);

  // FrozenArray<XRInputSource> getInputSources();

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

enum XRReferenceSpaceType {
  "identity",
  "stationary",
  "bounded",
  "unbounded"
};

dictionary XRReferenceSpaceOptions {
  required XRReferenceSpaceType type;
  XRStationaryReferenceSpaceSubtype subtype;
};
