/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Exists only to hook into clippy.

#![cfg_attr(feature = "clippy", feature(plugin, plugin_registrar, rustc_private))]
#![deny(unsafe_code)]

#[cfg(feature = "clippy")]
extern crate clippy_lints;
#[cfg(feature = "clippy")]
extern crate rustc_plugin;

#[cfg(feature = "clippy")]
use rustc_plugin::Registry;

#[cfg(feature = "clippy")]
#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    ::clippy_lints::register_plugins(reg);
}
