/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crossbeam_channel::{Sender, unbounded};
use net_traits::clientstorage::proxy_msg::ClientStorageProxyMsg;
use net_traits::clientstorage::proxy_sender::ClientStorageProxySender;

pub struct TestClientStorageProxySender {
    sender: Sender<ClientStorageProxyMsg>,
}

impl TestClientStorageProxySender {
    pub fn new_boxed() -> Box<dyn ClientStorageProxySender> {
        let (sender, _receiver) = unbounded();
        Box::new(Self { sender })
    }

    pub fn from_sender_boxed(
        sender: Sender<ClientStorageProxyMsg>,
    ) -> Box<dyn ClientStorageProxySender> {
        Box::new(Self { sender })
    }
}

impl ClientStorageProxySender for TestClientStorageProxySender {
    fn clone_box(&self) -> Box<dyn ClientStorageProxySender> {
        Box::new(Self {
            sender: self.sender.clone(),
        })
    }

    fn send(&self, msg: ClientStorageProxyMsg) {
        self.sender.send(msg).unwrap();
    }
}
