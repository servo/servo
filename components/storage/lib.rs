/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod client_storage;
mod indexeddb;
pub(crate) mod shared;
mod storage_thread;
mod webstorage;

pub use client_storage::ClientStorageThreadFactory;
pub(crate) use indexeddb::IndexedDBThreadFactory;
pub use storage_thread::new_storage_threads;
pub(crate) use webstorage::WebStorageThreadFactory;
