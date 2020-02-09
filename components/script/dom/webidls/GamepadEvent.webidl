/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/gamepad/#gamepadevent-interface
[Exposed=Window, Pref="dom.gamepad.enabled"]
interface GamepadEvent : Event {
  [Throws] constructor(DOMString type, GamepadEventInit eventInitDict);
  readonly attribute Gamepad gamepad;
};

dictionary GamepadEventInit : EventInit {
  required Gamepad gamepad;
};
