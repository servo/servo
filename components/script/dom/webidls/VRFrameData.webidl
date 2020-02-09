/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webvr/#interface-vrframedata

[Exposed=Window, Pref="dom.webvr.enabled"]
interface VRFrameData {
  [Throws] constructor();
  readonly attribute DOMHighResTimeStamp timestamp;
  readonly attribute Float32Array leftProjectionMatrix;
  readonly attribute Float32Array leftViewMatrix;
  readonly attribute Float32Array rightProjectionMatrix;
  readonly attribute Float32Array rightViewMatrix;
  readonly attribute VRPose pose;
};
