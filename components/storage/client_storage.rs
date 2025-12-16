/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;

use base::generic_channel::{
    self, GenericReceiver, GenericSender, RoutedReceiver, to_receive_result,
};
use base::id::StorageKeyConnectionId;
use log::{debug, warn};
use storage_traits::client_storage::{
    ClientStorageThreadMessage, StorageKeyConnectionBackendMessage,
};

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
    storage_key_connections: HashMap<StorageKeyConnectionId, StorageKeyConnection>,
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
            storage_key_connections: HashMap::new(),
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
            ClientStorageThreadMessage::NewStorageKeyConnection { connection_id } => {
                self.handle_new_storage_key_connection(connection_id);
            },
            ClientStorageThreadMessage::StorageKeyConnectionBackendMessage {
                connection_id,
                message,
            } => {
                self.handle_storage_key_connection_backend_message(connection_id, message);
            },
            ClientStorageThreadMessage::Exit(sender) => {
                self.handle_exit();
                let _ = sender.send(());
            },
        }
    }

    fn handle_new_storage_key_connection(&mut self, connection_id: StorageKeyConnectionId) {
        let connection = StorageKeyConnection::new(connection_id);

        self.storage_key_connections
            .insert(connection_id, connection);
    }

    fn handle_storage_key_connection_backend_message(
        &mut self,
        connection_id: StorageKeyConnectionId,
        message: StorageKeyConnectionBackendMessage,
    ) {
        let connection = self.storage_key_connections.get(&connection_id).unwrap();

        connection.handle_message(message);
    }

    fn handle_exit(&mut self) {
        self.exiting = true;
    }
}

pub struct StorageKeyConnection {
    _connection_id: StorageKeyConnectionId,
}

impl StorageKeyConnection {
    pub fn new(connection_id: StorageKeyConnectionId) -> Self {
        StorageKeyConnection {
            _connection_id: connection_id,
        }
    }

    fn handle_message(&self, message: StorageKeyConnectionBackendMessage) {
        match message {
            StorageKeyConnectionBackendMessage::Test(sender) => {
                self.handle_test(sender);
            },
        }
    }

    fn handle_test(&self, sender: GenericSender<i32>) {
        debug!("Handlig Test");
        let _ = sender.send(42);
    }
}
