/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::path::PathBuf;

use servo_base::generic_channel::GenericSender;
use servo_base::id::WebViewId;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use servo_url::ImmutableOrigin;

/// Unlike other types, shelves don't have any additional metadata
pub type Shelf = ImmutableOrigin;

/// Uniquely identifies a storage bottle for a given bucket
#[derive(Debug, Deserialize, Serialize)]
pub enum BottleIdent {
    LocalStorage,
    IndexedDB(String),
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

/// <https://storage.spec.whatwg.org/#storage-identifier>
#[derive(Debug, Deserialize, Serialize)]
pub enum StorageIdentifier {
    Caches,
    IndexedDB,
    LocalStorage,
    ServiceWorkerRegistrattions,
    SessionStorage,
}

impl StorageIdentifier {
    pub fn as_str(&self) -> &str {
        match self {
            StorageIdentifier::Caches => "caches",
            StorageIdentifier::IndexedDB => "indexeddb",
            StorageIdentifier::LocalStorage => "localstorage",
            StorageIdentifier::ServiceWorkerRegistrattions => "serviceworkerregistration",
            StorageIdentifier::SessionStorage => "sessionstorage",
        }
    }
}

impl BottleIdent {
    pub fn type_str(&self) -> &'static str {
        match self {
            BottleIdent::LocalStorage => "local_storage",
            BottleIdent::IndexedDB(_) => "indexeddb",
        }
    }

    pub fn database_name(&self) -> String {
        match self {
            BottleIdent::LocalStorage => "local_storage".to_string(),
            BottleIdent::IndexedDB(name) => format!("indexeddb_{}", name),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Bottle {
    pub bottle_type: BottleIdent,
    pub quota: Option<u64>,
}

/// Uniquely identifies a storage bucket within a shelf
#[derive(Debug, Deserialize, Serialize)]
pub enum BucketIdent {
    Default,
    Named(String),
}

impl BucketIdent {
    pub fn as_str(&self) -> &str {
        match self {
            BucketIdent::Default => "default",
            BucketIdent::Named(name) => name.as_str(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Bucket {
    pub bucket_type: BucketIdent,
    pub quota: Option<u64>,
    pub expiration: Option<DateTime<Local>>,
    pub persistent: bool,
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
pub enum CreateBottleError<T> {
    BottleAlreadyExists,
    BucketDoesNotExist,
    DatabaseAlreadyExists,
    DirectoryCreationFailed,
    Internal(T),
}

impl<T> From<T> for CreateBottleError<T> {
    fn from(err: T) -> Self {
        CreateBottleError::Internal(err)
    }
}

/// <https://storage.spec.whatwg.org/#storage-proxy-map>
#[derive(Debug, Deserialize, Serialize)]
pub struct StorageProxyMap {
    pub bottle_id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientStorageThreadMessage {
    CreateBucket {
        shelf: Shelf,
        bucket: Bucket,
        sender: GenericSender<Result<(), CreateBucketError<String>>>,
    },
    ObtainBottleMap {
        storage_type: StorageType,
        webview: WebViewId,
        storage_identifier: StorageIdentifier,
        origin: ImmutableOrigin,
        sender: GenericSender<Result<StorageProxyMap, CreateBottleError<String>>>,
    },
    /// Open a bottle for use
    OpenBottle {
        shelf: Shelf,
        bucket: BucketIdent,
        bottle: BottleIdent,
        sender: GenericSender<Result<PathBuf, String>>,
    },
    /// Delete all storage associated with the given shelf
    DeleteShelf {
        shelf: Shelf,
        sender: GenericSender<Result<(), String>>,
    },
    /// Delete all storage associated with the given bucket
    DeleteBucket {
        shelf: Shelf,
        bucket: BucketIdent,
        sender: GenericSender<Result<(), String>>,
    },
    /// Delete all storage associated with the given bottle
    DeleteBottle {
        shelf: Shelf,
        bucket: BucketIdent,
        bottle: BottleIdent,
        sender: GenericSender<Result<(), String>>,
    },
    /// Delete everything
    DeleteAll {
        sender: GenericSender<Result<(), String>>,
    },
    /// Send a reply when done cleaning up thread resources and then shut it down
    Exit(GenericSender<()>),
}
