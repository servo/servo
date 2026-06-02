/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBXR

// https://immersive-web.github.io/webxr/#xrreferencespaceevent-interface

[SecureContext, Exposed=Window, Pref="dom_webxr_enabled"]
interface XRReferenceSpaceEvent : Event {
  [Throws] constructor(DOMString type, XRReferenceSpaceEventInit eventInitDict);
  [SameObject] readonly attribute XRReferenceSpace referenceSpace;
  [SameObject] readonly attribute XRRigidTransform? transform;
};

dictionary XRReferenceSpaceEventInit : EventInit {
  required XRReferenceSpace referenceSpace;
  XRRigidTransform? transform = null;
};
