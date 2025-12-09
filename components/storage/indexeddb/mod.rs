/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod engines;

use std::borrow::ToOwned;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use base::generic_channel::{self, GenericReceiver, GenericSender, ReceiveError};
use base::threadpool::ThreadPool;
use log::{debug, error, warn};
use profile_traits::generic_callback::GenericCallback;
use rustc_hash::FxHashMap;
use servo_config::pref;
use servo_url::origin::ImmutableOrigin;
use storage_traits::indexeddb::{
    AsyncOperation, BackendError, BackendResult, CreateObjectResult, DbResult, IndexedDBThreadMsg,
    IndexedDBTxnMode, KeyPath, OpenDatabaseResult, SyncOperation,
};
use uuid::Uuid;

use crate::indexeddb::engines::{KvsEngine, KvsOperation, KvsTransaction, SqliteEngine};

pub trait IndexedDBThreadFactory {
    fn new(config_dir: Option<PathBuf>) -> Self;
}

impl IndexedDBThreadFactory for GenericSender<IndexedDBThreadMsg> {
    fn new(config_dir: Option<PathBuf>) -> GenericSender<IndexedDBThreadMsg> {
        let (chan, port) = generic_channel::channel().unwrap();

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
}

impl<E: KvsEngine> IndexedDBEnvironment<E> {
    fn new(engine: E) -> IndexedDBEnvironment<E> {
        IndexedDBEnvironment {
            engine,
            transactions: FxHashMap::default(),
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
    fn start_transaction(&mut self, txn: u64, sender: Option<GenericSender<BackendResult<()>>>) {
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

    fn delete_database(self, sender: GenericCallback<BackendResult<()>>) {
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

/// Keeping track of pending upgrade transactions.
/// TODO: move into general transaction lifecycle.
struct PendingUpgrade {
    sender: GenericCallback<OpenDatabaseResult>,
    db_version: u64,
}

struct IndexedDBManager {
    port: GenericReceiver<IndexedDBThreadMsg>,
    idb_base_dir: PathBuf,
    databases: HashMap<IndexedDBDescription, IndexedDBEnvironment<SqliteEngine>>,
    thread_pool: Arc<ThreadPool>,
    /// Tracking pending upgrade transactions.
    /// TODO: move into general transaction lifecyle.
    pending_upgrades: HashMap<(String, u64), PendingUpgrade>,
    /// A global counter to produce unique transaction ids.
    /// TODO: remove once db connections lifecyle is managed.
    /// A global counter is only necessary because of how deleting a db currently
    /// does not wait for connection to close and transactions to finish.
    serial_number_counter: u64,
}

impl IndexedDBManager {
    fn new(port: GenericReceiver<IndexedDBThreadMsg>, idb_base_dir: PathBuf) -> IndexedDBManager {
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
            thread_pool: Arc::new(ThreadPool::new(thread_count, "IndexedDB".to_string())),
            pending_upgrades: Default::default(),
            serial_number_counter: 0,
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
                Err(ReceiveError::Disconnected) => {
                    break;
                },
                Err(e) => {
                    warn!("Error in IndexedDB thread: {e:?}");
                    continue;
                },
            };
            match message {
                IndexedDBThreadMsg::Sync(SyncOperation::Exit(sender)) => {
                    let _ = sender.send(());
                    break;
                },
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
                IndexedDBThreadMsg::OpenTransactionInactive { name, transaction } => {
                    self.handle_open_transaction_inactive(name, transaction);
                },
            }
        }
    }

    /// Handle when an open transaction becomes inactive.
    /// TODO: transaction lifecyle.
    fn handle_open_transaction_inactive(&mut self, name: String, transaction: u64) {
        let Some(pending_upgrade) = self.pending_upgrades.remove(&(name, transaction)) else {
            error!("OpenTransactionInactive received for non-existent pending upgrade.");
            return;
        };
        if pending_upgrade
            .sender
            .send(OpenDatabaseResult::Connection {
                version: pending_upgrade.db_version,
                upgraded: true,
            })
            .is_err()
        {
            error!("Failed to send OpenDatabaseResult::Connection to script.");
        };
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

    /// <https://w3c.github.io/IndexedDB/#upgrade-a-database>
    /// To upgrade a database with connection (a connection),
    /// a new version, and a request, run these steps:
    /// TODO: connection and request.
    fn upgrade_database(
        &mut self,
        idb_description: IndexedDBDescription,
        new_version: u64,
        db_name: String,
        sender: GenericCallback<OpenDatabaseResult>,
    ) {
        // Step 1: Let db be connection’s database.
        // TODO: connection.
        let db = self
            .databases
            .get_mut(&idb_description)
            .expect("Db should have been opened.");

        // Step 2: Let transaction be a new upgrade transaction with connection used as connection.
        let transaction_id = self.serial_number_counter;
        self.serial_number_counter += 1;

        // Step 3: Set transaction’s scope to connection’s object store set.
        // Step 4: Set db’s upgrade transaction to transaction.
        // Step 5: Set transaction’s state to inactive.
        // Step 6: Start transaction.
        // TODO: implement transactions and their lifecyle.

        // Step 7: Let old version be db’s version.
        let old_version = db.version().expect("DB should have a version.");

        // Step 8: Set db’s version to version.
        // This change is considered part of the transaction,
        // and so if the transaction is aborted, this change is reverted.
        // TODO: wrap in transaction.
        db.set_version(new_version)
            .expect("Setting the version should not fail");

        // Step 9: Set request’s processed flag to true.
        // TODO: implement requests.

        // Step 10: Queue a database task to run these steps:
        if sender
            .send(OpenDatabaseResult::Upgrade {
                version: new_version,
                old_version,
                transaction: transaction_id,
            })
            .is_err()
        {
            error!("Couldn't queue task for indexeddb upgrade event.");
        }

        // Step 11: Wait for transaction to finish.
        self.pending_upgrades.insert(
            (db_name, transaction_id),
            PendingUpgrade {
                sender,
                db_version: new_version,
            },
        );
    }

    /// <https://w3c.github.io/IndexedDB/#open-a-database-connection>
    fn open_database(
        &mut self,
        sender: GenericCallback<OpenDatabaseResult>,
        origin: ImmutableOrigin,
        db_name: String,
        version: Option<u64>,
    ) {
        // Step 1: Let queue be the connection queue for storageKey and name.
        // Step 2: Add request to queue.
        // Step 3: Wait until all previous requests in queue have been processed.
        // TODO: implement #request-connection-queue
        // TODO: use a storage key.

        let idb_description = IndexedDBDescription {
            origin,
            name: db_name.clone(),
        };

        let idb_base_dir = self.idb_base_dir.as_path();

        // Step 4: Let db be the database named name in origin, or null otherwise.
        let (db_version, version) = match self.databases.entry(idb_description.clone()) {
            Entry::Vacant(e) => {
                // Step 5: If version is undefined, let version be 1 if db is null, or db’s version otherwise.
                // Note: done below with the zero as first tuple item.

                // Step 6: If db is null, let db be a new database
                // with name name, version 0 (zero), and with no object stores.
                // If this fails for any reason, return an appropriate error
                // (e.g. a "QuotaExceededError" or "UnknownError" DOMException).
                // TODO: return error.
                let (engine, created_db_path) =
                    SqliteEngine::new(idb_base_dir, &idb_description, self.thread_pool.clone())
                        .expect("Failed to create sqlite engine");
                let db = IndexedDBEnvironment::new(engine);
                let db_version = db.version().expect("DB should have a version.");

                let version = if created_db_path {
                    version.unwrap_or(1)
                } else {
                    version.unwrap_or(db_version)
                };

                e.insert(db);
                (db_version, version)
            },
            Entry::Occupied(db) => {
                let db_version = db.get().version().expect("Db should have a version.");
                // Step 5: If version is undefined, let version be 1 if db is null, or db’s version otherwise.
                (db_version, version.unwrap_or(db_version))
            },
        };

        // Step 7: If db’s version is greater than version,
        // return a newly created "VersionError" DOMException
        // and abort these steps.
        if version < db_version {
            if sender.send(OpenDatabaseResult::VersionError).is_err() {
                debug!("Script exit during indexeddb database open");
            }
            return;
        }

        // Let connection be a new connection to db.
        // Set connection’s version to version.
        // TODO: track connections in the backend(each script `IDBDatabase` should have a matching connection).

        // Step 10: If db’s version is less than version, then:
        if db_version < version {
            // Step 10.1: Let openConnections be the set of all connections,
            // except connection, associated with db.
            // Step 10.2: For each entry of openConnections
            // that does not have its close pending flag set to true,
            // queue a database task to fire a version change event
            // named versionchange at entry with db’s version and version.
            // Step 10.3: Wait for all of the events to be fired.
            // Step 10.4: If any of the connections in openConnections are still not closed,
            // queue a database task to fire a version change event named blocked
            // at request with db’s version and version.
            // Step 10.5: Wait until all connections in openConnections are closed.
            // TODO: implement connections.

            // Step 10.6: Run upgrade a database using connection, version and request.
            self.upgrade_database(idb_description, version, db_name, sender);
            return;
        }

        // Step 11:
        if sender
            .send(OpenDatabaseResult::Connection {
                version: db_version,
                upgraded: false,
            })
            .is_err()
        {
            error!("Failed to send OpenDatabaseResult::Connection to script.");
        };
    }

    fn handle_sync_operation(&mut self, operation: SyncOperation) {
        match operation {
            SyncOperation::CloseDatabase(sender, origin, db_name) => {
                // TODO: Wait for all transactions created using connection to complete.
                // Note: current behavior is as if the `forced` flag is always set.
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
                self.open_database(sender, origin, db_name, version);
            },
            SyncOperation::DeleteDatabase(callback, origin, db_name) => {
                // https://w3c.github.io/IndexedDB/#delete-a-database
                // Step 4. Let db be the database named name in storageKey,
                // if one exists. Otherwise, return 0 (zero).
                let idb_description = IndexedDBDescription {
                    origin,
                    name: db_name,
                };
                if let Some(db) = self.databases.remove(&idb_description) {
                    db.delete_database(callback);
                } else {
                    let _ = callback.send(Ok(()));
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
                let transaction_id = self.serial_number_counter;
                self.serial_number_counter += 1;
                let _ = sender.send(transaction_id);
            },
            SyncOperation::Exit(_) => {
                unreachable!("We must've already broken out of event loop.");
            },
        }
    }
}
