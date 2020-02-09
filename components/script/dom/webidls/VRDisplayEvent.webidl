/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webvr/#interface-vrdisplayevent

enum VRDisplayEventReason {
  "navigation",
  "mounted",
  "unmounted",
  "requested"
};

[Exposed=Window, Pref="dom.webvr.enabled"]
interface VRDisplayEvent : Event {
  [Throws] constructor(DOMString type, VRDisplayEventInit eventInitDict);
  readonly attribute VRDisplay display;
  readonly attribute VRDisplayEventReason? reason;
};

dictionary VRDisplayEventInit : EventInit {
  required VRDisplay display;
  VRDisplayEventReason reason;
};
