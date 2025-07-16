/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use serde::{Deserialize, Serialize};
use servo_url::origin::ImmutableOrigin;

// std::error::Error implemented debug, could be useful for displaying the error
pub type DbError = Box<dyn std::error::Error + Send + Sync>;
pub type DbResult<T> = Result<T, DbError>;

// https://www.w3.org/TR/IndexedDB-2/#enumdef-idbtransactionmode
#[derive(Debug, Deserialize, Serialize)]
pub enum IndexedDBTxnMode {
    Readonly,
    Readwrite,
    Versionchange,
}

// https://www.w3.org/TR/IndexedDB-2/#key-type
// FIXME:(arihant2math) Ordering needs to completely be reimplemented as per https://www.w3.org/TR/IndexedDB-2/#compare-two-keys
#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub enum IndexedDBKeyType {
    Number(f64),
    String(String),
    Binary(Vec<u8>),
    // FIXME:(arihant2math) Date should not be stored as a Vec<u8>
    Date(Vec<u8>),
    Array(Vec<IndexedDBKeyType>),
    // FIXME:(arihant2math) implment ArrayBuffer
}

// <https://www.w3.org/TR/IndexedDB-2/#key-range>
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[allow(unused)]
pub struct IndexedDBKeyRange {
    pub lower: Option<IndexedDBKeyType>,
    pub upper: Option<IndexedDBKeyType>,
    pub lower_open: bool,
    pub upper_open: bool,
}

impl From<IndexedDBKeyType> for IndexedDBKeyRange {
    fn from(key: IndexedDBKeyType) -> Self {
        IndexedDBKeyRange {
            lower: Some(key.clone()),
            upper: Some(key),
            ..Default::default()
        }
    }
}

impl IndexedDBKeyRange {
    // <https://www.w3.org/TR/IndexedDB-2/#in>
    pub fn contains(&self, key: &IndexedDBKeyType) -> bool {
        // A key is in a key range if both of the following conditions are fulfilled:
        // The lower bound is null, or it is less than key,
        // or it is both equal to key and the lower open flag is unset.
        // The upper bound is null, or it is greater than key,
        // or it is both equal to key and the upper open flag is unset
        let lower_bound_condition = self
            .lower
            .as_ref()
            .is_none_or(|lower| lower < key || (!self.lower_open && lower == key));
        let upper_bound_condition = self
            .upper
            .as_ref()
            .is_none_or(|upper| key < upper || (!self.upper_open && key == upper));
        lower_bound_condition && upper_bound_condition
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PutItemResult {
    Success,
    CannotOverwrite,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AsyncReadOnlyOperation {
    /// Gets the value associated with the given key in the associated idb data
    GetItem {
        sender: IpcSender<DbResult<Option<Vec<u8>>>>,
        key: IndexedDBKeyType,
    },

    Count {
        sender: IpcSender<DbResult<u64>>,
        key: IndexedDBKeyType,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AsyncReadWriteOperation {
    /// Sets the value of the given key in the associated idb data
    PutItem {
        sender: IpcSender<DbResult<PutItemResult>>,
        key: IndexedDBKeyType, // Key
        value: Vec<u8>,
        should_overwrite: bool,
    },

    /// Removes the key/value pair for the given key in the associated idb data
    RemoveItem {
        sender: IpcSender<DbResult<()>>,
        key: IndexedDBKeyType,
    },
    /// Clears all key/value pairs in the associated idb data
    Clear(IpcSender<DbResult<()>>),
}

/// Operations that are not executed instantly, but rather added to a
/// queue that is eventually run.
#[derive(Debug, Deserialize, Serialize)]
pub enum AsyncOperation {
    ReadOnly(AsyncReadOnlyOperation),
    ReadWrite(AsyncReadWriteOperation),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CreateObjectStoreResult {
    Created,
    AlreadyExists,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SyncOperation {
    /// Upgrades the version of the database
    UpgradeVersion(
        IpcSender<DbResult<u64>>,
        ImmutableOrigin,
        String, // Database
        u64,    // Serial number for the transaction
        u64,    // Version to upgrade to
    ),
    /// Checks if an object store has a key generator, used in e.g. Put
    HasKeyGenerator(
        IpcSender<DbResult<bool>>,
        ImmutableOrigin,
        String, // Database
        String, // Store
    ),

    /// Commits changes of a transaction to the database
    Commit(
        IpcSender<DbResult<()>>,
        ImmutableOrigin,
        String, // Database
        u64,    // Transaction serial number
    ),

    /// Creates a new store for the database
    CreateObjectStore(
        IpcSender<DbResult<CreateObjectStoreResult>>,
        ImmutableOrigin,
        String, // Database
        String, // Store
        bool,
    ),

    DeleteObjectStore(
        IpcSender<DbResult<()>>,
        ImmutableOrigin,
        String, // Database
        String, // Store
    ),

    CloseDatabase(
        IpcSender<DbResult<()>>,
        ImmutableOrigin,
        String, // Database
    ),

    OpenDatabase(
        IpcSender<u64>, // Returns the version
        ImmutableOrigin,
        String,      // Database
        Option<u64>, // Eventual version
    ),

    /// Deletes the database
    DeleteDatabase(
        IpcSender<DbResult<()>>,
        ImmutableOrigin,
        String, // Database
    ),

    /// Returns an unique identifier that is used to be able to
    /// commit/abort transactions.
    RegisterNewTxn(
        IpcSender<u64>,
        ImmutableOrigin,
        String, // Database
    ),

    /// Starts executing the requests of a transaction
    /// <https://www.w3.org/TR/IndexedDB-2/#transaction-start>
    StartTransaction(
        IpcSender<DbResult<()>>,
        ImmutableOrigin,
        String, // Database
        u64,    // The serial number of the mutating transaction
    ),

    /// Returns the version of the database
    Version(
        IpcSender<DbResult<u64>>,
        ImmutableOrigin,
        String, // Database
    ),

    /// Send a reply when done cleaning up thread resources and then shut it down
    Exit(IpcSender<()>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum IndexedDBThreadMsg {
    Sync(SyncOperation),
    Async(
        ImmutableOrigin,
        String, // Database
        String, // ObjectStore
        u64,    // Serial number of the transaction that requests this operation
        IndexedDBTxnMode,
        AsyncOperation,
    ),
}
