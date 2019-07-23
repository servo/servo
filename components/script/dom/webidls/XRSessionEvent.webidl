/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrsessionevent-interface

[SecureContext, Exposed=Window, Pref="dom.webxr.enabled", Constructor
(DOMString type, XRSessionEventInit eventInitDict)]
interface XRSessionEvent : Event {
  [SameObject] readonly attribute XRSession session;
};

dictionary XRSessionEventInit : EventInit {
  required XRSession session;
};
