/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::{Deserialize, Serialize};

use super::actor_id::ClientStorageActorId;

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientStorageTestMsg {
    // child to parent:
    SyncPing,

    Ping,

    TestCursorConstructor { actor_id: ClientStorageActorId },

    Delete,

    // parent to child:
    SyncPingReply,

    Pong,
}

impl ClientStorageTestMsg {
    pub fn is_sync_reply(&self) -> bool {
        matches!(self, ClientStorageTestMsg::SyncPingReply)
    }
}
