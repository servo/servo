/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod indexeddb;
mod storage_thread;
pub mod webstorage;

pub use storage_thread::new_storage_threads;
