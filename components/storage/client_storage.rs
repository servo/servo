/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::thread;

use base::generic_channel::{self, GenericReceiver, GenericSender};
use storage_traits::client_storage::ClientStorageThreadMessage;

pub trait ClientStorageThreadFactory {
    fn new(config_dir: Option<PathBuf>) -> Self;
}

impl ClientStorageThreadFactory for GenericSender<ClientStorageThreadMessage> {
    fn new(config_dir: Option<PathBuf>) -> GenericSender<ClientStorageThreadMessage> {
        let (generic_sender, generic_receiver) = generic_channel::channel().unwrap();

        let generic_sender_clone = generic_sender.clone();

        thread::Builder::new()
            .name("ClientStorageThread".to_owned())
            .spawn(move || {
                ClientStorageThread::new(config_dir, generic_sender, generic_receiver).start();
            })
            .expect("Thread spawning failed");

        generic_sender_clone
    }
}

pub struct ClientStorageThread {
    _base_dir: PathBuf,
    _generic_sender: GenericSender<ClientStorageThreadMessage>,
    generic_receiver: GenericReceiver<ClientStorageThreadMessage>,
}

impl ClientStorageThread {
    pub fn new(
        config_dir: Option<PathBuf>,
        generic_sender: GenericSender<ClientStorageThreadMessage>,
        generic_receiver: GenericReceiver<ClientStorageThreadMessage>,
    ) -> ClientStorageThread {
        let base_dir = config_dir
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clientstorage");

        ClientStorageThread {
            _base_dir: base_dir,
            _generic_sender: generic_sender,
            generic_receiver,
        }
    }

    pub fn start(&mut self) {
        #[allow(clippy::never_loop)]
        loop {
            match self.generic_receiver.recv().unwrap() {
                ClientStorageThreadMessage::Exit(sender) => {
                    let _ = sender.send(());
                    return;
                },
            }
        }
    }
}
