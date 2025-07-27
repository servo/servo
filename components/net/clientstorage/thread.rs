/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crossbeam_channel::{self, Receiver, Sender, unbounded};
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use log::debug;
use net_traits::clientstorage::actor_id::ClientStorageActorId;
use net_traits::clientstorage::mixed_msg::ClientStorageMixedMsg;
use net_traits::clientstorage::routed_msg::ClientStorageRoutedMsg;
use net_traits::clientstorage::thread_msg::ClientStorageThreadMsg;

use super::actors_parent::ClientStorageTestParent;
use super::parent::ClientStorageParent;

pub struct ClientStorageThread {
    _base_dir: PathBuf,
    msg_sender: Sender<ClientStorageThreadMsg>,
    msg_receiver: Receiver<ClientStorageThreadMsg>,
    actors: RefCell<HashMap<ClientStorageActorId, ClientStorageParent>>,
    next_global_id: Cell<u64>,
    exiting: Cell<bool>,
}

impl ClientStorageThread {
    pub fn new(config_dir: Option<PathBuf>) -> Rc<ClientStorageThread> {
        let base_dir = config_dir
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clientstorage");

        let (msg_sender, msg_receiver) = unbounded();

        Rc::new(ClientStorageThread {
            _base_dir: base_dir,
            msg_sender,
            msg_receiver,
            actors: RefCell::new(HashMap::new()),
            next_global_id: Cell::new(1),
            exiting: Cell::new(false),
        })
    }

    pub fn start(self: &Rc<Self>, ipc_receiver: IpcReceiver<ClientStorageThreadMsg>) {
        let msg_sender = self.msg_sender.clone();

        ROUTER.add_typed_route(
            ipc_receiver,
            Box::new(move |result| {
                let _ = msg_sender.send(result.unwrap());
            }),
        );

        loop {
            let msg = self.msg_receiver.recv().unwrap();

            self.recv_thread_message(msg);

            if self.exiting.get() {
                break;
            }
        }
    }

    fn recv_thread_message(self: &Rc<Self>, msg: ClientStorageThreadMsg) {
        match msg {
            ClientStorageThreadMsg::TestConstructor {
                child_to_parent_receiver,
                parent_to_child_sender,
                sender,
            } => {
                let actor_id =
                    self.recv_test_constructor(child_to_parent_receiver, parent_to_child_sender);
                let _ = sender.send(actor_id);
            },

            ClientStorageThreadMsg::Routed(msg) => {
                self.recv_routed_message(msg);
            },

            ClientStorageThreadMsg::Exit(sender) => {
                self.recv_exit();
                let _ = sender.send(());
            },
        }
    }

    fn recv_test_constructor(
        self: &Rc<Self>,
        child_to_parent_receiver: IpcReceiver<ClientStorageRoutedMsg>,
        parent_to_child_sender: IpcSender<ClientStorageRoutedMsg>,
    ) -> ClientStorageActorId {
        let msg_sender = self.msg_sender.clone();

        let global_id = self.next_global_id.get();
        self.next_global_id.set(global_id + 1);

        let actor_id = ClientStorageActorId {
            global_id,
            local_id: 0,
        };

        ROUTER.add_typed_route(
            child_to_parent_receiver,
            Box::new(move |result| {
                let msg = result.unwrap();
                let _ = msg_sender.send(ClientStorageThreadMsg::Routed(msg));
            }),
        );

        let actor = ClientStorageTestParent::new();

        actor.bind(Rc::clone(self), parent_to_child_sender, actor_id);

        actor_id
    }

    fn recv_routed_message(self: &Rc<Self>, msg: ClientStorageRoutedMsg) {
        let actor = self.actors.borrow().get(&msg.actor_id).unwrap().clone();

        match (actor, msg.data) {
            (
                ClientStorageParent::ClientStorageTest(actor),
                ClientStorageMixedMsg::ClientStorageTest(msg),
            ) => {
                actor.recv_message(msg);
            },

            (
                ClientStorageParent::ClientStorageTestCursor(actor),
                ClientStorageMixedMsg::ClientStorageTestCursor(msg),
            ) => {
                actor.recv_message(msg);
            },

            _ => {},
        }
    }

    fn recv_exit(self: &Rc<Self>) {
        self.exiting.set(true);
    }

    pub fn register_actor(
        self: &Rc<Self>,
        actor_id: ClientStorageActorId,
        actor: ClientStorageParent,
    ) {
        self.actors.borrow_mut().insert(actor_id, actor);
    }

    pub fn unregister_actor(self: &Rc<Self>, actor_id: ClientStorageActorId) {
        self.actors.borrow_mut().remove(&actor_id);
    }
}

impl Drop for ClientStorageThread {
    fn drop(&mut self) {
        debug!("Dropping ClientStorageThread");
    }
}
