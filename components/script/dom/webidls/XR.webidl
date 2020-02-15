/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xr-interface
[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
interface XR: EventTarget {
  // Methods
  Promise<boolean> isSessionSupported(XRSessionMode mode);
  Promise<XRSession> requestSession(XRSessionMode mode, optional  XRSessionInit parameters = {});

  // Events
  // attribute EventHandler ondevicechange;
};

[SecureContext]
partial interface Navigator {
  [SameObject, Pref="dom.webxr.enabled"] readonly attribute XR xr;
};

enum XRSessionMode {
  "inline",
  "immersive-vr",
  "immersive-ar"
};

dictionary XRSessionInit {
  sequence<any> requiredFeatures;
  sequence<any> optionalFeatures;
};

partial interface XR {
  // https://github.com/immersive-web/webxr-test-api/
  [SameObject, Pref="dom.webxr.test"] readonly attribute XRTest test;
};
