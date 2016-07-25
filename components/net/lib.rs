/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(box_syntax)]
#![feature(fnbox)]
#![feature(mpsc_select)]
#![feature(plugin)]
#![plugin(plugins)]

#![deny(unsafe_code)]

#[macro_use]
extern crate bitflags;
extern crate brotli;
extern crate content_blocker as content_blocker_parser;
extern crate cookie as cookie_rs;
extern crate device;
extern crate devtools_traits;
extern crate flate2;
extern crate hyper;
extern crate immeta;
extern crate ipc_channel;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] #[no_link] extern crate matches;
#[macro_use]
extern crate mime;
extern crate mime_guess;
extern crate msg;
extern crate net_traits;
extern crate openssl;
extern crate openssl_verify;
extern crate profile_traits;
extern crate rand;
extern crate rustc_serialize;
extern crate threadpool;
extern crate time;
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
extern crate tinyfiledialogs;
extern crate unicase;
extern crate url;
extern crate util;
extern crate uuid;
extern crate webrender_traits;
extern crate websocket;

pub mod about_loader;
pub mod blob_loader;
pub mod bluetooth_thread;
pub mod chrome_loader;
pub mod connector;
pub mod content_blocker;
pub mod cookie;
pub mod cookie_storage;
pub mod data_loader;
pub mod file_loader;
pub mod filemanager_thread;
pub mod hsts;
pub mod http_loader;
pub mod image_cache_thread;
pub mod mime_classifier;
pub mod pub_domains;
pub mod resource_thread;
pub mod storage_thread;
pub mod websocket_loader;

/// An implementation of the [Fetch specification](https://fetch.spec.whatwg.org/)
pub mod fetch {
    pub mod cors_cache;
    pub mod methods;
}
