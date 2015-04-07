// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
