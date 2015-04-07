#![cfg_attr(test, feature(net, alloc))]

extern crate geom;
extern crate gfx;
extern crate net;
extern crate net_traits;
extern crate profile;
extern crate util;
extern crate url;

#[cfg(test)] #[path="gfx/mod.rs"] mod gfx_tests;
#[cfg(test)] #[path="net/mod.rs"] mod net_tests;
#[cfg(test)] #[path="util/mod.rs"] mod util_tests;

