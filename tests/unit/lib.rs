/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![cfg_attr(test, feature(net, alloc))]

#![plugin(string_cache_plugin)]

extern crate cssparser;
extern crate geom;
extern crate gfx;
extern crate net;
extern crate net_traits;
extern crate profile;
extern crate script;
extern crate selectors;
extern crate string_cache;
extern crate style;
extern crate util;
extern crate url;

#[cfg(test)] #[path="gfx/mod.rs"] mod gfx_tests;
#[cfg(test)] #[path="net/mod.rs"] mod net_tests;
#[cfg(test)] #[path="script/mod.rs"] mod script_tests;
#[cfg(test)] #[path="style/mod.rs"] mod style_tests;
#[cfg(test)] #[path="util/mod.rs"] mod util_tests;
