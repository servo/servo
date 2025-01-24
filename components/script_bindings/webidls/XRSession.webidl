/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBXR

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

enum XRInteractionMode {
  "screen-space",
  "world-space",
};

callback XRFrameRequestCallback = undefined (DOMHighResTimeStamp time, XRFrame frame);

[SecureContext, Exposed=Window, Pref="dom_webxr_enabled"]
interface XRSession : EventTarget {
  // Attributes
  readonly attribute XRVisibilityState visibilityState;
  readonly attribute float? frameRate;
  readonly attribute Float32Array? supportedFrameRates;
  [SameObject] readonly attribute XRRenderState renderState;
  [SameObject] readonly attribute XRInputSourceArray inputSources;
  readonly attribute /*FrozenArray<DOMString>*/ any enabledFeatures;
  readonly attribute boolean isSystemKeyboardSupported;

  // Methods
  [Throws] undefined updateRenderState(optional XRRenderStateInit state = {});
  Promise<undefined> updateTargetFrameRate(float rate);
  Promise<XRReferenceSpace> requestReferenceSpace(XRReferenceSpaceType type);

  long requestAnimationFrame(XRFrameRequestCallback callback);
  undefined cancelAnimationFrame(long handle);

  Promise<undefined> end();

  // Events
  attribute EventHandler onend;
  attribute EventHandler onselect;
  attribute EventHandler onsqueeze;
  attribute EventHandler oninputsourceschange;
  attribute EventHandler onselectstart;
  attribute EventHandler onselectend;
  attribute EventHandler onsqueezestart;
  attribute EventHandler onsqueezeend;
  attribute EventHandler onvisibilitychange;
  attribute EventHandler onframeratechange;

  // AR Module
  // Attributes
  readonly attribute XREnvironmentBlendMode environmentBlendMode;
  readonly attribute XRInteractionMode interactionMode;

  // Hit Test Module
  // Methods
  Promise<XRHitTestSource> requestHitTestSource(XRHitTestOptionsInit options);
};
