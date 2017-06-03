/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![feature(box_syntax)]
#![feature(iterator_step_by)]

extern crate base64;
extern crate brotli;
extern crate cookie as cookie_rs;
extern crate devtools_traits;
extern crate flate2;
extern crate hyper;
extern crate hyper_openssl;
extern crate hyper_serde;
extern crate immeta;
extern crate ipc_channel;
#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] #[no_link] extern crate matches;
#[macro_use]
extern crate mime;
extern crate mime_guess;
extern crate msg;
extern crate net_traits;
extern crate openssl;
extern crate parse_hosts;
extern crate profile_traits;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate servo_config;
extern crate servo_url;
extern crate time;
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
extern crate tinyfiledialogs;
extern crate unicase;
extern crate url;
extern crate uuid;
extern crate webrender_traits;
extern crate websocket;

mod blob_loader;
mod chrome_loader;
pub mod connector;
pub mod cookie;
pub mod cookie_storage;
mod data_loader;
pub mod filemanager_thread;
mod hosts;
pub mod hsts;
pub mod http_loader;
pub mod image_cache;
pub mod mime_classifier;
pub mod resource_thread;
mod storage_thread;
pub mod subresource_integrity;
mod websocket_loader;
/// An implementation of the [Fetch specification](https://fetch.spec.whatwg.org/)
pub mod fetch {
    pub mod cors_cache;
    pub mod methods;
}

/// A module for re-exports of items used in unit tests.
pub mod test {
    pub use chrome_loader::resolve_chrome_url;
    pub use http_loader::HttpState;
    pub use hosts::{replace_host_table, parse_hostsfile};
}
