/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use heed::types::*;
use heed::{Database, Env, EnvOpenOptions};
use log::warn;
use net_traits::indexeddb_thread::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, IdbResult, IndexedDBTxnMode,
};
use tokio::sync::oneshot;

use super::{KvsEngine, KvsTransaction, SanitizedName};
use crate::resource_thread::CoreResourceThreadPool;

type HeedDatabase = Database<Bytes, Bytes>;

// A simple store that also has a key generator that can be used if no key
// is provided for the stored objects
#[derive(Clone)]
struct Store {
    inner: HeedDatabase,
    // https://www.w3.org/TR/IndexedDB-2/#key-generator
    key_generator: Option<u64>,
}

pub struct HeedEngine {
    heed_env: Arc<Env>,
    open_stores: Arc<RwLock<HashMap<SanitizedName, Store>>>,
    read_pool: Arc<CoreResourceThreadPool>,
    write_pool: Arc<CoreResourceThreadPool>,
}

impl HeedEngine {
    pub fn new(
        base_dir: &Path,
        db_file_name: &Path,
        thread_pool: Arc<CoreResourceThreadPool>,
    ) -> Self {
        let mut db_dir = PathBuf::new();
        db_dir.push(base_dir);
        db_dir.push(db_file_name);

        std::fs::create_dir_all(&db_dir).expect("Could not create OS directory for idb");
        // FIXME:(arihant2math) gracefully handle errors like hitting max dbs
        #[allow(unsafe_code)]
        let env = unsafe {
            EnvOpenOptions::new()
                .max_dbs(1024)
                .open(db_dir)
                .expect("Failed to open db_dir")
        };
        Self {
            heed_env: Arc::new(env),
            open_stores: Arc::new(RwLock::new(HashMap::new())),
            read_pool: thread_pool.clone(),
            write_pool: thread_pool,
        }
    }
}

impl KvsEngine for HeedEngine {
    type Error = heed::Error;

    fn create_store(&self, store_name: SanitizedName, auto_increment: bool) -> heed::Result<()> {
        let mut write_txn = self.heed_env.write_txn()?;
        let _ = self.heed_env.clear_stale_readers();
        let new_store: HeedDatabase = self
            .heed_env
            .create_database(&mut write_txn, Some(&*store_name.to_string()))?;

        write_txn.commit()?;

        let key_generator = { if auto_increment { Some(0) } else { None } };

        let store = Store {
            inner: new_store,
            key_generator,
        };

        self.open_stores
            .write()
            .expect("Could not acquire lock on stores")
            .insert(store_name, store);
        Ok(())
    }

    fn delete_store(&self, store_name: SanitizedName) -> heed::Result<()> {
        // TODO: Actually delete store instead of just clearing it
        let mut write_txn = self.heed_env.write_txn()?;
        let store: HeedDatabase = self
            .heed_env
            .create_database(&mut write_txn, Some(&*store_name.to_string()))?;
        store.clear(&mut write_txn)?;
        write_txn.commit()?;

        let mut open_stores = self.open_stores.write().unwrap();
        open_stores.retain(|key, _| key != &store_name);
        Ok(())
    }

    fn close_store(&self, store_name: SanitizedName) -> heed::Result<()> {
        // FIXME: (arihant2math) unused
        // FIXME:(arihant2math) return error if no store ...
        let mut open_stores = self.open_stores.write().unwrap();
        open_stores.retain(|key, _| key != &store_name);
        Ok(())
    }

