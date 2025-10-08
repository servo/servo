/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod indexeddb;
pub(crate) mod shared;
mod storage_thread;
mod webstorage;

pub(crate) use indexeddb::IndexedDBThreadFactory;
pub use storage_thread::new_storage_threads;
pub(crate) use webstorage::WebStorageThreadFactory;
