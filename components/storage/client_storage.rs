/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::thread;

use base::generic_channel::{
    self, GenericReceiver, GenericSender, RoutedReceiver, to_receive_result,
};
use log::warn;
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
    routed_receiver: RoutedReceiver<ClientStorageThreadMessage>,
    exiting: bool,
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

        let routed_receiver = generic_receiver.route_preserving_errors();

        ClientStorageThread {
            _base_dir: base_dir,
            _generic_sender: generic_sender,
            routed_receiver,
            exiting: false,
        }
    }

    pub fn start(&mut self) {
        loop {
            let receive_result = to_receive_result(self.routed_receiver.recv());

            let message = match receive_result {
                Ok(message) => message,
                Err(error) => {
                    warn!("Error on ClientStorageThread receive ({})", error);
                    break;
                },
            };

            self.handle_message(message);

            if self.exiting {
                break;
            }
        }
    }

    fn handle_message(&mut self, message: ClientStorageThreadMessage) {
        match message {
            ClientStorageThreadMessage::Exit(sender) => {
                self.handle_exit();
                let _ = sender.send(());
            },
        }
    }

    fn handle_exit(&mut self) {
        self.exiting = true;
    }
}
