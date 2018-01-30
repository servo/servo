/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg(test)]

extern crate gfx;
extern crate gfx_traits;
extern crate ipc_channel;
extern crate metrics;
extern crate msg;
extern crate net_traits;
extern crate profile_traits;
extern crate servo_url;
extern crate time;
extern crate webrender_api;

mod interactive_time;
mod paint_time;
