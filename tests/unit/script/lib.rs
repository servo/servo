/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]

extern crate euclid;
extern crate msg;
extern crate script;
extern crate servo_url;

#[cfg(test)] mod origin;
#[cfg(all(test, target_pointer_width = "64"))] mod size_of;
#[cfg(test)] mod textinput;
#[cfg(test)] mod headers;
#[cfg(test)] mod htmlareaelement;

