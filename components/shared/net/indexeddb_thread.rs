/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use ipc_channel::ipc::IpcSender;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_url::origin::ImmutableOrigin;

// TODO Box<dyn Error> is not serializable, fix needs to be found
pub type DbError = String;
/// A DbResult wraps any part of a call that has to reach into the backend (in this case sqlite.rs)
/// These errors could be anything, depending on the backend
pub type DbResult<T> = Result<T, DbError>;

/// Any error from the backend
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BackendError {
    DbNotFound,
    StoreNotFound,
    DbErr(DbError),
}

impl From<DbError> for BackendError {
    fn from(value: DbError) -> Self {
        BackendError::DbErr(value)
    }
}

impl Display for BackendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for BackendError {}

pub type BackendResult<T> = Result<T, BackendError>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum KeyPath {
    String(String),
    Sequence(Vec<String>),
}

// https://www.w3.org/TR/IndexedDB-2/#enumdef-idbtransactionmode
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IndexedDBTxnMode {
    Readonly,
    Readwrite,
    Versionchange,
}

/// <https://www.w3.org/TR/IndexedDB-2/#key-type>
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum IndexedDBKeyType {
    Number(f64),
    String(String),
    Binary(Vec<u8>),
    Date(f64),
    Array(Vec<IndexedDBKeyType>),
    // FIXME:(arihant2math) implment ArrayBuffer
}

/// <https://www.w3.org/TR/IndexedDB-2/#compare-two-keys>
impl PartialOrd for IndexedDBKeyType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 1. Let ta be the type of a.
        // 2. Let tb be the type of b.

        match (self, other) {
            // Step 3: If ta is array and tb is binary, string, date or number, return 1.
            (
                IndexedDBKeyType::Array(_),
                IndexedDBKeyType::Binary(_) |
                IndexedDBKeyType::Date(_) |
                IndexedDBKeyType::Number(_) |
                IndexedDBKeyType::String(_),
            ) => Some(Ordering::Greater),
            // Step 4: If tb is array and ta is binary, string, date or number, return -1.
            (
                IndexedDBKeyType::Binary(_) |
                IndexedDBKeyType::Date(_) |
                IndexedDBKeyType::Number(_) |
                IndexedDBKeyType::String(_),
                IndexedDBKeyType::Array(_),
            ) => Some(Ordering::Less),
            // Step 5: If ta is binary and tb is string, date or number, return 1.
            (
                IndexedDBKeyType::Binary(_),
                IndexedDBKeyType::String(_) |
                IndexedDBKeyType::Date(_) |
                IndexedDBKeyType::Number(_),
            ) => Some(Ordering::Greater),
            // Step 6: If tb is binary and ta is string, date or number, return -1.
            (
                IndexedDBKeyType::String(_) |
                IndexedDBKeyType::Date(_) |
                IndexedDBKeyType::Number(_),
                IndexedDBKeyType::Binary(_),
            ) => Some(Ordering::Less),
            // Step 7: If ta is string and tb is date or number, return 1.
            (
                IndexedDBKeyType::String(_),
                IndexedDBKeyType::Date(_) | IndexedDBKeyType::Number(_),
            ) => Some(Ordering::Greater),
            // Step 8: If tb is string and ta is date or number, return -1.
            (
                IndexedDBKeyType::Date(_) | IndexedDBKeyType::Number(_),
                IndexedDBKeyType::String(_),
            ) => Some(Ordering::Less),
            // Step 9: If ta is date and tb is number, return 1.
            (IndexedDBKeyType::Date(_), IndexedDBKeyType::Number(_)) => Some(Ordering::Greater),
            // Step 10: If tb is date and ta is number, return -1.
            (IndexedDBKeyType::Number(_), IndexedDBKeyType::Date(_)) => Some(Ordering::Less),
            // Step 11 skipped
            // TODO: Likely a tiny bit wrong (use js number comparison)
            (IndexedDBKeyType::Number(a), IndexedDBKeyType::Number(b)) => a.partial_cmp(b),
            // TODO: Likely a tiny bit wrong (use js string comparison)
            (IndexedDBKeyType::String(a), IndexedDBKeyType::String(b)) => a.partial_cmp(b),
            // TODO: Likely a little wrong (use js binary comparison)
            (IndexedDBKeyType::Binary(a), IndexedDBKeyType::Binary(b)) => a.partial_cmp(b),
            (IndexedDBKeyType::Date(a), IndexedDBKeyType::Date(b)) => a.partial_cmp(b),
            // TODO: Probably also wrong (the items in a and b should be compared, double check against the spec)
            (IndexedDBKeyType::Array(a), IndexedDBKeyType::Array(b)) => a.partial_cmp(b),
            // No catch-all is used, rust ensures that all variants are handled
        }
    }
}

impl PartialEq for IndexedDBKeyType {
    fn eq(&self, other: &Self) -> bool {
        let cmp = self.partial_cmp(other);
        match cmp {
            Some(Ordering::Equal) => true,
            Some(Ordering::Less) | Some(Ordering::Greater) => false,
            None => {
                // If we can't compare the two keys, we assume they are not equal.
                false
            },
        }
    }
}

