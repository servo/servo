/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crossbeam_channel::{self, Receiver, Sender, unbounded};
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::router::ROUTER;
use log::debug;
use net_traits::clientstorage::mixed_msg::ClientStorageMixedMsg;
use net_traits::clientstorage::routed_msg::ClientStorageRoutedMsg;
use net_traits::clientstorage::thread_msg::ClientStorageThreadMsg;

use super::actors_parent::ClientStorageTestParent;
use super::parent::ClientStorageParent;

pub struct ClientStorageThread {
    _base_dir: PathBuf,
    msg_sender: Sender<ClientStorageThreadMsg>,
    msg_receiver: Receiver<ClientStorageThreadMsg>,
    actors: RefCell<HashMap<u64, ClientStorageParent>>,
    next_id: Cell<u64>,
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
            next_id: Cell::new(1),
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
                sender,
            } => {
                let id = self.recv_test_constructor(child_to_parent_receiver);
                let _ = sender.send(id);
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
    ) -> u64 {
        let msg_sender = self.msg_sender.clone();

        let id = self.next_id.get();
        self.next_id.set(id + 1);

        ROUTER.add_typed_route(
            child_to_parent_receiver,
            Box::new(move |result| {
                let msg = result.unwrap();
                let _ = msg_sender.send(ClientStorageThreadMsg::Routed(msg));
            }),
        );

        let actor = ClientStorageTestParent::new();

        actor.bind(Rc::clone(self), id);

        id
    }

    fn recv_routed_message(self: &Rc<Self>, msg: ClientStorageRoutedMsg) {
        if let Some((actor, msg)) = {
            let actors = self.actors.borrow();

            let actor = actors.get(&msg.id).unwrap();

            match (actor, msg.data) {
                (
                    ClientStorageParent::ClientStorageTest(actor),
                    ClientStorageMixedMsg::ClientStorageTest(msg),
                ) => Some((Rc::clone(actor), msg)),
            }
        } {
            actor.recv_message(msg);
        }
    }

    fn recv_exit(self: &Rc<Self>) {
        self.exiting.set(true);
    }

    pub fn register_actor(self: &Rc<Self>, id: u64, actor: ClientStorageParent) {
        self.actors.borrow_mut().insert(id, actor);
    }

    pub fn unregister_actor(self: &Rc<Self>, id: u64) {
        self.actors.borrow_mut().remove(&id);
    }
}

impl Drop for ClientStorageThread {
    fn drop(&mut self) {
        debug!("Dropping ClientStorageThread");
    }
}
