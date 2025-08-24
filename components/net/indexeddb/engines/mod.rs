/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;

use net_traits::indexeddb_thread::{AsyncOperation, CreateObjectResult, IndexedDBTxnMode, KeyPath};
use tokio::sync::oneshot;

pub use self::sqlite::SqliteEngine;

mod sqlite;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct SanitizedName {
    name: String,
}

impl SanitizedName {
    pub fn new(name: String) -> SanitizedName {
        let name = name.replace("https://", "");
        let name = name.replace("http://", "");
        // FIXME:(arihant2math) Disallowing special characters might be a big problem,
        // but better safe than sorry. E.g. the db name '../other_origin/db',
        // would let us access databases from another origin.
        let name = name
            .chars()
            .map(|c| match c {
                'A'..='Z' => c,
                'a'..='z' => c,
                '0'..='9' => c,
                '-' => c,
                '_' => c,
                _ => '-',
            })
            .collect();
        SanitizedName { name }
    }
}

impl std::fmt::Display for SanitizedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct KvsOperation {
    pub store_name: SanitizedName,
    pub operation: AsyncOperation,
}

pub struct KvsTransaction {
    // Mode could be used by a more optimal implementation of transactions
    // that has different allocated threadpools for reading and writing
    #[allow(unused)]
    pub mode: IndexedDBTxnMode,
    pub requests: VecDeque<KvsOperation>,
}

pub trait KvsEngine {
    type Error: std::error::Error;

    fn create_store(
        &self,
        store_name: SanitizedName,
        key_path: Option<KeyPath>,
        auto_increment: bool,
    ) -> Result<CreateObjectResult, Self::Error>;

    fn delete_store(&self, store_name: SanitizedName) -> Result<(), Self::Error>;

    #[expect(dead_code)]
    fn close_store(&self, store_name: SanitizedName) -> Result<(), Self::Error>;

    fn delete_database(self) -> Result<(), Self::Error>;

    fn process_transaction(
        &self,
        transaction: KvsTransaction,
    ) -> oneshot::Receiver<Option<Vec<u8>>>;

    fn has_key_generator(&self, store_name: SanitizedName) -> bool;
    fn key_path(&self, store_name: SanitizedName) -> Option<KeyPath>;

    fn create_index(
        &self,
        store_name: SanitizedName,
        index_name: String,
        key_path: KeyPath,
        unique: bool,
        multi_entry: bool,
    ) -> Result<CreateObjectResult, Self::Error>;
    fn delete_index(
        &self,
        store_name: SanitizedName,
        index_name: String,
    ) -> Result<(), Self::Error>;

    fn version(&self) -> Result<u64, Self::Error>;
    fn set_version(&self, version: u64) -> Result<(), Self::Error>;

    fn object_store_names(&self) -> Result<Vec<String>, Self::Error>;
}
