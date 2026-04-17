/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use servo_base::generic_channel::{self, GenericReceiver, GenericSender};
use servo_base::id::WebViewId;
use servo_url::ImmutableOrigin;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientStorageThreadHandle {
    sender: GenericSender<ClientStorageThreadMessage>,
}

impl ClientStorageThreadHandle {
    pub fn new(sender: GenericSender<ClientStorageThreadMessage>) -> Self {
        ClientStorageThreadHandle { sender }
    }

    pub fn obtain_a_storage_bottle_map(
        &self,
        storage_type: StorageType,
        webview: WebViewId,
        storage_identifier: StorageIdentifier,
        origin: ImmutableOrigin,
    ) -> GenericReceiver<Result<StorageProxyMap, String>> {
        let (sender, receiver) = generic_channel::channel().unwrap();
        let message = ClientStorageThreadMessage::ObtainBottleMap {
            storage_type,
            webview,
            storage_identifier,
            origin,
            sender,
        };
        self.sender.send(message).unwrap();
        receiver
    }

    pub fn create_database(
        &self,
        bottle_id: i64,
        name: String,
    ) -> GenericReceiver<Result<PathBuf, String>> {
        let (sender, receiver) = generic_channel::channel().unwrap();
        let message = ClientStorageThreadMessage::CreateDatabase {
            bottle_id,
            name,
            sender,
        };
        self.sender.send(message).unwrap();
        receiver
    }

    pub fn delete_database(
        &self,
        bottle_id: i64,
        name: String,
    ) -> GenericReceiver<Result<(), String>> {
        let (sender, receiver) = generic_channel::channel().unwrap();
        let message = ClientStorageThreadMessage::DeleteDatabase {
            bottle_id,
            name,
            sender,
        };
        self.sender.send(message).unwrap();
        receiver
    }
}

impl From<ClientStorageThreadHandle> for GenericSender<ClientStorageThreadMessage> {
    fn from(handle: ClientStorageThreadHandle) -> Self {
        handle.sender
    }
}

impl Deref for ClientStorageThreadHandle {
    type Target = GenericSender<ClientStorageThreadMessage>;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

impl DerefMut for ClientStorageThreadHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sender
    }
}

/// <https://storage.spec.whatwg.org/#storage-type>
#[derive(Debug, Deserialize, Serialize)]
pub enum StorageType {
    Local,
    Session,
}

impl StorageType {
    pub fn as_str(&self) -> &str {
        match self {
            StorageType::Local => "local",
            StorageType::Session => "session",
        }
    }
}

/// <https://storage.spec.whatwg.org/#bucket-mode>
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum Mode {
    /// It is initially "best-effort".
    #[default]
    BestEffort,
    Persistent,
}

impl Mode {
    pub fn as_str(&self) -> &str {
        match self {
            Mode::BestEffort => "best-effort",
            Mode::Persistent => "persistent",
        }
    }
}

/// <https://storage.spec.whatwg.org/#storage-identifier>
#[derive(Debug, Deserialize, Serialize)]
pub enum StorageIdentifier {
    Caches,
    IndexedDB,
    LocalStorage,
    ServiceWorkerRegistrations,
    SessionStorage,
}

impl StorageIdentifier {
    pub fn as_str(&self) -> &str {
        match self {
            StorageIdentifier::Caches => "caches",
            StorageIdentifier::IndexedDB => "indexeddb",
            StorageIdentifier::LocalStorage => "localstorage",
            StorageIdentifier::ServiceWorkerRegistrations => "serviceworkerregistration",
            StorageIdentifier::SessionStorage => "sessionstorage",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CreateBucketError<T> {
    BucketAlreadyExists,
    Internal(T),
}

impl<T> From<T> for CreateBucketError<T> {
    fn from(err: T) -> Self {
        CreateBucketError::Internal(err)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientStorageErrorr<T> {
    BottleAlreadyExists,
    BucketDoesNotExist,
    DatabaseAlreadyExists,
    DatabaseDoesNotExist,
    DirectoryCreationFailed,
    DirectoryDeletionFailed,
    Internal(T),
}

impl<T> From<T> for ClientStorageErrorr<T> {
    fn from(err: T) -> Self {
        ClientStorageErrorr::Internal(err)
    }
}

/// <https://storage.spec.whatwg.org/#storage-proxy-map>
#[derive(Debug, Deserialize, Serialize)]
pub struct StorageProxyMap {
    pub bottle_id: i64,
    pub handle: ClientStorageThreadHandle,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientStorageThreadMessage {
    ObtainBottleMap {
        storage_type: StorageType,
        webview: WebViewId,
        storage_identifier: StorageIdentifier,
        origin: ImmutableOrigin,
        sender: GenericSender<Result<StorageProxyMap, String>>,
    },
    CreateDatabase {
        bottle_id: i64,
        name: String,
        sender: GenericSender<Result<PathBuf, String>>,
    },
    DeleteDatabase {
        bottle_id: i64,
        name: String,
        sender: GenericSender<Result<(), String>>,
    },
    /// Send a reply when done cleaning up thread resources and then shut it down
    Exit(GenericSender<()>),
}
