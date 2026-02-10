/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;

use storage_traits::indexeddb::{
    AsyncOperation, CreateObjectResult, IndexedDBIndex, IndexedDBTxnMode, KeyPath,
};
use tokio::sync::oneshot;

pub use self::sqlite::SqliteEngine;

mod sqlite;

pub struct KvsOperation {
    pub store_name: String,
    pub operation: AsyncOperation,
}

pub struct KvsTransaction {
    // Mode could be used by a more optimal implementation of transactions
    // that has different allocated threadpools for reading and writing
    pub mode: IndexedDBTxnMode,
    pub requests: VecDeque<KvsOperation>,
}

pub trait KvsEngine {
    type Error: std::error::Error;

    fn create_store(
        &self,
        store_name: &str,
        key_path: Option<KeyPath>,
        auto_increment: bool,
    ) -> Result<CreateObjectResult, Self::Error>;

    fn delete_store(&self, store_name: &str) -> Result<(), Self::Error>;

    #[expect(dead_code)]
    fn close_store(&self, store_name: &str) -> Result<(), Self::Error>;

    fn delete_database(self) -> Result<(), Self::Error>;

    fn process_transaction(
        &self,
        transaction: KvsTransaction,
    ) -> oneshot::Receiver<Option<Vec<u8>>>;

    fn has_key_generator(&self, store_name: &str) -> bool;
    fn key_path(&self, store_name: &str) -> Option<KeyPath>;
    fn indexes(&self, store_name: &str) -> Result<Vec<IndexedDBIndex>, Self::Error>;

    fn create_index(
        &self,
        store_name: &str,
        index_name: String,
        key_path: KeyPath,
        unique: bool,
        multi_entry: bool,
    ) -> Result<CreateObjectResult, Self::Error>;
    fn delete_index(&self, store_name: &str, index_name: String) -> Result<(), Self::Error>;

    fn version(&self) -> Result<u64, Self::Error>;
    fn set_version(&self, version: u64) -> Result<(), Self::Error>;
}