// <https://www.w3.org/TR/IndexedDB-2/#key-range>
#[derive(Clone, Debug, Default, Deserialize, MallocSizeOf, Serialize)]
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
    pub fn only(key: IndexedDBKeyType) -> Self {
        Self::from(key)
    }

    pub fn new(
        lower: Option<IndexedDBKeyType>,
        upper: Option<IndexedDBKeyType>,
        lower_open: bool,
        upper_open: bool,
    ) -> Self {
        IndexedDBKeyRange {
            lower,
            upper,
            lower_open,
            upper_open,
        }
    }

    pub fn lower_bound(key: IndexedDBKeyType, open: bool) -> Self {
        IndexedDBKeyRange {
            lower: Some(key),
            upper: None,
            lower_open: open,
            upper_open: false,
        }
    }

    pub fn upper_bound(key: IndexedDBKeyType, open: bool) -> Self {
        IndexedDBKeyRange {
            lower: None,
            upper: Some(key),
            lower_open: false,
            upper_open: open,
        }
    }

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

    pub fn is_singleton(&self) -> bool {
        self.lower.is_some() && self.lower == self.upper && !self.lower_open && !self.upper_open
    }

    pub fn as_singleton(&self) -> Option<&IndexedDBKeyType> {
        if self.is_singleton() {
            return Some(self.lower.as_ref().unwrap());
        }
        None
    }
}

#[test]
fn test_as_singleton() {
    let key = IndexedDBKeyType::Number(1.0);
    let key2 = IndexedDBKeyType::Number(2.0);
    let range = IndexedDBKeyRange::only(key.clone());
    assert!(range.is_singleton());
    assert!(range.as_singleton().is_some());
    let range = IndexedDBKeyRange::new(Some(key), Some(key2.clone()), false, false);
    assert!(!range.is_singleton());
    assert!(range.as_singleton().is_none());
    let full_range = IndexedDBKeyRange::new(None, None, false, false);
    assert!(!full_range.is_singleton());
    assert!(full_range.as_singleton().is_none());
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum PutItemResult {
    Success,
    CannotOverwrite,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AsyncReadOnlyOperation {
    /// Gets the value associated with the given key in the associated idb data
    GetKey {
        sender: IpcSender<BackendResult<Option<IndexedDBKeyType>>>,
        key_range: IndexedDBKeyRange,
    },
    GetItem {
        sender: IpcSender<BackendResult<Option<Vec<u8>>>>,
        key_range: IndexedDBKeyRange,
    },

    Count {
        sender: IpcSender<BackendResult<u64>>,
        key_range: IndexedDBKeyRange,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AsyncReadWriteOperation {
    /// Sets the value of the given key in the associated idb data
    PutItem {
        sender: IpcSender<BackendResult<PutItemResult>>,
        key: IndexedDBKeyType,
        value: Vec<u8>,
        should_overwrite: bool,
    },

    /// Removes the key/value pair for the given key in the associated idb data
    RemoveItem {
        sender: IpcSender<BackendResult<()>>,
        key: IndexedDBKeyType,
    },
    /// Clears all key/value pairs in the associated idb data
    Clear(IpcSender<BackendResult<()>>),
}

/// Operations that are not executed instantly, but rather added to a
/// queue that is eventually run.
#[derive(Debug, Deserialize, Serialize)]
pub enum AsyncOperation {
    ReadOnly(AsyncReadOnlyOperation),
    ReadWrite(AsyncReadWriteOperation),
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum CreateObjectResult {
    Created,
    AlreadyExists,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SyncOperation {
    /// Upgrades the version of the database
    UpgradeVersion(
        IpcSender<BackendResult<u64>>,
        ImmutableOrigin,
        String, // Database
        u64,    // Serial number for the transaction
        u64,    // Version to upgrade to
    ),
    /// Checks if an object store has a key generator, used in e.g. Put
    HasKeyGenerator(
        IpcSender<BackendResult<bool>>,
        ImmutableOrigin,
        String, // Database
        String, // Store
    ),
    /// Gets an object store's key path
    KeyPath(
        /// Object stores do not have to have key paths
        IpcSender<BackendResult<Option<KeyPath>>>,
        ImmutableOrigin,
        String, // Database
        String, // Store
    ),

    /// Commits changes of a transaction to the database
    Commit(
        IpcSender<BackendResult<()>>,
        ImmutableOrigin,
        String, // Database
        u64,    // Transaction serial number
    ),

    /// Creates a new index for the database
    CreateIndex(
        IpcSender<BackendResult<CreateObjectResult>>,
        ImmutableOrigin,
        String,  // Database
        String,  // Store
        String,  // Index name
        KeyPath, // key path
        bool,    // unique flag
        bool,    // multientry flag
    ),
    /// Delete an index
    DeleteIndex(
        IpcSender<BackendResult<()>>,
        ImmutableOrigin,
        String, // Database
        String, // Store
        String, // Index name
    ),

    /// Creates a new store for the database
    CreateObjectStore(
        IpcSender<BackendResult<CreateObjectResult>>,
        ImmutableOrigin,
        String,          // Database
        String,          // Store
        Option<KeyPath>, // Key Path
        bool,
    ),

    DeleteObjectStore(
        IpcSender<BackendResult<()>>,
        ImmutableOrigin,
        String, // Database
        String, // Store
    ),

    CloseDatabase(
        IpcSender<BackendResult<()>>,
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
        IpcSender<BackendResult<()>>,
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
        IpcSender<BackendResult<()>>,
        ImmutableOrigin,
        String, // Database
        u64,    // The serial number of the mutating transaction
    ),

    /// Returns the version of the database
    Version(
        IpcSender<BackendResult<u64>>,
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
