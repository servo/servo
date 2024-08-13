/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use log::warn;
use net_traits::indexeddb_thread::{AsyncOperation, IndexedDBKeyType, IndexedDBTxnMode};
use rkv::{Manager, Rkv, SingleStore, StoreOptions, Value};
use tokio::sync::oneshot;

use super::{KvsEngine, KvsTransaction, SanitizedName};

// A simple store that also has a key generator that can be used if no key
// is provided for the stored objects
#[derive(Clone)]
struct Store {
    inner: SingleStore,
    key_generator: Option<u64>, // https://www.w3.org/TR/IndexedDB-2/#key-generator
}

pub struct RkvEngine {
    rkv_handle: Arc<RwLock<Rkv>>,
    open_stores: Arc<RwLock<HashMap<SanitizedName, Store>>>,
    pool: rayon::ThreadPool,
}

impl RkvEngine {
    pub fn new(base_dir: &Path, db_file_name: &Path) -> Self {
        let mut db_dir = PathBuf::new();
        db_dir.push(base_dir);
        db_dir.push(db_file_name);

        std::fs::create_dir_all(&db_dir).expect("Could not create OS directory for idb");
        let rkv_handle = Manager::singleton()
            .write()
            .expect("Could not get write lock")
            .get_or_create(db_dir.as_path(), Rkv::new)
            .expect("Could not create database with this origin");

        // FIXME:(rasviitanen) What is a reasonable number of threads?
        RkvEngine {
            rkv_handle,
            open_stores: Arc::new(RwLock::new(HashMap::new())),
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(16)
                .build()
                .expect("Could not create IDBTransaction thread pool"),
        }
    }
}

impl KvsEngine for RkvEngine {
    fn create_store(&self, store_name: SanitizedName, auto_increment: bool) {
        let rkv = self.rkv_handle.read().unwrap();
        let new_store = rkv
            .open_single(&*store_name.to_string(), StoreOptions::create())
            .unwrap();

        let key_generator = {
            if auto_increment {
                Some(0)
            } else {
                None
            }
        };

        let store = Store {
            inner: new_store,
            key_generator,
        };

        self.open_stores
            .write()
            .expect("Could not aquire lock")
            .insert(store_name, store);
    }

    fn has_key_generator(&self, store_name: SanitizedName) -> bool {
        let stores = self
            .open_stores
            .read()
            .expect("Could not aquire read lock on stores");

        stores
            .get(&store_name)
            .expect("Store not found")
            .key_generator
            .is_some()
    }

    // Starts a transaction, processes all operations for that transaction,
    // and commits the changes.
    fn process_transaction(
        &self,
        transaction: KvsTransaction,
    ) -> oneshot::Receiver<Option<Vec<u8>>> {
        let db_handle = self.rkv_handle.clone();
        let stores = self.open_stores.clone();

        let (tx, rx) = oneshot::channel();
        self.pool.spawn(move || {
            let db = db_handle
                .read()
                .expect("Could not aquire read lock on idb handle");
            let stores = stores.read().expect("Could not aquire read lock on stores");
            if let IndexedDBTxnMode::Readonly = transaction.mode {
                let reader = db.read().expect("Could not create reader for idb");
                for request in transaction.requests {
                    match request.operation {
                        AsyncOperation::GetItem(key) => {
                            let store = stores
                                .get(&request.store_name)
                                .expect("Could not get store");
                            let result = store.inner.get(&reader, key).expect("Could not get item");

                            if let Some(Value::Blob(blob)) = result {
                                request.sender.send(Some(blob.to_vec())).unwrap();
                            } else {
                                request.sender.send(None).unwrap();
                            }
                        },
                        _ => {
                            // We cannot reach this, as checks are made earlier so that
                            // no modifying requests are executed on readonly transactions
                            unreachable!(
                                "Cannot execute modifying request with readonly transactions"
                            );
                        },
                    }
                }
            } else {
                // Aquiring a writer will block the thread if another `readwrite` transaction is active
                let mut writer = db.write().expect("Could not create writer for idb");
                for request in transaction.requests {
                    match request.operation {
                        AsyncOperation::PutItem(key, value, overwrite) => {
                            let store = stores
                                .get(&request.store_name)
                                .expect("Could not get store");
                            let key = match key {
                                IndexedDBKeyType::String(inner) => inner,
                                IndexedDBKeyType::Number(inner) => inner,
                                IndexedDBKeyType::Binary(inner) => inner,
                            };
                            if overwrite {
                                let result = store
                                    .inner
                                    .put(&mut writer, key.clone(), &Value::Blob(&value))
                                    .ok()
                                    .and(Some(key));
                                request.sender.send(result).unwrap();
                            } else {
                                // FIXME:(rasviitanen) We should be able to set some flags in
                                // `rkv` in order to do this without running a get request first
                                if store
                                    .inner
                                    .get(&writer, key.clone())
                                    .expect("Could not get item")
                                    .is_none()
                                {
                                    let result = store
                                        .inner
                                        .put(&mut writer, key.clone(), &Value::Blob(&value))
                                        .ok()
                                        .and(Some(key));
                                    request.sender.send(result).unwrap();
                                } else {
                                    request.sender.send(None).unwrap();
                                }
                            }
                        },
                        AsyncOperation::GetItem(key) => {
                            let store = stores
                                .get(&request.store_name)
                                .expect("Could not get store");
                            let result = store.inner.get(&writer, key).expect("Could not get item");

                            if let Some(Value::Blob(blob)) = result {
                                request.sender.send(Some(blob.to_vec())).unwrap();
                            } else {
                                request.sender.send(None).unwrap();
                            }
                        },
                        AsyncOperation::RemoveItem(key) => {
                            let store = stores
                                .get(&request.store_name)
                                .expect("Could not get store");
                            let result = store
                                .inner
                                .delete(&mut writer, key.clone())
                                .ok()
                                .and(Some(key));
                            request.sender.send(result).unwrap();
                        },
                    }
                }

                writer.commit().expect("Failed to commit to database");
            }

            if tx.send(None).is_err() {
                warn!("IDBTransaction's execution channel is dropped");
            };
        });

        rx
    }
}
