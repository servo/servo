/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use ipc_channel::ipc::IpcReceiver;
use net_traits::clientstorage::thread_msg::ClientStorageThreadMsg;

pub struct ClientStorageThread {
    _base_dir: PathBuf,
}

impl ClientStorageThread {
    pub fn new(config_dir: Option<PathBuf>) -> ClientStorageThread {
        let base_dir = config_dir
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clientstorage");

        ClientStorageThread {
            _base_dir: base_dir,
        }
    }

    pub fn start(&mut self, ipc_receiver: IpcReceiver<ClientStorageThreadMsg>) {
        #[allow(clippy::never_loop)]
        loop {
            match ipc_receiver.recv().unwrap() {
                ClientStorageThreadMsg::Exit(sender) => {
                    let _ = sender.send(());
                    return;
                },
            }
        }
    }
}
