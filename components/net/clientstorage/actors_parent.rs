/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use ipc_channel::ipc::IpcSender;
use log::debug;
use net_traits::clientstorage::actor_id::ClientStorageActorId;
use net_traits::clientstorage::mixed_msg::ClientStorageMixedMsg;
use net_traits::clientstorage::routed_msg::ClientStorageRoutedMsg;
use net_traits::clientstorage::test_msg::ClientStorageTestMsg;

use super::parent::ClientStorageParent;
use super::thread::ClientStorageThread;

struct BoundState {
    thread: Rc<ClientStorageThread>,
    ipc_sender: IpcSender<ClientStorageRoutedMsg>,
    actor_id: ClientStorageActorId,
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

    pub fn bind(
        self: &Rc<Self>,
        thread: Rc<ClientStorageThread>,
        ipc_sender: IpcSender<ClientStorageRoutedMsg>,
        actor_id: ClientStorageActorId,
    ) {
        thread.register_actor(
            actor_id,
            ClientStorageParent::ClientStorageTest(Rc::clone(self)),
        );

        self.bound_state.borrow_mut().replace(BoundState {
            thread,
            ipc_sender,
            actor_id,
        });
    }

    pub fn send_sync_ping_reply(self: &Rc<Self>) {
        self.send_message(ClientStorageTestMsg::SyncPingReply);
    }

    pub fn send_pong(self: &Rc<Self>) {
        self.send_message(ClientStorageTestMsg::Pong);
    }

    fn send_message(&self, msg: ClientStorageTestMsg) {
        self.send_mixed_message(ClientStorageMixedMsg::ClientStorageTest(msg));
    }

    fn send_mixed_message(&self, msg: ClientStorageMixedMsg) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            self.send_routed_message(
                bound_state,
                ClientStorageRoutedMsg {
                    actor_id: bound_state.actor_id,
                    data: msg,
                },
            );
        }
    }

    fn send_routed_message(&self, bound_state: &BoundState, msg: ClientStorageRoutedMsg) {
        bound_state.ipc_sender.send(msg).unwrap();
    }

    pub fn recv_message(self: &Rc<Self>, msg: ClientStorageTestMsg) {
        match msg {
            ClientStorageTestMsg::SyncPing => {
                self.recv_sync_ping();
            },

            ClientStorageTestMsg::Ping => {
                self.recv_ping();
            },

            ClientStorageTestMsg::Delete => {
                self.recv_delete();
            },

            _ => {},
        }
    }

    fn recv_sync_ping(self: &Rc<Self>) {
        self.send_sync_ping_reply();
    }

    fn recv_ping(self: &Rc<Self>) {
        self.send_pong();
    }

    fn recv_delete(self: &Rc<Self>) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            bound_state.thread.unregister_actor(bound_state.actor_id);
        }
    }
}

impl Drop for ClientStorageTestParent {
    fn drop(&mut self) {
        debug!("Dropping ClientStorageTestParent");
    }
}
