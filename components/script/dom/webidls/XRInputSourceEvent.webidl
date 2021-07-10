/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrinputsourceevent-interface

[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
interface XRInputSourceEvent : Event {
  [Throws] constructor(DOMString type, XRInputSourceEventInit eventInitDict);
  [SameObject] readonly attribute XRFrame frame;
  [SameObject] readonly attribute XRInputSource inputSource;
};

dictionary XRInputSourceEventInit : EventInit {
  required XRFrame frame;
  required XRInputSource inputSource;
};
