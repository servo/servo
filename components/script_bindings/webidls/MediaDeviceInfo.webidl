/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

 // https://w3c.github.io/mediacapture-main/#device-info

[Exposed=Window,
SecureContext, Pref="dom.webrtc.enabled"]
interface MediaDeviceInfo {
  readonly attribute DOMString deviceId;
  readonly attribute MediaDeviceKind kind;
  readonly attribute DOMString label;
  readonly attribute DOMString groupId;
  [Default] object toJSON();
};

enum MediaDeviceKind {
  "audioinput",
  "audiooutput",
  "videoinput"
};
