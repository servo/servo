/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[expect(clippy::module_inception, reason = "The interface name is Gamepad")]
pub(crate) mod gamepad;
pub(crate) use gamepad::Gamepad;
pub(crate) mod gamepadbutton;
pub(crate) mod gamepadbuttonlist;
pub(crate) mod gamepadevent;
pub(crate) mod gamepadhapticactuator;
pub(crate) mod gamepadpose;
