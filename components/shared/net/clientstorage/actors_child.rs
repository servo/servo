/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use ipc_channel::ipc::IpcSender;
use log::debug;

use super::actor_id::ClientStorageActorId;
use super::child::ClientStorageChild;
use super::mixed_msg::ClientStorageMixedMsg;
use super::proxy::ClientStorageProxy;
use super::routed_msg::ClientStorageRoutedMsg;
use super::test_msg::ClientStorageTestMsg;

struct BoundState {
    proxy: Rc<ClientStorageProxy>,
    ipc_sender: IpcSender<ClientStorageRoutedMsg>,
    actor_id: ClientStorageActorId,
}

type PongCallback = Box<dyn Fn()>;

pub struct ClientStorageTestChild {
    bound_state: RefCell<Option<BoundState>>,
    pong_callback: RefCell<Option<PongCallback>>,
}

impl ClientStorageTestChild {
    pub fn new() -> Rc<Self> {
        Rc::new(ClientStorageTestChild {
            bound_state: RefCell::new(None),
            pong_callback: RefCell::new(None),
        })
    }

    pub fn bind(
        self: &Rc<Self>,
        proxy: Rc<ClientStorageProxy>,
        ipc_sender: IpcSender<ClientStorageRoutedMsg>,
        actor_id: ClientStorageActorId,
    ) {
        proxy.register_actor(
            actor_id,
            ClientStorageChild::ClientStorageTest(Rc::clone(self)),
        );

        self.bound_state.borrow_mut().replace(BoundState {
            proxy,
            ipc_sender,
            actor_id,
        });
    }

    pub fn send_sync_ping(self: &Rc<Self>) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            self.send_message(bound_state, ClientStorageTestMsg::SyncPing);

            bound_state.proxy.sync_receiver().recv().unwrap();
        }
    }

    pub fn send_ping(self: &Rc<Self>) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            self.send_message(bound_state, ClientStorageTestMsg::Ping);
        }
    }

    pub fn send_delete(&self) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            self.send_message(bound_state, ClientStorageTestMsg::Delete);

            bound_state.proxy.unregister_actor(bound_state.actor_id);
        }
    }

    fn send_message(&self, bound_state: &BoundState, msg: ClientStorageTestMsg) {
        self.send_mixed_message(bound_state, ClientStorageMixedMsg::ClientStorageTest(msg));
    }

    fn send_mixed_message(&self, bound_state: &BoundState, msg: ClientStorageMixedMsg) {
        self.send_routed_message(
            bound_state,
            ClientStorageRoutedMsg {
                actor_id: bound_state.actor_id,
                data: msg,
            },
        );
    }

    fn send_routed_message(&self, bound_state: &BoundState, msg: ClientStorageRoutedMsg) {
        bound_state.ipc_sender.send(msg).unwrap();
    }

    pub fn recv_message(self: &Rc<Self>, msg: ClientStorageTestMsg) {
        #[allow(clippy::single_match)]
        match msg {
            ClientStorageTestMsg::Pong => {
                self.recv_pong();
            },

            _ => {},
        }
    }

    fn recv_pong(self: &Rc<Self>) {
        let pong_callback = self.pong_callback.borrow_mut().take();

        if let Some(callback) = pong_callback {
            callback();
        }
    }

    pub fn set_pong_callback<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.pong_callback.borrow_mut().replace(Box::new(callback));
    }
}

impl Drop for ClientStorageTestChild {
    fn drop(&mut self) {
        debug!("Dropping ClientStorageTestChild");
    }
}
