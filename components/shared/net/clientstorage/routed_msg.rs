/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::{Deserialize, Serialize};

use super::mixed_msg::ClientStorageMixedMsg;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientStorageRoutedMsg {
    pub id: u64,
    pub data: ClientStorageMixedMsg,
}

impl ClientStorageRoutedMsg {
    pub fn is_sync_reply(&self) -> bool {
        self.data.is_sync_reply()
    }
}
