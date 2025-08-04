/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ipc_channel::ipc::IpcSender;
use log::debug;
use net_traits::clientstorage::actor_id::ClientStorageActorId;
use net_traits::clientstorage::mixed_msg::ClientStorageMixedMsg;
use net_traits::clientstorage::routed_msg::ClientStorageRoutedMsg;
use net_traits::clientstorage::test_cursor_msg::ClientStorageTestCursorMsg;
use net_traits::clientstorage::test_msg::ClientStorageTestMsg;

use super::parent::ClientStorageParent;
use super::thread::ClientStorageThread;

struct BoundState {
    thread: Rc<ClientStorageThread>,
    ipc_sender: IpcSender<ClientStorageRoutedMsg>,
    actor_id: ClientStorageActorId,
}

impl BoundState {
    pub fn send_mixed_message(&self, msg: ClientStorageMixedMsg) {
        self.send_routed_message(ClientStorageRoutedMsg {
            actor_id: self.actor_id,
            data: msg,
        });
    }

    pub fn send_routed_message(&self, msg: ClientStorageRoutedMsg) {
        self.ipc_sender.send(msg).unwrap();
    }
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
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            bound_state.send_mixed_message(ClientStorageMixedMsg::ClientStorageTest(msg));
        }
    }

    pub fn recv_message(self: &Rc<Self>, msg: ClientStorageTestMsg) {
        match msg {
            ClientStorageTestMsg::SyncPing => {
                self.recv_sync_ping();
            },

            ClientStorageTestMsg::Ping => {
                self.recv_ping();
            },

            ClientStorageTestMsg::TestCursorConstructor { actor_id } => {
                self.recv_test_cursor_constructor(actor_id);
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

    fn recv_test_cursor_constructor(self: &Rc<Self>, actor_id: ClientStorageActorId) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            let actor = ClientStorageTestCursorParent::new();

            actor.bind(
                Rc::clone(&bound_state.thread),
                bound_state.ipc_sender.clone(),
                actor_id,
            );
        }
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

pub struct ClientStorageTestCursorParent {
    bound_state: RefCell<Option<BoundState>>,
    next_number: Cell<u64>,
}

impl ClientStorageTestCursorParent {
    pub fn new() -> Rc<Self> {
        Rc::new(ClientStorageTestCursorParent {
            bound_state: RefCell::new(None),
            next_number: Cell::new(1),
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
            ClientStorageParent::ClientStorageTestCursor(Rc::clone(self)),
        );

        self.bound_state.borrow_mut().replace(BoundState {
            thread,
            ipc_sender,
            actor_id,
        });
    }

    pub fn send_response(self: &Rc<Self>) {
        let number = self.next_number.get();
        self.next_number.set(number + 1);

        self.send_message(ClientStorageTestCursorMsg::Response(number));
    }

    fn send_message(&self, msg: ClientStorageTestCursorMsg) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            bound_state.send_mixed_message(ClientStorageMixedMsg::ClientStorageTestCursor(msg));
        }
    }

    pub fn recv_message(self: &Rc<Self>, msg: ClientStorageTestCursorMsg) {
        match msg {
            ClientStorageTestCursorMsg::Continue => {
                self.recv_continue();
            },

            ClientStorageTestCursorMsg::Delete => {
                self.recv_delete();
            },

            _ => {},
        }
    }

    fn recv_continue(self: &Rc<Self>) {
        self.send_response();
    }

    fn recv_delete(self: &Rc<Self>) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            bound_state.thread.unregister_actor(bound_state.actor_id);
        }
    }
}

impl Drop for ClientStorageTestCursorParent {
    fn drop(&mut self) {
        debug!("Dropping ClientStorageTestCursorParent");
    }
}
