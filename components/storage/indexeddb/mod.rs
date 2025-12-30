/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod engines;

use std::borrow::ToOwned;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::{mem, thread};

use base::generic_channel::{self, GenericReceiver, GenericSender, ReceiveError};
use base::threadpool::ThreadPool;
use log::{debug, error, warn};
use profile_traits::generic_callback::GenericCallback;
use rusqlite::Error as RusqliteError;
use rustc_hash::FxHashMap;
use servo_config::pref;
use servo_url::origin::ImmutableOrigin;
use storage_traits::indexeddb::{
    AsyncOperation, BackendError, BackendResult, CreateObjectResult, DatabaseInfo, DbResult,
    IndexedDBThreadMsg, IndexedDBTxnMode, KeyPath, OpenDatabaseResult, SyncOperation,
};
use uuid::Uuid;

use crate::indexeddb::engines::{KvsEngine, KvsOperation, KvsTransaction, SqliteEngine};
use crate::shared::is_sqlite_disk_full_error;

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

        let chan_cloned = chan.clone();
        thread::Builder::new()
            .name("IndexedDBManager".to_owned())
            .spawn(move || {
                IndexedDBManager::new(port, chan_cloned, idb_base_dir).start();
            })
            .expect("Thread spawning failed");

        chan
    }
}

/// A key used to track databases.
/// TODO: use a storage key.
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

struct TransactionEntry {
    kvs: KvsTransaction,
    creation_order: u64,
    scope: HashSet<String>,
    finished: bool,
    started: bool,
}

struct IndexedDBEnvironment<E: KvsEngine> {
    engine: E,
    transactions: FxHashMap<u64, TransactionEntry>,
    next_creation_order: u64,
}

impl<E: KvsEngine> IndexedDBEnvironment<E> {
    fn new(engine: E) -> IndexedDBEnvironment<E> {
        IndexedDBEnvironment {
            engine,
            transactions: FxHashMap::default(),
            next_creation_order: 0,
        }
    }

    fn queue_operation(
        &mut self,
        store_name: &str,
        serial_number: u64,
        mode: IndexedDBTxnMode,
        operation: AsyncOperation,
    ) {
        let entry = self.transactions.entry(serial_number).or_insert_with(|| {
            let creation_order = self.next_creation_order;
            self.next_creation_order += 1;
            TransactionEntry {
                kvs: KvsTransaction {
                    requests: VecDeque::new(),
                    mode,
                },
                creation_order,
                scope: HashSet::new(),
                finished: false,
                started: false,
            }
        });
        entry.scope.insert(store_name.to_string());
        entry.kvs.requests.push_back(KvsOperation {
            operation,
            store_name: String::from(store_name),
        });
    }

