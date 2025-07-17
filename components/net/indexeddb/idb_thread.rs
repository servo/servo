/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::borrow::ToOwned;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use ipc_channel::ipc::{self, IpcError, IpcReceiver, IpcSender};
use log::{debug, warn};
use net_traits::indexeddb_thread::{
    AsyncOperation, IdbResult, IndexedDBThreadMsg, IndexedDBTxnMode, SyncOperation,
};
use servo_config::pref;
use servo_url::origin::ImmutableOrigin;

use crate::indexeddb::engines::{
    HeedEngine, KvsEngine, KvsOperation, KvsTransaction, SanitizedName,
};
use crate::resource_thread::CoreResourceThreadPool;

pub trait IndexedDBThreadFactory {
    fn new(config_dir: Option<PathBuf>) -> Self;
}

impl IndexedDBThreadFactory for IpcSender<IndexedDBThreadMsg> {
    fn new(config_dir: Option<PathBuf>) -> IpcSender<IndexedDBThreadMsg> {
        let (chan, port) = ipc::channel().unwrap();

        let mut idb_base_dir = PathBuf::new();
        if let Some(p) = config_dir {
            idb_base_dir.push(p);
        }
        idb_base_dir.push("IndexedDB");

        thread::Builder::new()
            .name("IndexedDBManager".to_owned())
            .spawn(move || {
                IndexedDBManager::new(port, idb_base_dir).start();
            })
            .expect("Thread spawning failed");

        chan
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct IndexedDBDescription {
    origin: ImmutableOrigin,
    name: String,
}

impl IndexedDBDescription {
    // Converts the database description to a folder name where all
    // data for this database is stored
    fn as_path(&self) -> PathBuf {
        let mut path = PathBuf::new();

        let sanitized_origin = SanitizedName::new(self.origin.ascii_serialization());
        let sanitized_name = SanitizedName::new(self.name.clone());
        path.push(sanitized_origin.to_string());
        path.push(sanitized_name.to_string());

        path
    }
}

struct IndexedDBEnvironment<E: KvsEngine> {
    engine: E,
    version: u64,

    transactions: HashMap<u64, KvsTransaction>,
    serial_number_counter: u64,
}

impl<E: KvsEngine> IndexedDBEnvironment<E> {
    fn new(engine: E, version: u64) -> IndexedDBEnvironment<E> {
        IndexedDBEnvironment {
            engine,
            version,

            transactions: HashMap::new(),
            serial_number_counter: 0,
        }
    }

    fn queue_operation(
        &mut self,
        sender: IpcSender<Result<Option<IdbResult>, ()>>,
        store_name: SanitizedName,
        serial_number: u64,
        mode: IndexedDBTxnMode,
        operation: AsyncOperation,
    ) {
        self.transactions
            .entry(serial_number)
            .or_insert_with(|| KvsTransaction {
                requests: VecDeque::new(),
                mode,
            })
            .requests
            .push_back(KvsOperation {
                sender,
                operation,
                store_name,
            });
    }

    // Executes all requests for a transaction (without committing)
    fn start_transaction(&mut self, txn: u64, sender: Option<IpcSender<Result<(), ()>>>) {
        // FIXME:(arihant2math) find a way to optimizations in this function
        // rather than on the engine level code (less repetition)
        if let Some(txn) = self.transactions.remove(&txn) {
            let _ = self.engine.process_transaction(txn).blocking_recv();
        }

        // We have a sender if the transaction is started manually, and they
        // probably want to know when it is finished
        if let Some(sender) = sender {
            if sender.send(Ok(())).is_err() {
                warn!("IDBTransaction starter dropped its channel");
            }
        }
    }

    fn has_key_generator(&self, store_name: SanitizedName) -> bool {
        self.engine.has_key_generator(store_name)
    }

    fn create_object_store(
        &mut self,
        sender: IpcSender<Result<(), ()>>,
        store_name: SanitizedName,
        auto_increment: bool,
    ) {
        let result = self.engine.create_store(store_name, auto_increment);

        if result.is_ok() {
            let _ = sender.send(Ok(()));
        } else {
            let _ = sender.send(Err(()));
        }
    }

    fn delete_object_store(
        &mut self,
        sender: IpcSender<Result<(), ()>>,
        store_name: SanitizedName,
    ) {
        let result = self.engine.delete_store(store_name);

        if result.is_ok() {
            let _ = sender.send(Ok(()));
        } else {
            let _ = sender.send(Err(()));
        }
    }
}

struct IndexedDBManager {
    port: IpcReceiver<IndexedDBThreadMsg>,
    idb_base_dir: PathBuf,
    databases: HashMap<IndexedDBDescription, IndexedDBEnvironment<HeedEngine>>,
    thread_pool: Arc<CoreResourceThreadPool>,
}

impl IndexedDBManager {
    fn new(port: IpcReceiver<IndexedDBThreadMsg>, idb_base_dir: PathBuf) -> IndexedDBManager {
        debug!("New indexedDBManager");

        let thread_count = thread::available_parallelism()
            .map(|i| i.get())
            .unwrap_or(pref!(threadpools_fallback_worker_num) as usize)
            .min(pref!(threadpools_indexeddb_workers_max).max(1) as usize);
        IndexedDBManager {
            port,
            idb_base_dir,
            databases: HashMap::new(),
            thread_pool: Arc::new(CoreResourceThreadPool::new(
                thread_count,
                "IndexedDB".to_string(),
            )),
        }
    }
}

impl IndexedDBManager {
    fn start(&mut self) {
        if !pref!(dom_indexeddb_enabled) {
            return;
        }
        loop {
            // FIXME:(arihant2math) No message *most likely* means that
            // the ipc sender has been dropped, so we break the look
            let message = match self.port.recv() {
                Ok(msg) => msg,
                Err(e) => match e {
                    IpcError::Disconnected => {
                        break;
                    },
                    other => {
                        warn!("Error in IndexedDB thread: {:?}", other);
                        continue;
                    },
                },
            };
            match message {
                IndexedDBThreadMsg::Sync(operation) => {
                    self.handle_sync_operation(operation);
                },
                IndexedDBThreadMsg::Async(
                    sender,
                    origin,
                    db_name,
                    store_name,
                    txn,
                    mode,
                    operation,
                ) => {
                    let store_name = SanitizedName::new(store_name);
                    if let Some(db) = self.get_database_mut(origin, db_name) {
                        // Queues an operation for a transaction without starting it
                        db.queue_operation(sender, store_name, txn, mode, operation);
                        // FIXME:(arihant2math) Schedule transactions properly:
                        // for now, we start them directly.
                        db.start_transaction(txn, None);
                    }
                },
            }
        }
    }

    fn get_database(
        &self,
        origin: ImmutableOrigin,
        db_name: String,
    ) -> Option<&IndexedDBEnvironment<HeedEngine>> {
        let idb_description = IndexedDBDescription {
            origin,
            name: db_name,
        };

        self.databases.get(&idb_description)
    }

    fn get_database_mut(
        &mut self,
        origin: ImmutableOrigin,
        db_name: String,
    ) -> Option<&mut IndexedDBEnvironment<HeedEngine>> {
        let idb_description = IndexedDBDescription {
            origin,
            name: db_name,
        };

        self.databases.get_mut(&idb_description)
    }

    fn handle_sync_operation(&mut self, operation: SyncOperation) {
        match operation {
            SyncOperation::CloseDatabase(sender, origin, db_name) => {
                let idb_description = IndexedDBDescription {
                    origin,
                    name: db_name,
                };
                if let Some(_db) = self.databases.remove(&idb_description) {
                    // TODO: maybe close store here?
                }
                let _ = sender.send(Ok(()));
            },
            SyncOperation::OpenDatabase(sender, origin, db_name, version) => {
                let idb_description = IndexedDBDescription {
                    origin,
                    name: db_name,
                };

                let idb_base_dir = self.idb_base_dir.as_path();

                match self.databases.entry(idb_description.clone()) {
                    Entry::Vacant(e) => {
                        let db = IndexedDBEnvironment::new(
                            HeedEngine::new(
                                idb_base_dir,
                                &idb_description.as_path(),
                                self.thread_pool.clone(),
                            ),
                            version.unwrap_or(0),
                        );
                        let _ = sender.send(db.version);
                        e.insert(db);
                    },
                    Entry::Occupied(db) => {
                        let _ = sender.send(db.get().version);
                    },
                }
            },
            SyncOperation::DeleteDatabase(sender, origin, db_name) => {
                // https://w3c.github.io/IndexedDB/#delete-a-database
                // Step 4. Let db be the database named name in storageKey,
                // if one exists. Otherwise, return 0 (zero).
                let idb_description = IndexedDBDescription {
                    origin,
                    name: db_name,
                };
                if self.databases.remove(&idb_description).is_none() {
                    let _ = sender.send(Ok(()));
                    return;
                }

                // FIXME:(rasviitanen) Possible security issue?
                // FIXME:(arihant2math) using remove_dir_all with arbitrary input ...
                let mut db_dir = self.idb_base_dir.clone();
                db_dir.push(idb_description.as_path());
                if std::fs::remove_dir_all(&db_dir).is_err() {
                    let _ = sender.send(Err(()));
                } else {
                    let _ = sender.send(Ok(()));
                }
            },
            SyncOperation::HasKeyGenerator(sender, origin, db_name, store_name) => {
                let store_name = SanitizedName::new(store_name);
                let result = self
                    .get_database(origin, db_name)
                    .map(|db| db.has_key_generator(store_name))
                    .expect("No Database");
                sender.send(result).expect("Could not send generator info");
            },
            SyncOperation::Commit(sender, _origin, _db_name, _txn) => {
                // FIXME:(arihant2math) This does nothing at the moment
                sender.send(Err(())).expect("Could not send commit status");
            },
            SyncOperation::UpgradeVersion(sender, origin, db_name, _txn, version) => {
                if let Some(db) = self.get_database_mut(origin, db_name) {
                    if version > db.version {
                        db.version = version;
                    }
                    // erroring out if the version is not upgraded can be and non-replicable
                    let _ = sender.send(Ok(db.version));
                } else {
                    let _ = sender.send(Err(()));
                }
            },
            SyncOperation::CreateObjectStore(
                sender,
                origin,
                db_name,
                store_name,
                auto_increment,
            ) => {
                let store_name = SanitizedName::new(store_name);
                if let Some(db) = self.get_database_mut(origin, db_name) {
                    db.create_object_store(sender, store_name, auto_increment);
                }
            },
            SyncOperation::DeleteObjectStore(sender, origin, db_name, store_name) => {
                let store_name = SanitizedName::new(store_name);
                if let Some(db) = self.get_database_mut(origin, db_name) {
                    db.delete_object_store(sender, store_name);
                }
            },
            SyncOperation::StartTransaction(sender, origin, db_name, txn) => {
                if let Some(db) = self.get_database_mut(origin, db_name) {
                    db.start_transaction(txn, Some(sender));
                };
            },
            SyncOperation::Version(sender, origin, db_name) => {
                if let Some(db) = self.get_database(origin, db_name) {
                    let _ = sender.send(db.version);
                };
            },
            SyncOperation::RegisterNewTxn(sender, origin, db_name) => {
                if let Some(db) = self.get_database_mut(origin, db_name) {
                    db.serial_number_counter += 1;
                    let _ = sender.send(db.serial_number_counter);
                }
            },
            SyncOperation::Exit(sender) => {
                // FIXME:(rasviitanen) Nothing to do?
                let _ = sender.send(());
            },
        }
    }
}
