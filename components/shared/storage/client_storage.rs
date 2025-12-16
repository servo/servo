/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::{self, GenericSender};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientStorageThreadMessage {
    /// Send a reply when done cleaning up thread resources and then shut it down
    Exit(GenericSender<()>),
}

pub struct ClientStorageProxy {
    generic_sender: GenericSender<ClientStorageThreadMessage>,
}

impl ClientStorageProxy {
    pub fn new(generic_sender: GenericSender<ClientStorageThreadMessage>) -> ClientStorageProxy {
        ClientStorageProxy { generic_sender }
    }

    pub fn send_exit(&self) {
        let (sender, receiver) = generic_channel::channel().unwrap();
        self.generic_sender
            .send(ClientStorageThreadMessage::Exit(sender))
            .unwrap();
        receiver.recv().unwrap()
    }
}
