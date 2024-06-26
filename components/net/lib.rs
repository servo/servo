/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

pub mod async_runtime;
pub mod connector;
pub mod cookie;
pub mod cookie_storage;
mod data_loader;
mod decoder;
pub mod filemanager_thread;
mod hosts;
pub mod hsts;
pub mod http_cache;
pub mod http_loader;
pub mod image_cache;
pub mod local_directory_listing;
pub mod mime_classifier;
pub mod resource_thread;
mod storage_thread;
pub mod subresource_integrity;
mod websocket_loader;

/// An implementation of the [Fetch specification](https://fetch.spec.whatwg.org/)
pub mod fetch {
    pub mod cors_cache;
    pub mod headers;
    pub mod methods;
}

/// A module for re-exports of items used in unit tests.
pub mod test {
    pub use crate::hosts::{parse_hostsfile, replace_host_table};
    pub use crate::http_loader::HttpState;
}
