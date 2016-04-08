/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(fnbox)]
#![feature(mpsc_select)]
#![feature(plugin)]
#![feature(plugin)]
#![plugin(plugins)]

#![deny(unsafe_code)]

extern crate brotli;
extern crate cookie as cookie_rs;
extern crate devtools_traits;
extern crate flate2;
extern crate hyper;
extern crate immeta;
extern crate ipc_channel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate mime;
extern crate mime_guess;
extern crate msg;
extern crate net_traits;
extern crate openssl;
extern crate rustc_serialize;
extern crate threadpool;
extern crate time;
extern crate url;
extern crate util;
extern crate uuid;
extern crate webrender_traits;
extern crate websocket;

pub mod about_loader;
pub mod cookie;
pub mod cookie_storage;
pub mod data_loader;
pub mod file_loader;
pub mod hsts;
pub mod http_loader;
pub mod image_cache_thread;
pub mod mime_classifier;
pub mod pub_domains;
pub mod resource_thread;
pub mod storage_thread;
pub mod websocket_loader;

/// An implementation of the [Fetch spec](https://fetch.spec.whatwg.org/)
pub mod fetch {
    pub mod cors_cache;
    pub mod methods;
}
