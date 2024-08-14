/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{LazyLock, Mutex};

use tokio::runtime::Runtime;

pub static HANDLE: LazyLock<Mutex<Option<Runtime>>> =
    LazyLock::new(|| Mutex::new(Some(Runtime::new().unwrap())));
