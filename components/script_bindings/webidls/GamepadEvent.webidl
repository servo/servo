/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_GAMEPAD

// https://w3c.github.io/gamepad/#gamepadevent-interface
[Exposed=Window, Pref="dom_gamepad_enabled"]
interface GamepadEvent : Event {
  constructor(DOMString type, optional GamepadEventInit eventInitDict = {});

  // Gamepad attribute is required in the original idl but it is practically
  // impossible to keep it so because GamepadEventInit.gamepad can be null.
  // Discussed in: https://github.com/w3c/gamepad/pull/217
  [SameObject] readonly attribute Gamepad? gamepad;
};

dictionary GamepadEventInit : EventInit {
  Gamepad? gamepad = null;
};
