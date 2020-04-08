/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/hit-test/#xrray-interface

[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
interface XRRay {
  constructor(optional DOMPointInit origin = {}, optional DOMPointInit direction = {});
  constructor(XRRigidTransform transform);
  [SameObject] readonly attribute DOMPointReadOnly origin;
  [SameObject] readonly attribute DOMPointReadOnly direction;
  [SameObject] readonly attribute Float32Array matrix;
};
