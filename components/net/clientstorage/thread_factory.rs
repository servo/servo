/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::thread;

use ipc_channel::ipc::{self, IpcSender};
use net_traits::clientstorage::thread_msg::ClientStorageThreadMsg;

use super::thread::ClientStorageThread;

pub trait ClientStorageThreadFactory {
    fn new(config_dir: Option<PathBuf>) -> Self;
}

impl ClientStorageThreadFactory for IpcSender<ClientStorageThreadMsg> {
    fn new(config_dir: Option<PathBuf>) -> IpcSender<ClientStorageThreadMsg> {
        let (ipc_sender, ipc_receiver) = ipc::channel().unwrap();

        thread::Builder::new()
            .name("ClientStorageThread".to_owned())
            .spawn(move || {
                let thread = ClientStorageThread::new(config_dir);

                thread.start(ipc_receiver);
            })
            .expect("Thread spawning failed");

        ipc_sender
    }
}
