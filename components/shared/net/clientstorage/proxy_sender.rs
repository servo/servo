/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::proxy_msg::ClientStorageProxyMsg;

pub trait ClientStorageProxySender: Send {
    fn clone_box(&self) -> Box<dyn ClientStorageProxySender>;

    fn send(&self, msg: ClientStorageProxyMsg);
}

impl Clone for Box<dyn ClientStorageProxySender> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