    // Executes all requests for a transaction (without committing)
    fn start_transaction(
        &mut self,
        txn: u64,
        sender: Option<GenericSender<BackendResult<()>>>,
    ) -> bool {
        // FIXME:(arihant2math) find optimizations in this function
        //   rather than on the engine level code (less repetition)
        let Some(entry) = self.transactions.get_mut(&txn) else {
            return false;
        };
        if entry.finished {
            return false;
        }
        if entry.kvs.requests.is_empty() {
            entry.started = false;
            return false;
        }
        entry.started = true;
        let mode = entry.kvs.mode;
        let mut requests = VecDeque::new();
        mem::swap(&mut requests, &mut entry.kvs.requests);
        let kvs = KvsTransaction { mode, requests };
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // Requests must be executed in the order in which they were made against the transaction.
        let _ = self.engine.process_transaction(kvs).blocking_recv();
        entry.started = false;

        // We have a sender if the transaction is started manually, and they
        // probably want to know when it is finished
        if let Some(sender) = sender {
            if sender.send(Ok(())).is_err() {
                warn!("IDBTransaction starter dropped its channel");
            }
        }
        true
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

    fn delete_database(self) -> BackendResult<()> {
        let result = self.engine.delete_database();
        result
            .map_err(|err| format!("{err:?}"))
            .map_err(BackendError::from)
    }

    fn delete_database_now(self) -> DbResult<()> {
        self.engine
            .delete_database()
            .map_err(|err| format!("{err:?}"))
    }

    fn version(&self) -> Result<u64, E::Error> {
        self.engine.version()
    }

    fn set_version(&mut self, version: u64) -> DbResult<()> {
        self.engine
            .set_version(version)
            .map_err(|err| format!("{err:?}"))
    }

    // https://w3c.github.io/IndexedDB/#transaction-concept
    // Two transactions have overlapping scope if any object store is in both transactions' scope.
    fn scopes_overlap(a: &TransactionEntry, b: &TransactionEntry) -> bool {
        a.scope.iter().any(|store| b.scope.contains(store))
    }

    fn is_readwrite_like(mode: IndexedDBTxnMode) -> bool {
        matches!(
            mode,
            IndexedDBTxnMode::Readwrite | IndexedDBTxnMode::Versionchange
        )
    }

    // https://w3c.github.io/IndexedDB/#transaction-scheduling
    // A read-only transactions tx can start when there are no read/write transactions which:
    // Were created before tx; and
    // have overlapping scopes with tx; and
    // are not finished.
    fn can_start_readonly(&self, txn_id: u64) -> bool {
        let Some(entry) = self.transactions.get(&txn_id) else {
            return false;
        };
        self.transactions.iter().all(|(other_id, other)| {
            if *other_id == txn_id {
                return true;
            }
            if other.finished || other.creation_order >= entry.creation_order {
                return true;
            }
            if Self::is_readwrite_like(other.kvs.mode) && Self::scopes_overlap(entry, other) {
                return false;
            }
            true
        })
    }

    // https://w3c.github.io/IndexedDB/#transaction-scheduling
    // A read/write transaction tx can start when there are no transactions which:
    // Were created before tx; and
    // have overlapping scopes with tx; and
    // are not finished.
    fn can_start_readwrite(&self, txn_id: u64) -> bool {
        let Some(entry) = self.transactions.get(&txn_id) else {
            return false;
        };
        self.transactions.iter().all(|(other_id, other)| {
            if *other_id == txn_id {
                return true;
            }
            if other.finished || other.creation_order >= entry.creation_order {
                return true;
            }
            if Self::scopes_overlap(entry, other) {
                return false;
            }
            true
        })
    }

    // Scheduling happens in the backend so scope and ordering are evaluated against queued work.
    // https://w3c.github.io/IndexedDB/#transaction-scheduling
    fn schedule_transactions(
        &mut self,
        origin: ImmutableOrigin,
        db_name: String,
        sender: GenericSender<IndexedDBThreadMsg>,
    ) {
        // https://w3c.github.io/IndexedDB/#transaction-scheduling
        // If multiple read/write transactions are attempting to access the same object store
        // the transaction that was created first is the transaction which gets access, until the transaction is finished.
        let mut ordered: Vec<(u64, u64)> = self
            .transactions
            .iter()
            .filter(|(_, entry)| !entry.finished)
            .map(|(id, entry)| (*id, entry.creation_order))
            .collect();
        ordered.sort_by_key(|(_, order)| *order);

        for (txn_id, _) in ordered {
            let should_start = {
                let Some(entry) = self.transactions.get(&txn_id) else {
                    continue;
                };
                if entry.started || entry.finished {
                    continue;
                }
                match entry.kvs.mode {
                    IndexedDBTxnMode::Readonly => self.can_start_readonly(txn_id),
                    IndexedDBTxnMode::Readwrite | IndexedDBTxnMode::Versionchange => {
                        self.can_start_readwrite(txn_id)
                    },
                }
            };

            if should_start {
                if let Some(entry) = self.transactions.get_mut(&txn_id) {
                    entry.started = true;
                }
                // https://w3c.github.io/IndexedDB/#transaction-lifecycle
                // the implementation must queue a database task to start the transaction asynchronously.
                let _ = sender.send(IndexedDBThreadMsg::StartQueuedTransaction {
                    origin: origin.clone(),
                    name: db_name.clone(),
                    transaction: txn_id,
                });
            }
        }
    }

    fn mark_transaction_finished(&mut self, txn: u64) -> bool {
        let Some(entry) = self.transactions.get_mut(&txn) else {
            return false;
        };
        entry.finished = true;
        entry.started = false;
        true
    }

    fn abort_transaction(&mut self, txn: u64) -> bool {
        let Some(entry) = self.transactions.get_mut(&txn) else {
            return false;
        };
        entry.finished = true;
        entry.started = false;
        entry.kvs.requests.clear();
        true
    }
}

fn backend_error_from_sqlite_error(err: RusqliteError) -> BackendError {
    if is_sqlite_disk_full_error(&err) {
        BackendError::QuotaExceeded
    } else {
        BackendError::DbErr(format!("{err:?}"))
    }
}

/// Keeping track of pending upgrade transactions.
/// TODO: move into general transaction lifecycle.
struct PendingUpgrade {
    sender: GenericCallback<BackendResult<OpenDatabaseResult>>,
    origin: ImmutableOrigin,
    old_version: u64,
    db_version: u64,
    created_db: bool,
}

struct IndexedDBManager {
    port: GenericReceiver<IndexedDBThreadMsg>,
    sender: GenericSender<IndexedDBThreadMsg>,
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
    fn new(
        port: GenericReceiver<IndexedDBThreadMsg>,
        sender: GenericSender<IndexedDBThreadMsg>,
        idb_base_dir: PathBuf,
    ) -> IndexedDBManager {
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
            sender,
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
                    let sender = self.sender.clone();
                    if let Some(db) = self.get_database_mut(origin.clone(), db_name.clone()) {
                        // Queues an operation for a transaction without starting it
                        db.queue_operation(&store_name, txn, mode, operation);
                        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
                        // When each request associated with a transaction is processed, a success or error event will be fired
                        // Requests must be executed in the order in which they were made against the transaction.
                        db.schedule_transactions(origin, db_name, sender);
                    }
                },
                IndexedDBThreadMsg::StartQueuedTransaction {
                    origin,
                    name,
                    transaction,
                } => {
                    let sender = self.sender.clone();
                    if let Some(db) = self.get_database_mut(origin.clone(), name.clone()) {
                        if db.start_transaction(transaction, None) {
                            db.schedule_transactions(origin, name, sender);
                        }
                    }
                },
                IndexedDBThreadMsg::TransactionFinished {
                    origin,
                    name,
                    transaction,
                } => {
                    let sender = self.sender.clone();
                    if let Some(db) = self.get_database_mut(origin.clone(), name.clone()) {
                        if db.mark_transaction_finished(transaction) {
                            // https://w3c.github.io/IndexedDB/#transaction-lifecycle
                            // https://w3c.github.io/IndexedDB/#transaction-scheduling
                            // Once a transaction has committed or aborted, it enters [finished]. No requests can be made
                            // and scheduling constraints depend on “are not finished”.
                            db.schedule_transactions(origin, name, sender);
                        }
                    }
                },
                IndexedDBThreadMsg::AbortTransaction {
                    origin,
                    name,
                    transaction,
                } => {
                    let sender = self.sender.clone();
                    if let Some(db) = self.get_database_mut(origin.clone(), name.clone()) {
                        if db.abort_transaction(transaction) {
                            db.schedule_transactions(origin, name, sender);
                        }
                    }
                },
                IndexedDBThreadMsg::OpenTransactionInactive {
                    name,
                    transaction,
                    aborted,
                } => {
                    self.handle_open_transaction_inactive(name, transaction, aborted);
                },
            }
        }
    }

    /// Handle when an open transaction becomes inactive.
    /// TODO: transaction lifecyle.
    fn handle_open_transaction_inactive(&mut self, name: String, transaction: u64, aborted: bool) {
        let Some(pending_upgrade) = self.pending_upgrades.remove(&(name.clone(), transaction))
        else {
            error!("OpenTransactionInactive received for non-existent pending upgrade.");
            return;
        };
        if aborted {
            if pending_upgrade.created_db && pending_upgrade.old_version == 0 {
                let idb_description = IndexedDBDescription {
                    origin: pending_upgrade.origin.clone(),
                    name: name.clone(),
                };
                if let Some(db) = self.databases.remove(&idb_description) {
                    let _ = db.delete_database_now();
                }
            } else if let Some(db) = self.get_database_mut(pending_upgrade.origin.clone(), name) {
                let _ = db.set_version(pending_upgrade.old_version);
            }

            if pending_upgrade
                .sender
                .send(Ok(OpenDatabaseResult::AbortError))
                .is_err()
            {
                error!("Failed to send OpenDatabaseResult::AbortError to script.");
            };
            return;
        }

        if pending_upgrade
            .sender
            .send(Ok(OpenDatabaseResult::Connection {
                version: pending_upgrade.db_version,
                upgraded: true,
            }))
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
        sender: GenericCallback<BackendResult<OpenDatabaseResult>>,
        created_db: bool,
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
            .send(Ok(OpenDatabaseResult::Upgrade {
                version: new_version,
                old_version,
                transaction: transaction_id,
                created_db,
            }))
            .is_err()
        {
            error!("Couldn't queue task for indexeddb upgrade event.");
        }

        // Step 11: Wait for transaction to finish.
        self.pending_upgrades.insert(
            (db_name, transaction_id),
            PendingUpgrade {
                sender,
                origin: idb_description.origin.clone(),
                old_version,
                db_version: new_version,
                created_db,
            },
        );
    }

    /// <https://w3c.github.io/IndexedDB/#open-a-database-connection>
    fn open_database(
        &mut self,
        sender: GenericCallback<BackendResult<OpenDatabaseResult>>,
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
        let requested_version = version;

        // Step 4: Let db be the database named name in origin, or null otherwise.
        let (db_version, version, created_db) = match self.databases.entry(idb_description.clone())
        {
            Entry::Vacant(e) => {
                // Step 5: If version is undefined, let version be 1 if db is null, or db’s version otherwise.
                // Note: done below with the zero as first tuple item.

                // https://www.w3.org/TR/IndexedDB/#open-a-database-connection
                // Step 6: If db is null, let db be a new database
                // with name name, version 0 (zero), and with no object stores.
                // If this fails for any reason, return an appropriate error
                // (e.g. a "QuotaExceededError" or "UnknownError" DOMException).
                let engine = match SqliteEngine::new(
                    idb_base_dir,
                    &idb_description,
                    self.thread_pool.clone(),
                ) {
                    Ok(engine) => engine,
                    Err(err) => {
                        if let Err(e) = sender.send(Err(backend_error_from_sqlite_error(err))) {
                            debug!("Script exit during indexeddb database open {:?}", e);
                        }
                        return;
                    },
                };
                let created_db_path = engine.created_db_path();
                let db = IndexedDBEnvironment::new(engine);
                let db_version = match db.version() {
                    Ok(version) => version,
                    Err(err) => {
                        if let Err(e) = sender.send(Err(backend_error_from_sqlite_error(err))) {
                            debug!("Script exit during indexeddb database open {:?}", e);
                        }

                        return;
                    },
                };

                let version = if created_db_path {
                    requested_version.unwrap_or(1)
                } else {
                    requested_version.unwrap_or(db_version)
                };

                e.insert(db);
                (db_version, version, created_db_path)
            },
            Entry::Occupied(db) => {
                let db_version = match db.get().version() {
                    Ok(version) => version,
                    Err(err) => {
                        if let Err(e) = sender.send(Err(backend_error_from_sqlite_error(err))) {
                            debug!("Script exit during indexeddb database open {:?}", e);
                        }
                        return;
                    },
                };
                // Step 5: If version is undefined, let version be 1 if db is null, or db’s version otherwise.
                (db_version, requested_version.unwrap_or(db_version), false)
            },
        };

        // Step 7: If db’s version is greater than version,
        // return a newly created "VersionError" DOMException
        // and abort these steps.
        if version < db_version {
            if sender.send(Ok(OpenDatabaseResult::VersionError)).is_err() {
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
            self.upgrade_database(idb_description, version, db_name, sender, created_db);
            return;
        }

        // Step 11:
        if sender
            .send(Ok(OpenDatabaseResult::Connection {
                version: db_version,
                upgraded: false,
            }))
            .is_err()
        {
            error!("Failed to send OpenDatabaseResult::Connection to script.");
        };
    }

    /// <https://www.w3.org/TR/IndexedDB/#delete-a-database>
    fn delete_database(&mut self, idb_description: IndexedDBDescription) -> BackendResult<u64> {
        // Step 1: Let queue be the connection queue for storageKey and name.
        // Step 2: Add request to queue.
        // Step 3: Wait until all previous requests in queue have been processed.
        // TODO: implement connection queue.

        // Step4: Let db be the database named name in storageKey, if one exists. Otherwise, return 0 (zero).
        let version = if let Some(db) = self.databases.remove(&idb_description) {
            // Step 5: Let openConnections be the set of all connections associated with db.
            // Step6: For each entry of openConnections that does not have its close pending flag set to true,
            // queue a database task to fire a version change event named versionchange
            // at entry with db’s version and null.
            // Step 7: Wait for all of the events to be fired.
            // Step 8: If any of the connections in openConnections are still not closed,
            // queue a database task to fire a version change event
            // named blocked at request with db’s version and null.
            // Step 9: Wait until all connections in openConnections are closed.
            // TODO: implement connections.

            // Step 10: Let version be db’s version.
            let version = db.version().map_err(backend_error_from_sqlite_error)?;

            // Step 11: Delete db.
            // If this fails for any reason,
            // return an appropriate error (e.g. a QuotaExceededError, or an "UnknownError" DOMException).
            db.delete_database()?;
            version
        } else {
            0
        };

        // step 12: Return version.
        BackendResult::Ok(version)
    }

    fn handle_sync_operation(&mut self, operation: SyncOperation) {
        match operation {
            SyncOperation::GetDatabases(sender, origin) => {
                // The in-parallel steps of https://www.w3.org/TR/IndexedDB/#dom-idbfactory-databases

                // Step 4.1 Let databases be the set of databases in storageKey.
                // If this cannot be determined for any reason,
                // then queue a database task to reject p with an appropriate error
                // (e.g. an "UnknownError" DOMException) and terminate these steps.
                // TODO: separate database and connection concepts.
                // For now using `self.databases`, which track connections.

                // Step 4.2: Let result be a new list.
                let info_list: Vec<DatabaseInfo> = self
                    .databases
                    .iter()
                    .filter_map(|(description, info)| {
                        // Step 4.3: For each db of databases:
                        if let Ok(version) = info.version() {
                            // Step 4.3.4: If db’s version is 0, then continue.
                            if version == 0 {
                                None
                            } else {
                                // Step 4.3.5: Let info be a new IDBDatabaseInfo dictionary.
                                // Step 4.3.6: Set info’s name dictionary member to db’s name.
                                // Step 4.3.7: Set info’s version dictionary member to db’s version.
                                // Step 4.3.8: Append info to result.
                                if description.origin == origin {
                                    Some(DatabaseInfo {
                                        name: description.name.clone(),
                                        version,
                                    })
                                } else {
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                // Note: if anything went wrong, we reply with an error.
                let result = if info_list.len() == self.databases.len() {
                    Ok(info_list)
                } else {
                    Err(BackendError::DbErr(
                        "Unknown error getting database info.".to_string(),
                    ))
                };

                // Step 4.4: Queue a database task to resolve p with result.
                if sender.send(result).is_err() {
                    debug!("Couldn't send SyncOperation::GetDatabases reply.");
                }
            },
            SyncOperation::CloseDatabase(sender, origin, db_name) => {
                // TODO: Wait for all transactions created using connection to complete.
                // Note: current behavior is as if the `forced` flag is always set.
                // TODO: do not delete the database, only the connection.
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
                let idb_description = IndexedDBDescription {
                    origin,
                    name: db_name,
                };
                // https://www.w3.org/TR/IndexedDB/#dom-idbfactory-deletedatabase
                // Step 4.1: Let result be the result of deleting a database,
                // with storageKey, name, and request.
                let result = self.delete_database(idb_description);
                if callback.send(result).is_err() {
                    error!("Failed to send delete database result to script");
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
                    let _ = sender.send(db.version().map_err(backend_error_from_sqlite_error));
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
                let sender_for_schedule = self.sender.clone();
                if let Some(db) = self.get_database_mut(origin.clone(), db_name.clone()) {
                    let _ = db.start_transaction(txn, Some(sender));
                    db.schedule_transactions(origin, db_name, sender_for_schedule);
                } else {
                    let _ = sender.send(Err(BackendError::DbNotFound));
                }
            },
            SyncOperation::Version(sender, origin, db_name) => {
                if let Some(db) = self.get_database(origin, db_name) {
                    let _ = sender.send(db.version().map_err(backend_error_from_sqlite_error));
                } else {
                    let _ = sender.send(Err(BackendError::DbNotFound));
                }
            },
            SyncOperation::RegisterNewTxn(sender, _origin, _db_name) => {
                // Note: ignoring origin and name for now,
                // but those could be used again when implementing
                // lifecycle.
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
