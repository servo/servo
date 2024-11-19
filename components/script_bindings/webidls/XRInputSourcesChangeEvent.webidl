/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrinputsourceschangedevent-interface

[SecureContext, Exposed=Window, Pref="dom.webxr.test"]
interface XRInputSourcesChangeEvent : Event {
  constructor(DOMString type, XRInputSourcesChangeEventInit eventInitDict);
  [SameObject] readonly attribute XRSession session;
  /* [SameObject] */ readonly attribute /* FrozenArray<XRInputSource> */ any added;
  /* [SameObject] */ readonly attribute /* FrozenArray<XRInputSource> */ any removed;
};

dictionary XRInputSourcesChangeEventInit : EventInit {
  required XRSession session;
  required sequence<XRInputSource> added;
  required sequence<XRInputSource> removed;
};
