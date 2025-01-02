/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/gamepad/#gamepad-interface
[Exposed=Window, Pref="dom.gamepad.enabled"]
interface Gamepad {
    readonly attribute DOMString id;
    readonly attribute long index;
    readonly attribute boolean connected;
    readonly attribute DOMHighResTimeStamp timestamp;
    readonly attribute DOMString mapping;
    readonly attribute Float64Array axes;
    [SameObject] readonly attribute GamepadButtonList buttons;
    [SameObject] readonly attribute GamepadHapticActuator vibrationActuator;
};

// https://w3c.github.io/gamepad/extensions.html#partial-gamepad-interface
partial interface Gamepad {
  readonly attribute GamepadHand hand;
  // readonly attribute FrozenArray<GamepadHapticActuator> hapticActuators;
  readonly attribute GamepadPose? pose;
};

// https://w3c.github.io/gamepad/extensions.html#gamepadhand-enum
enum GamepadHand {
  "",  /* unknown, both hands, or not applicable */
  "left",
  "right"
};

// https://www.w3.org/TR/gamepad/#extensions-to-the-windoweventhandlers-interface-mixin
partial interface mixin WindowEventHandlers {
  attribute EventHandler ongamepadconnected;
  attribute EventHandler ongamepaddisconnected;
};
