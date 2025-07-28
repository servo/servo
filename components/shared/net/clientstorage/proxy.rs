/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crossbeam_channel::{self, Receiver, Sender, unbounded};
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use log::debug;

use super::actors_child::ClientStorageTestChild;
use super::child::ClientStorageChild;
use super::mixed_msg::ClientStorageMixedMsg;
use super::proxy_msg::ClientStorageProxyMsg;
use super::proxy_sender::ClientStorageProxySender;
use super::routed_msg::ClientStorageRoutedMsg;
use super::thread_msg::ClientStorageThreadMsg;

pub struct ClientStorageProxy {
    ipc_sender: IpcSender<ClientStorageThreadMsg>,
    msg_sender: Box<dyn ClientStorageProxySender>,
    sync_sender: Sender<ClientStorageRoutedMsg>,
    sync_receiver: Receiver<ClientStorageRoutedMsg>,
    actors: RefCell<HashMap<u64, ClientStorageChild>>,
}

impl ClientStorageProxy {
    pub fn new(
        ipc_sender: IpcSender<ClientStorageThreadMsg>,
        msg_sender: Box<dyn ClientStorageProxySender>,
    ) -> Rc<ClientStorageProxy> {
        let (sync_sender, sync_receiver) = unbounded();

        Rc::new(ClientStorageProxy {
            ipc_sender,
            msg_sender,
            sync_sender,
            sync_receiver,
            actors: RefCell::new(HashMap::new()),
        })
    }

    pub fn sync_receiver(&self) -> &Receiver<ClientStorageRoutedMsg> {
        &self.sync_receiver
    }

    pub fn send_test_constructor(self: &Rc<Self>, actor: &Rc<ClientStorageTestChild>) {
        // Messages the parent receives
        let (child_to_parent_sender, child_to_parent_receiver) = ipc::channel().unwrap();

        // Messages the child receives
        let (parent_to_child_sender, parent_to_child_receiver) = ipc::channel().unwrap();

        let msg_sender = self.msg_sender.clone();

        let sync_sender = self.sync_sender.clone();

        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_sender
            .send(ClientStorageThreadMsg::TestConstructor {
                child_to_parent_receiver,
                parent_to_child_sender,
                sender,
            })
            .unwrap();
        let global_id = receiver.recv().unwrap();

        ROUTER.add_typed_route(
            parent_to_child_receiver,
            Box::new(move |result| {
                let msg = result.unwrap();

                if msg.is_sync_reply() {
                    sync_sender.send(msg).unwrap();
                } else {
                    msg_sender.send(ClientStorageProxyMsg::Routed(msg));
                }
            }),
        );

        actor.bind(Rc::clone(self), child_to_parent_sender, global_id);
    }

    pub fn send_exit(self: &Rc<Self>) {
        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_sender
            .send(ClientStorageThreadMsg::Exit(sender))
            .unwrap();
        receiver.recv().unwrap();
    }

    pub fn recv_proxy_message(self: &Rc<Self>, msg: ClientStorageProxyMsg) {
        match msg {
            ClientStorageProxyMsg::Routed(msg) => {
                self.recv_routed_message(msg);
            },
        }
    }

    fn recv_routed_message(self: &Rc<Self>, msg: ClientStorageRoutedMsg) {
        if let Some((actor, msg)) = {
            let actors = self.actors.borrow();

            let actor = actors.get(&msg.global_id).unwrap();

            match (actor, msg.data) {
                (
                    ClientStorageChild::ClientStorageTest(actor),
                    ClientStorageMixedMsg::ClientStorageTest(msg),
                ) => Some((Rc::clone(actor), msg)),
            }
        } {
            actor.recv_message(msg);
        }
    }

    pub fn register_actor(self: &Rc<Self>, global_id: u64, actor: ClientStorageChild) {
        self.actors.borrow_mut().insert(global_id, actor);
    }

    pub fn unregister_actor(self: &Rc<Self>, global_id: u64) {
        self.actors.borrow_mut().remove(&global_id);
    }
}

impl Drop for ClientStorageProxy {
    fn drop(&mut self) {
        debug!("Dropping ClientStorageProxy");
    }
}
