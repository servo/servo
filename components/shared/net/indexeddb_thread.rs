/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use serde::{Deserialize, Serialize};
use servo_url::origin::ImmutableOrigin;

#[derive(Debug, Deserialize, Serialize)]
pub enum IndexedDBThreadReturnType {
    Open(Option<u64>),
    NextSerialNumber(u64),
    StartTransaction(Result<(), ()>),
    Commit(Result<(), ()>),
    Version(u64),
    CreateObjectStore(Option<String>),
    UpgradeVersion(Result<u64, ()>),
    KVResult(Option<Vec<u8>>),
    Exit,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum IndexedDBTxnMode {
    Readonly,
    Readwrite,
    Versionchange,
}

// https://www.w3.org/TR/IndexedDB-2/#key-type
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum IndexedDBKeyType {
    Number(Vec<u8>),
    String(Vec<u8>),
    Binary(Vec<u8>),
    Date(Vec<u8>),
    // FIXME:(arihant2math) implment Array(),
}

// Operations that are not executed instantly, but rather added to a
// queue that is eventually run.
#[derive(Debug, Deserialize, Serialize)]
pub enum AsyncOperation {
    /// Gets the value associated with the given key in the associated idb data
    GetItem(
        IndexedDBKeyType, // Key
    ),

    /// Sets the value of the given key in the associated idb data
    PutItem(
        IndexedDBKeyType, // Key
        Vec<u8>,          // Value
        bool,             // Should overwrite
    ),

    /// Removes the key/value pair for the given key in the associated idb data
    RemoveItem(
        IndexedDBKeyType, // Key
    ),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SyncOperation {
    // Upgrades the version of the database
    UpgradeVersion(
        IpcSender<IndexedDBThreadReturnType>,
        ImmutableOrigin,
        String, // Database
        u64,    // Serial number for the transaction
        u64,    // Version to upgrade to
    ),
    // Checks if an object store has a key generator, used in e.g. Put
    HasKeyGenerator(
        IpcSender<bool>,
        ImmutableOrigin,
        String, // Database
        String, // Store
    ),

    // Commits changes of a transaction to the database
    Commit(
        IpcSender<IndexedDBThreadReturnType>,
        ImmutableOrigin,
        String, // Database
        u64,    // Transaction serial number
    ),

    // Creates a new store for the database
    CreateObjectStore(
        IpcSender<Result<(), ()>>,
        ImmutableOrigin,
        String, // Database
        String, // Store
        bool,
    ),

    OpenDatabase(
        IpcSender<u64>, // Returns the version
        ImmutableOrigin,
        String,      // Database
        Option<u64>, // Eventual version
    ),

    // Deletes the database
    DeleteDatabase(
        IpcSender<Result<(), ()>>,
        ImmutableOrigin,
        String, // Database
    ),

    // Returns an unique identifier that is used to be able to
    // commit/abort transactions.
    RegisterNewTxn(
        IpcSender<u64>,
        ImmutableOrigin,
        String, // Database
    ),

    // Starts executing the requests of a transaction
    // https://www.w3.org/TR/IndexedDB-2/#transaction-start
    StartTransaction(
        IpcSender<Result<(), ()>>,
        ImmutableOrigin,
        String, // Database
        u64,    // The serial number of the mutating transaction
    ),

    // Returns the version of the database
    Version(
        IpcSender<u64>,
        ImmutableOrigin,
        String, // Database
    ),

    /// Send a reply when done cleaning up thread resources and then shut it down
    Exit(IpcSender<IndexedDBThreadReturnType>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum IndexedDBThreadMsg {
    Sync(SyncOperation),
    Async(
        IpcSender<Option<Vec<u8>>>, // Sender to send the result of the async operation
        ImmutableOrigin,
        String, // Database
        String, // ObjectStore
        u64,    // Serial number of the transaction that requests this operation
        IndexedDBTxnMode,
        AsyncOperation,
    ),
}
