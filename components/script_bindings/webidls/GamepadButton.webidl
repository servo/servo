/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/gamepad/#gamepadbutton-interface
[Exposed=Window, Pref="dom.gamepad.enabled"]
interface GamepadButton {
    readonly attribute boolean pressed;
    readonly attribute boolean touched;
    readonly attribute double value;
};
