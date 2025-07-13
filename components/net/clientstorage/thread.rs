/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::path::PathBuf;

use crossbeam_channel::{self, Receiver, Sender, unbounded};
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::router::ROUTER;
use net_traits::clientstorage::mixed_msg::ClientStorageMixedMsg;
use net_traits::clientstorage::routed_msg::ClientStorageRoutedMsg;
use net_traits::clientstorage::thread_msg::ClientStorageThreadMsg;

use super::actors_parent::ClientStorageTestParent;
use super::parent::ClientStorageParent;

pub struct ClientStorageThread {
    _base_dir: PathBuf,
    msg_sender: Sender<ClientStorageThreadMsg>,
    msg_receiver: Receiver<ClientStorageThreadMsg>,
    actors: HashMap<u64, ClientStorageParent>,
    next_id: u64,
    exiting: bool,
}

impl ClientStorageThread {
    pub fn new(config_dir: Option<PathBuf>) -> ClientStorageThread {
        let base_dir = config_dir
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clientstorage");

        let (msg_sender, msg_receiver) = unbounded();

        ClientStorageThread {
            _base_dir: base_dir,
            msg_sender,
            msg_receiver,
            actors: HashMap::new(),
            next_id: 1,
            exiting: false,
        }
    }

    pub fn start(&mut self, ipc_receiver: IpcReceiver<ClientStorageThreadMsg>) {
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

            if self.exiting {
                break;
            }
        }
    }

    fn recv_thread_message(&mut self, msg: ClientStorageThreadMsg) {
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
        &mut self,
        child_to_parent_receiver: IpcReceiver<ClientStorageRoutedMsg>,
    ) -> u64 {
        let msg_sender = self.msg_sender.clone();

        let id = self.next_id;
        self.next_id += 1;

        ROUTER.add_typed_route(
            child_to_parent_receiver,
            Box::new(move |result| {
                let msg = result.unwrap();
                let _ = msg_sender.send(ClientStorageThreadMsg::Routed(msg));
            }),
        );

        let actor = ClientStorageTestParent::new();

        self.actors
            .insert(id, ClientStorageParent::ClientStorageTest(actor));

        id
    }

    fn recv_routed_message(&mut self, msg: ClientStorageRoutedMsg) {
        let actor = self.actors.get(&msg.id).unwrap();

        match (actor, msg.data) {
            (
                ClientStorageParent::ClientStorageTest(test_actor),
                ClientStorageMixedMsg::ClientStorageTest(test_msg),
            ) => {
                test_actor.recv_message(test_msg);
            },
        }
    }

    fn recv_exit(&mut self) {
        self.exiting = true;
    }
}
