/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use log::debug;
use net_traits::clientstorage::test_msg::ClientStorageTestMsg;

use super::parent::ClientStorageParent;
use super::thread::ClientStorageThread;

struct BoundState {
    thread: Rc<ClientStorageThread>,
    id: u64,
}

pub struct ClientStorageTestParent {
    bound_state: RefCell<Option<BoundState>>,
}

#[allow(clippy::new_without_default)]
impl ClientStorageTestParent {
    pub fn new() -> Rc<Self> {
        Rc::new(ClientStorageTestParent {
            bound_state: RefCell::new(None),
        })
    }

    pub fn bind(self: &Rc<Self>, thread: Rc<ClientStorageThread>, id: u64) {
        thread.register_actor(id, ClientStorageParent::ClientStorageTest(Rc::clone(self)));

        self.bound_state
            .borrow_mut()
            .replace(BoundState { thread, id });
    }

    pub fn recv_message(self: &Rc<Self>, msg: ClientStorageTestMsg) {
        match msg {
            ClientStorageTestMsg::SyncPing(sender) => {
                self.recv_sync_ping();
                let _ = sender.send(());
            },

            ClientStorageTestMsg::Delete => {
                self.recv_delete();
            },
        }
    }

    fn recv_sync_ping(self: &Rc<Self>) {}

    fn recv_delete(self: &Rc<Self>) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            bound_state.thread.unregister_actor(bound_state.id);
        }
    }
}

impl Drop for ClientStorageTestParent {
    fn drop(&mut self) {
        debug!("Dropping ClientStorageTestParent");
    }
}
