/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Mutex;

use lazy_static::lazy_static;
use tokio::runtime::Runtime;

lazy_static! {
    pub static ref HANDLE: Mutex<Option<Runtime>> = Mutex::new(Some(Runtime::new().unwrap()));
}
