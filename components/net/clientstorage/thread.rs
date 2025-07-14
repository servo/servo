/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use crossbeam_channel::unbounded;
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::router::ROUTER;
use net_traits::clientstorage::thread_msg::ClientStorageThreadMsg;

pub struct ClientStorageThread {
    _base_dir: PathBuf,
    exiting: bool,
}

impl ClientStorageThread {
    pub fn new(config_dir: Option<PathBuf>) -> ClientStorageThread {
        let base_dir = config_dir
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clientstorage");

        ClientStorageThread {
            _base_dir: base_dir,
            exiting: false,
        }
    }

    pub fn start(&mut self, ipc_receiver: IpcReceiver<ClientStorageThreadMsg>) {
        let (msg_sender, msg_receiver) = unbounded();

        ROUTER.add_typed_route(
            ipc_receiver,
            Box::new(move |result| {
                let _ = msg_sender.send(result.unwrap());
            }),
        );

        loop {
            let msg = msg_receiver.recv().unwrap();

            self.recv_thread_message(msg);

            if self.exiting {
                break;
            }
        }
    }

    fn recv_thread_message(&mut self, msg: ClientStorageThreadMsg) {
        match msg {
            ClientStorageThreadMsg::Exit(sender) => {
                self.recv_exit();
                let _ = sender.send(());
            },
        }
    }

    fn recv_exit(&mut self) {
        self.exiting = true;
    }
}
