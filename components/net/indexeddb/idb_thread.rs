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
    AsyncOperation, BackendError, BackendResult, CreateObjectResult, DbResult, IndexedDBThreadMsg,
    IndexedDBTxnMode, KeyPath, SyncOperation,
};
use rustc_hash::FxHashMap;
use servo_config::pref;
use servo_url::origin::ImmutableOrigin;
use uuid::Uuid;

use crate::indexeddb::engines::{KvsEngine, KvsOperation, KvsTransaction, SqliteEngine};
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
    pub origin: ImmutableOrigin,
    pub name: String,
}

impl IndexedDBDescription {
    // randomly generated namespace for our purposes
    const NAMESPACE_SERVO_IDB: &uuid::Uuid = &Uuid::from_bytes([
        0x37, 0x9e, 0x56, 0xb0, 0x1a, 0x76, 0x44, 0xc2, 0xa0, 0xdb, 0xe2, 0x18, 0xc5, 0xc8, 0xa3,
        0x5d,
    ]);
    // Converts the database description to a folder name where all
    // data for this database is stored
    pub(super) fn as_path(&self) -> PathBuf {
        let mut path = PathBuf::new();

        // uuid v5 is deterministic
        let origin_uuid = Uuid::new_v5(
            Self::NAMESPACE_SERVO_IDB,
            self.origin.ascii_serialization().as_bytes(),
        );
        let db_name_uuid = Uuid::new_v5(Self::NAMESPACE_SERVO_IDB, self.name.as_bytes());
        path.push(origin_uuid.to_string());
        path.push(db_name_uuid.to_string());

        path
    }
}

struct IndexedDBEnvironment<E: KvsEngine> {
    engine: E,
    transactions: FxHashMap<u64, KvsTransaction>,
    serial_number_counter: u64,
}

impl<E: KvsEngine> IndexedDBEnvironment<E> {
    fn new(engine: E) -> IndexedDBEnvironment<E> {
        IndexedDBEnvironment {
            engine,
            transactions: FxHashMap::default(),
            serial_number_counter: 0,
        }
    }

    fn queue_operation(
        &mut self,
        store_name: &str,
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
                operation,
                store_name: String::from(store_name),
            });
    }

    // Executes all requests for a transaction (without committing)
    fn start_transaction(&mut self, txn: u64, sender: Option<IpcSender<BackendResult<()>>>) {
        // FIXME:(arihant2math) find optimizations in this function
        //   rather than on the engine level code (less repetition)
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

    fn has_key_generator(&self, store_name: &str) -> bool {
        self.engine.has_key_generator(store_name)
    }

    fn key_path(&self, store_name: &str) -> Option<KeyPath> {
        self.engine.key_path(store_name)
    }

    fn create_index(
        &self,
        store_name: &str,
        index_name: String,
        key_path: KeyPath,
        unique: bool,
        multi_entry: bool,
    ) -> DbResult<CreateObjectResult> {
        self.engine
            .create_index(store_name, index_name, key_path, unique, multi_entry)
            .map_err(|err| format!("{err:?}"))
    }

    fn delete_index(&self, store_name: &str, index_name: String) -> DbResult<()> {
        self.engine
            .delete_index(store_name, index_name)
            .map_err(|err| format!("{err:?}"))
    }

    fn create_object_store(
        &mut self,
        store_name: &str,
        key_path: Option<KeyPath>,
        auto_increment: bool,
    ) -> DbResult<CreateObjectResult> {
        self.engine
            .create_store(store_name, key_path, auto_increment)
            .map_err(|err| format!("{err:?}"))
    }

    fn delete_object_store(&mut self, store_name: &str) -> DbResult<()> {
        let result = self.engine.delete_store(store_name);
        result.map_err(|err| format!("{err:?}"))
    }

    fn delete_database(self, sender: IpcSender<BackendResult<()>>) {
        let result = self.engine.delete_database();
        let _ = sender.send(
            result
                .map_err(|err| format!("{err:?}"))
                .map_err(BackendError::from),
        );
    }

    fn version(&self) -> DbResult<u64> {
        self.engine.version().map_err(|err| format!("{err:?}"))
    }

    fn set_version(&mut self, version: u64) -> DbResult<()> {
        self.engine
            .set_version(version)
            .map_err(|err| format!("{err:?}"))
    }
}

struct IndexedDBManager {
    port: IpcReceiver<IndexedDBThreadMsg>,
    idb_base_dir: PathBuf,
    databases: HashMap<IndexedDBDescription, IndexedDBEnvironment<SqliteEngine>>,
    thread_pool: Arc<CoreResourceThreadPool>,
}

