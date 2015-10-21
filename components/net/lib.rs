/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(fnbox)]
#![feature(mpsc_select)]
#![feature(path_ext)]
#![feature(plugin)]
#![feature(vec_push_all)]
#![feature(plugin)]
#![plugin(plugins)]

#[macro_use]
extern crate log;
extern crate cookie as cookie_rs;
extern crate devtools_traits;
extern crate euclid;
extern crate flate2;
extern crate hyper;
extern crate ipc_channel;
extern crate net_traits;
extern crate openssl;
extern crate rustc_serialize;
extern crate time;
extern crate url;
extern crate util;
extern crate uuid;

pub mod about_loader;
pub mod cookie;
pub mod cookie_storage;
pub mod data_loader;
pub mod file_loader;
pub mod hsts;
pub mod http_loader;
pub mod image_cache_task;
pub mod mime_classifier;
pub mod pub_domains;
pub mod resource_task;
pub mod storage_task;

/// An implementation of the [Fetch spec](https://fetch.spec.whatwg.org/)
pub mod fetch {
    #![allow(dead_code, unused)] // XXXManishearth this is only temporary until the Fetch mod starts being used
    pub mod cors_cache;
    pub mod request;
    pub mod response;
}