    // Starts a transaction, processes all operations for that transaction,
    // and commits the changes.
    fn process_transaction(
        &self,
        transaction: KvsTransaction,
    ) -> oneshot::Receiver<Option<Vec<u8>>> {
        // This executes in a thread pool, and `readwrite` transactions
        // will block their thread if the writer is occupied, so we can
        // probably do some smart things here in order to optimize.
        // Queueing 8 writers will for example block 7 threads,
        // so write operations are reserved for just one thread,
        // so that the rest of the threads can work in parallel with read txns.
        let heed_env = self.heed_env.clone();
        let stores = self.open_stores.clone();

        let (tx, rx) = oneshot::channel();
        if let IndexedDBTxnMode::Readonly = transaction.mode {
            self.read_pool.spawn(move || {
                let env = heed_env;
                let rtxn = env.read_txn().expect("Could not create idb store reader");
                let mut results = vec![];
                for request in transaction.requests {
                    match request.operation {
                        AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem(key)) => {
                            let key: Vec<u8> = bincode::serialize(&key).unwrap();
                            let stores = stores
                                .read()
                                .expect("Could not acquire read lock on stores");
                            let store = stores
                                .get(&request.store_name)
                                .expect("Could not get store");
                            let result = store.inner.get(&rtxn, &key).expect("Could not get item");

                            if let Some(blob) = result {
                                results
                                    .push((request.sender, Some(IdbResult::Data(blob.to_vec()))));
                            } else {
                                results.push((request.sender, None));
                            }
                        },
                        AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count(key)) => {
                            let _key: Vec<u8> = bincode::serialize(&key).unwrap();
                            let stores = stores
                                .read()
                                .expect("Could not acquire read lock on stores");
                            let _store = stores
                                .get(&request.store_name)
                                .expect("Could not get store");
                            // FIXME:(arihant2math) Return count with sender
                        },
                        AsyncOperation::ReadWrite(..) => {
                            // We cannot reach this, as checks are made earlier so that
                            // no modifying requests are executed on readonly transactions
                            unreachable!(
                                "Cannot execute modifying request with readonly transactions"
                            );
                        },
                    }
                }

                if tx.send(None).is_err() {
                    warn!("IDBTransaction's execution channel is dropped");
                };

                if let Err(e) = rtxn.commit() {
                    warn!("Error committing transaction: {:?}", e);
                    for (sender, _) in results {
                        let _ = sender.send(Err(()));
                    }
                } else {
                    for (sender, result) in results {
                        let _ = sender.send(Ok(result));
                    }
                }
            });
        } else {
            self.write_pool.spawn(move || {
                // Acquiring a writer will block the thread if another `readwrite` transaction is active
                let env = heed_env;
                let mut wtxn = env.write_txn().expect("Could not create idb store writer");
                let mut results = vec![];
                for request in transaction.requests {
                    match request.operation {
                        AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem(
                            key,
                            value,
                            overwrite,
                        )) => {
                            let serialized_key: Vec<u8> = bincode::serialize(&key).unwrap();
                            let stores = stores
                                .write()
                                .expect("Could not acquire write lock on stores");
                            let store = stores
                                .get(&request.store_name)
                                .expect("Could not get store");
                            if overwrite ||
                                store
                                    .inner
                                    .get(&wtxn, &serialized_key)
                                    .expect("Could not get item")
                                    .is_none()
                            {
                                let result = store
                                    .inner
                                    .put(&mut wtxn, &serialized_key, &value)
                                    .ok()
                                    .and(Some(IdbResult::Key(key)));
                                results.push((request.sender, result));
                            } else {
                                results.push((request.sender, None));
                            }
                        },
                        AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem(key)) => {
                            let key: Vec<u8> = bincode::serialize(&key).unwrap();
                            let stores = stores
                                .read()
                                .expect("Could not acquire write lock on stores");
                            let store = stores
                                .get(&request.store_name)
                                .expect("Could not get store");
                            let result = store.inner.get(&wtxn, &key).expect("Could not get item");

                            results.push((
                                request.sender,
                                result.map(|blob| IdbResult::Data(blob.to_vec())),
                            ));
                        },
                        AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem(key)) => {
                            let serialized_key: Vec<u8> = bincode::serialize(&key).unwrap();
                            let stores = stores
                                .write()
                                .expect("Could not acquire write lock on stores");
                            let store = stores
                                .get(&request.store_name)
                                .expect("Could not get store");
                            let result = store
                                .inner
                                .delete(&mut wtxn, &serialized_key)
                                .ok()
                                .and(Some(IdbResult::Key(key)));
                            results.push((request.sender, result));
                        },
                        AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count(key)) => {
                            let _key: Vec<u8> = bincode::serialize(&key).unwrap();
                            let stores = stores
                                .read()
                                .expect("Could not acquire read lock on stores");
                            let _store = stores
                                .get(&request.store_name)
                                .expect("Could not get store");
                            // FIXME:(arihant2math) Return count with sender
                        },
                        AsyncOperation::ReadWrite(AsyncReadWriteOperation::Clear) => {
                            let stores = stores
                                .write()
                                .expect("Could not acquire write lock on stores");
                            let store = stores
                                .get(&request.store_name)
                                .expect("Could not get store");
                            // FIXME:(arihant2math) Error handling
                            let _ = store.inner.clear(&mut wtxn);
                        },
                    }
                }

                if let Err(e) = wtxn.commit() {
                    warn!("Error committing to database: {:?}", e);
                    for (sender, _) in results {
                        let _ = sender.send(Err(()));
                    }
                } else {
                    for (sender, result) in results {
                        let _ = sender.send(Ok(result));
                    }
                }
            })
        }
        rx
    }

    fn has_key_generator(&self, store_name: SanitizedName) -> bool {
        let has_generator = self
            .open_stores
            .read()
            .expect("Could not acquire read lock on stores")
            .get(&store_name)
            .expect("Store not found")
            .key_generator
            .is_some();
        has_generator
    }
}