impl IndexedDBManager {
    fn new(port: IpcReceiver<IndexedDBThreadMsg>, idb_base_dir: PathBuf) -> IndexedDBManager {
        debug!("New indexedDBManager");

        // Uses an estimate of the system cpus to process IndexedDB transactions
        // See https://doc.rust-lang.org/stable/std/thread/fn.available_parallelism.html
        // If no information can be obtained about the system, uses 4 threads as a default
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
                IndexedDBThreadMsg::Async(origin, db_name, store_name, txn, mode, operation) => {
                    if let Some(db) = self.get_database_mut(origin, db_name) {
                        // Queues an operation for a transaction without starting it
                        db.queue_operation(&store_name, txn, mode, operation);
                        // FIXME:(arihant2math) Schedule transactions properly
                        // while db.transactions.iter().any(|s| s.1.mode == IndexedDBTxnMode::Readwrite) {
                        //     std::hint::spin_loop();
                        // }
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
    ) -> Option<&IndexedDBEnvironment<SqliteEngine>> {
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
    ) -> Option<&mut IndexedDBEnvironment<SqliteEngine>> {
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
                    // TODO: maybe a close database function should be added to the trait and called here?
                }
                let _ = sender.send(Ok(()));
            },
            SyncOperation::OpenDatabase(sender, origin, db_name, version) => {
                let idb_description = IndexedDBDescription {
                    origin,
                    name: db_name,
                };

                let idb_base_dir = self.idb_base_dir.as_path();

                let version = version.unwrap_or(0);

                match self.databases.entry(idb_description.clone()) {
                    Entry::Vacant(e) => {
                        let db = IndexedDBEnvironment::new(
                            SqliteEngine::new(
                                idb_base_dir,
                                &idb_description,
                                self.thread_pool.clone(),
                            )
                            .expect("Failed to create sqlite engine"),
                        );
                        let _ = sender.send(db.version().unwrap_or(version));
                        e.insert(db);
                    },
                    Entry::Occupied(db) => {
                        let _ = sender.send(db.get().version().unwrap_or(version));
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
                if let Some(db) = self.databases.remove(&idb_description) {
                    db.delete_database(sender);
                } else {
                    let _ = sender.send(Ok(()));
                }
            },
            SyncOperation::HasKeyGenerator(sender, origin, db_name, store_name) => {
                let result = self
                    .get_database(origin, db_name)
                    .map(|db| db.has_key_generator(&store_name));
                let _ = sender.send(result.ok_or(BackendError::DbNotFound));
            },
            SyncOperation::KeyPath(sender, origin, db_name, store_name) => {
                let result = self
                    .get_database(origin, db_name)
                    .map(|db| db.key_path(&store_name));
                let _ = sender.send(result.ok_or(BackendError::DbNotFound));
            },
            SyncOperation::CreateIndex(
                sender,
                origin,
                db_name,
                store_name,
                index_name,
                key_path,
                unique,
                multi_entry,
            ) => {
                if let Some(db) = self.get_database(origin, db_name) {
                    let result =
                        db.create_index(&store_name, index_name, key_path, unique, multi_entry);
                    let _ = sender.send(result.map_err(BackendError::from));
                } else {
                    let _ = sender.send(Err(BackendError::DbNotFound));
                }
            },
            SyncOperation::DeleteIndex(sender, origin, db_name, store_name, index_name) => {
                if let Some(db) = self.get_database(origin, db_name) {
                    let result = db.delete_index(&store_name, index_name);
                    let _ = sender.send(result.map_err(BackendError::from));
                } else {
                    let _ = sender.send(Err(BackendError::DbNotFound));
                }
            },
            SyncOperation::Commit(sender, _origin, _db_name, _txn) => {
                // FIXME:(arihant2math) This does nothing at the moment
                let _ = sender.send(Ok(()));
            },
            SyncOperation::UpgradeVersion(sender, origin, db_name, _txn, version) => {
                if let Some(db) = self.get_database_mut(origin, db_name) {
                    if version > db.version().unwrap_or(0) {
                        let _ = db.set_version(version);
                    }
                    // erroring out if the version is not upgraded can be and non-replicable
                    let _ = sender.send(db.version().map_err(BackendError::from));
                } else {
                    let _ = sender.send(Err(BackendError::DbNotFound));
                }
            },
            SyncOperation::CreateObjectStore(
                sender,
                origin,
                db_name,
                store_name,
                key_paths,
                auto_increment,
            ) => {
                if let Some(db) = self.get_database_mut(origin, db_name) {
                    let result = db.create_object_store(&store_name, key_paths, auto_increment);
                    let _ = sender.send(result.map_err(BackendError::from));
                } else {
                    let _ = sender.send(Err(BackendError::DbNotFound));
                }
            },
            SyncOperation::DeleteObjectStore(sender, origin, db_name, store_name) => {
                if let Some(db) = self.get_database_mut(origin, db_name) {
                    let result = db.delete_object_store(&store_name);
                    let _ = sender.send(result.map_err(BackendError::from));
                } else {
                    let _ = sender.send(Err(BackendError::DbNotFound));
                }
            },
            SyncOperation::StartTransaction(sender, origin, db_name, txn) => {
                if let Some(db) = self.get_database_mut(origin, db_name) {
                    db.start_transaction(txn, Some(sender));
                } else {
                    let _ = sender.send(Err(BackendError::DbNotFound));
                }
            },
            SyncOperation::Version(sender, origin, db_name) => {
                if let Some(db) = self.get_database(origin, db_name) {
                    let _ = sender.send(db.version().map_err(BackendError::from));
                } else {
                    let _ = sender.send(Err(BackendError::DbNotFound));
                }
            },
            SyncOperation::RegisterNewTxn(sender, origin, db_name) => {
                if let Some(db) = self.get_database_mut(origin, db_name) {
                    db.serial_number_counter += 1;
                    let _ = sender.send(Ok(db.serial_number_counter));
                } else {
                    let _ = sender.send(Err(BackendError::DbNotFound));
                }
            },
            SyncOperation::Exit(sender) => {
                // FIXME:(rasviitanen) Nothing to do?
                let _ = sender.send(());
            },
        }
    }
}
