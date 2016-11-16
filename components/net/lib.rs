/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(fnbox)]
#![feature(mpsc_select)]
#![feature(plugin)]
#![feature(proc_macro)]
#![plugin(plugins)]

#![deny(unsafe_code)]

#[macro_use]
extern crate bitflags;
extern crate brotli;
extern crate content_blocker as content_blocker_parser;
extern crate cookie as cookie_rs;
extern crate devtools_traits;
extern crate flate2;
extern crate hyper;
extern crate hyper_serde;
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
extern crate rustc_serialize;
#[macro_use]
extern crate serde_derive;
extern crate servo_url;
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

mod about_loader;
mod blob_loader;
mod chrome_loader;
mod connector;
mod content_blocker;
pub mod cookie;
pub mod cookie_storage;
mod data_loader;
mod file_loader;
pub mod filemanager_thread;
pub mod hsts;
mod http_loader;
pub mod image_cache_thread;
pub mod mime_classifier;
pub mod resource_thread;
mod storage_thread;
mod websocket_loader;

/// An implementation of the [Fetch specification](https://fetch.spec.whatwg.org/)
pub mod fetch {
    pub mod cors_cache;
    pub mod methods;
}

/// A module for re-exports of items used in unit tests.
pub mod test {
    pub use chrome_loader::resolve_chrome_url;
    pub use http_loader::{HttpRequest, HttpRequestFactory, HttpResponse, HttpState};
    pub use http_loader::{LoadError, LoadErrorType, UIProvider, load};
}
