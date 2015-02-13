/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(collections)]
#![feature(core)]
#![feature(env)]
#![feature(int_uint)]
#![feature(io)]
#![feature(path)]
#![feature(rustc_private)]
#![feature(std_misc)]
#![feature(unboxed_closures)]

#![allow(missing_copy_implementations)]

extern crate "cookie" as cookie_rs;
extern crate collections;
extern crate geom;
extern crate hyper;
extern crate png;
#[macro_use]
extern crate log;
extern crate openssl;
extern crate "rustc-serialize" as serialize;
extern crate util;
extern crate stb_image;
extern crate time;
extern crate url;

/// Image handling.
///
/// It may be surprising that this goes in the network crate as opposed to the graphics crate.
/// However, image handling is generally very integrated with the network stack (especially where
/// caching is involved) and as a result it must live in here.
pub mod image {
    pub mod base;
    pub mod holder;
}

pub mod about_loader;
pub mod file_loader;
pub mod http_loader;
pub mod data_loader;
pub mod cookie;
pub mod cookie_storage;
pub mod image_cache_task;
pub mod local_image_cache;
pub mod pub_domains;
pub mod resource_task;
pub mod storage_task;
mod sniffer_task;

/// An implementation of the [Fetch spec](http://fetch.spec.whatwg.org/)
pub mod fetch {
    #![allow(dead_code, unused)] // XXXManishearth this is only temporary until the Fetch mod starts being used
    pub mod request;
    pub mod response;
    pub mod cors_cache;
}
