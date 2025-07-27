/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ipc_channel::ipc::IpcSender;
use log::debug;

use super::actor_id::ClientStorageActorId;
use super::child::ClientStorageChild;
use super::mixed_msg::ClientStorageMixedMsg;
use super::proxy::ClientStorageProxy;
use super::routed_msg::ClientStorageRoutedMsg;
use super::test_cursor_msg::ClientStorageTestCursorMsg;
use super::test_msg::ClientStorageTestMsg;

struct BoundState {
    proxy: Rc<ClientStorageProxy>,
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

struct TopBoundState {
    base: BoundState,
    next_local_id: Cell<i64>,
}

impl TopBoundState {
    pub fn new(
        proxy: Rc<ClientStorageProxy>,
        ipc_sender: IpcSender<ClientStorageRoutedMsg>,
        actor_id: ClientStorageActorId,
    ) -> Self {
        Self {
            base: BoundState {
                proxy,
                ipc_sender,
                actor_id,
            },
            next_local_id: Cell::new(-1),
        }
    }

    fn proxy(&self) -> &Rc<ClientStorageProxy> {
        &self.base.proxy
    }

    fn ipc_sender(&self) -> &IpcSender<ClientStorageRoutedMsg> {
        &self.base.ipc_sender
    }

    fn actor_id(&self) -> &ClientStorageActorId {
        &self.base.actor_id
    }

    fn send_mixed_message(&self, msg: ClientStorageMixedMsg) {
        self.base.send_mixed_message(msg);
    }

    fn next_local_id(&self) -> i64 {
        let next_local_id = self.next_local_id.get();
        self.next_local_id.set(next_local_id - 1);

        next_local_id
    }
}

type PongCallback = Box<dyn Fn()>;

pub struct ClientStorageTestChild {
    bound_state: RefCell<Option<TopBoundState>>,
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

        self.bound_state
            .borrow_mut()
            .replace(TopBoundState::new(proxy, ipc_sender, actor_id));
    }

    pub fn send_sync_ping(self: &Rc<Self>) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            self.send_message(bound_state, ClientStorageTestMsg::SyncPing);

            bound_state.proxy().sync_receiver().recv().unwrap();
        }
    }

    pub fn send_ping(self: &Rc<Self>) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            self.send_message(bound_state, ClientStorageTestMsg::Ping);
        }
    }

    pub fn send_test_cursor_constructor(self: &Rc<Self>, actor: &Rc<ClientStorageTestCursorChild>) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            let local_id = bound_state.next_local_id();

            let actor_id = ClientStorageActorId {
                global_id: bound_state.actor_id().global_id,
                local_id,
            };

            self.send_message(
                bound_state,
                ClientStorageTestMsg::TestCursorConstructor { actor_id },
            );

            actor.bind(
                Rc::clone(bound_state.proxy()),
                bound_state.ipc_sender().clone(),
                actor_id,
            );
        }
    }

    pub fn send_delete(&self) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            self.send_message(bound_state, ClientStorageTestMsg::Delete);

            bound_state.proxy().unregister_actor(bound_state.actor_id());
        }
    }

    fn send_message(&self, bound_state: &TopBoundState, msg: ClientStorageTestMsg) {
        bound_state.send_mixed_message(ClientStorageMixedMsg::ClientStorageTest(msg));
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

type ResponseCallback = Box<dyn Fn(u64)>;

pub struct ClientStorageTestCursorChild {
    bound_state: RefCell<Option<BoundState>>,
    response_callback: RefCell<Option<ResponseCallback>>,
}

impl ClientStorageTestCursorChild {
    pub fn new() -> Rc<Self> {
        Rc::new(ClientStorageTestCursorChild {
            bound_state: RefCell::new(None),
            response_callback: RefCell::new(None),
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
            ClientStorageChild::ClientStorageTestCursor(Rc::clone(self)),
        );

        self.bound_state.borrow_mut().replace(BoundState {
            proxy,
            ipc_sender,
            actor_id,
        });
    }

    pub fn send_continue(self: &Rc<Self>) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            self.send_message(bound_state, ClientStorageTestCursorMsg::Continue);
        }
    }

    pub fn send_delete(&self) {
        if let Some(bound_state) = self.bound_state.borrow().as_ref() {
            self.send_message(bound_state, ClientStorageTestCursorMsg::Delete);

            bound_state.proxy.unregister_actor(&bound_state.actor_id);
        }
    }

    fn send_message(&self, bound_state: &BoundState, msg: ClientStorageTestCursorMsg) {
        bound_state.send_mixed_message(ClientStorageMixedMsg::ClientStorageTestCursor(msg));
    }

    pub fn recv_message(self: &Rc<Self>, msg: ClientStorageTestCursorMsg) {
        #[allow(clippy::single_match)]
        match msg {
            ClientStorageTestCursorMsg::Response(number) => {
                self.recv_response(number);
            },

            _ => {},
        }
    }

    fn recv_response(self: &Rc<Self>, number: u64) {
        if let Some(response_callback) = self.response_callback.borrow().as_ref() {
            response_callback(number);
        }
    }

    pub fn set_response_callback<F>(&self, callback: F)
    where
        F: Fn(u64) + 'static,
    {
        self.response_callback
            .borrow_mut()
            .replace(Box::new(callback));
    }
}

impl Drop for ClientStorageTestCursorChild {
    fn drop(&mut self) {
        debug!("Dropping ClientStorageTestCursorChild");
    }
}
