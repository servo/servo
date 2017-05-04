/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/gamepad/#gamepad-interface
[Pref="dom.gamepad.enabled"]
interface Gamepad {
    readonly attribute DOMString id;
    readonly attribute long index;
    readonly attribute boolean connected;
    readonly attribute DOMHighResTimeStamp timestamp;
    readonly attribute DOMString mapping;
    readonly attribute Float64Array axes;
    [SameObject] readonly attribute GamepadButtonList buttons;
};

// https://w3c.github.io/gamepad/extensions.html#dom-gamepad
partial interface Gamepad {
  readonly attribute DOMString hand;
  readonly attribute VRPose? pose;
};

// https://w3c.github.io/webvr/spec/1.1/#interface-gamepad
partial interface Gamepad {
  readonly attribute unsigned long displayId;
};
