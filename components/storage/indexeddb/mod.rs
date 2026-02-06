/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod engines;

use std::borrow::ToOwned;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use base::generic_channel::{self, GenericReceiver, GenericSender, ReceiveError};
use base::threadpool::ThreadPool;
use log::{debug, error, warn};
use profile_traits::generic_callback::GenericCallback;
use rusqlite::Error as RusqliteError;
use rustc_hash::FxHashMap;
use servo_config::pref;
use servo_url::origin::ImmutableOrigin;
use storage_traits::indexeddb::{
    AsyncOperation, BackendError, BackendResult, ConnectionMsg, CreateObjectResult, DatabaseInfo,
    DbResult, IndexedDBIndex, IndexedDBObjectStore, IndexedDBThreadMsg, IndexedDBTxnMode, KeyPath,
    SyncOperation, TxnCompleteMsg,
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

        let manager_sender = chan.clone();

        thread::Builder::new()
            .name("IndexedDBManager".to_owned())
            .spawn(move || {
                IndexedDBManager::new(port, manager_sender, idb_base_dir).start();
            })
            .expect("Thread spawning failed");

        chan
    }
}

/// A key used to track databases.
/// TODO: use a storage key.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    manager_sender: GenericSender<IndexedDBThreadMsg>,
    transactions: FxHashMap<u64, KvsTransaction>,
    queued_readwrite: VecDeque<u64>,
    queued_readonly: VecDeque<u64>,
    running_readwrite: Option<u64>,
    running_readonly: HashSet<u64>,
    handled_next_unhandled_request_id: FxHashMap<u64, u64>,
    handled_pending: FxHashMap<u64, HashSet<u64>>,
}

impl<E: KvsEngine> IndexedDBEnvironment<E> {
    fn new(
        engine: E,
        manager_sender: GenericSender<IndexedDBThreadMsg>,
    ) -> IndexedDBEnvironment<E> {
        IndexedDBEnvironment {
            engine,
            manager_sender,
            transactions: FxHashMap::default(),
            queued_readwrite: VecDeque::new(),
            queued_readonly: VecDeque::new(),
            running_readwrite: None,
            running_readonly: HashSet::new(),
            handled_next_unhandled_request_id: FxHashMap::default(),
            handled_pending: FxHashMap::default(),
        }
    }

    fn enqueue_txn(&mut self, txn: u64, mode: &IndexedDBTxnMode) {
        let queue = match mode {
            IndexedDBTxnMode::Readonly => &mut self.queued_readonly,
            _ => &mut self.queued_readwrite,
        };
        if !queue.contains(&txn) {
            queue.push_back(txn);
        }
    }

    fn queue_operation(
        &mut self,
        store_name: &str,
        serial_number: u64,
        mode: IndexedDBTxnMode,
        operation: AsyncOperation,
    ) {
        let mut enqueue_mode = None;
        match self.transactions.entry(serial_number) {
            Entry::Occupied(mut entry) => {
                let transaction = entry.get_mut();
                let transaction_mode = transaction.mode.clone();
                let was_empty = transaction.requests.is_empty();
                transaction.requests.push_back(KvsOperation {
                    operation,
                    store_name: String::from(store_name),
                });
                if was_empty {
                    let is_running = match transaction_mode {
                        IndexedDBTxnMode::Readonly => {
                            self.running_readonly.contains(&serial_number)
                        },
                        _ => self.running_readwrite == Some(serial_number),
                    };
                    if !is_running {
                        enqueue_mode = Some(transaction_mode);
                    }
                }
            },
            Entry::Vacant(entry) => {
                entry
                    .insert(KvsTransaction {
                        requests: VecDeque::new(),
                        mode: mode.clone(),
                    })
                    .requests
                    .push_back(KvsOperation {
                        operation,
                        store_name: String::from(store_name),
                    });
                enqueue_mode = Some(mode);
            },
        };
        if let Some(mode) = enqueue_mode {
            self.enqueue_txn(serial_number, &mode);
        }
    }

    fn schedule_transactions(&mut self, origin: ImmutableOrigin, db_name: &str) {
        // when a request comes in and the transaction can be started you can immediately run it;
        // otherwise run it when the transaction starts. Also re-check queued txns whenever another finishes.
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // “Read-only transactions can run concurrently.”
        // “Read/write transactions are exclusive.”
        if self.running_readwrite.is_some() {
            return;
        }

        // Drain runnable readonly txns first.
        while let Some(txn) = self.queued_readonly.pop_front() {
            let Some(transaction) = self.transactions.get(&txn) else {
                continue;
            };
            if transaction.requests.is_empty() {
                continue;
            }

            // Mark running and start async.
            // DO NOT clear here; clear on EngineTxnBatchComplete.
            self.running_readonly.insert(txn);
            self.start_transaction(origin.clone(), db_name.to_string(), txn, None);
        }

        if !self.running_readonly.is_empty() {
            return;
        }

        // Start at most one readwrite txn.
        while let Some(txn) = self.queued_readwrite.pop_front() {
            let Some(transaction) = self.transactions.get(&txn) else {
                continue;
            };

            if transaction.requests.is_empty() {
                continue;
            }

            // Mark running and start async.
            // DO NOT clear here; clear on EngineTxnBatchComplete.
            self.running_readwrite = Some(txn);
            self.start_transaction(origin, db_name.to_string(), txn, None);
            return;
        }
    }

    // Executes all requests for a transaction (without committing)
    fn start_transaction(
        &mut self,
        origin: ImmutableOrigin,
        db_name: String,
        txn: u64,
        sender: Option<GenericSender<BackendResult<()>>>,
    ) {
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // The implementation must queue a database task to start the transaction asynchronously.
        // “Requests must be executed in the order in which they were made against the transaction.”

        // Take the current queued batch of requests for this txn.
        // If more requests arrive while the engine is running, they’ll be queued into
        // transaction.requests and scheduled in a later batch once we receive EngineTxnBatchComplete.
        let (mode, requests) = match self.transactions.get_mut(&txn) {
            Some(transaction) => {
                let mode = transaction.mode.clone();
                let requests = std::mem::take(&mut transaction.requests);
                (mode, requests)
            },
            None => {
                // If a manual starter is waiting, treat as "nothing to do".
                if let Some(sender) = sender {
                    let _ = sender.send(Ok(()));
                }
                return;
            },
        };

        if requests.is_empty() {
            if let Some(sender) = sender {
                let _ = sender.send(Ok(()));
            }
            // Important: if there was no work, do NOT send EngineTxnBatchComplete,
            // otherwise we can create a pointless reschedule loop.
            return;
        }

        let rx = self
            .engine
            .process_transaction(KvsTransaction { mode, requests });

        // We must notify the manager thread when the engine finishes so it can:
        // - clear running_readonly / running_readwrite
        // - re-run scheduling (maybe start next queued txn or next batch)
        let manager_sender = self.manager_sender.clone();

        // NOTE: This is the “database task” that runs asynchronously.
        thread::spawn(move || {
            let _ = rx.blocking_recv();

            let _ = manager_sender.send(IndexedDBThreadMsg::EngineTxnBatchComplete {
                origin,
                db_name,
                txn,
            });

            // We have a sender if the transaction is started manually, and they
            // probably want to know when it is finished.
            if let Some(sender) = sender {
                let _ = sender.send(Ok(()));
            }
        });
    }

    fn mark_request_handled(&mut self, txn: u64, request_id: u64) {
        let current = self
            .handled_next_unhandled_request_id
            .get(&txn)
            .copied()
            .unwrap_or(0);
        if request_id == current {
            let mut next = current + 1;
            if let Some(pending) = self.handled_pending.get_mut(&txn) {
                while pending.remove(&next) {
                    next += 1;
                }
                if pending.is_empty() {
                    self.handled_pending.remove(&txn);
                }
            }
            self.handled_next_unhandled_request_id.insert(txn, next);
        } else if request_id > current {
            self.handled_pending
                .entry(txn)
                .or_default()
                .insert(request_id);
        }
    }

    fn abort_transaction(&mut self, txn: u64) {
        self.transactions.remove(&txn);
        self.queued_readonly.retain(|queued| *queued != txn);
        self.queued_readwrite.retain(|queued| *queued != txn);
        if self.running_readwrite == Some(txn) {
            self.running_readwrite = None;
        }
        self.running_readonly.remove(&txn);
        self.handled_next_unhandled_request_id.remove(&txn);
        self.handled_pending.remove(&txn);
    }

    fn has_key_generator(&self, store_name: &str) -> bool {
        self.engine.has_key_generator(store_name)
    }

    fn key_path(&self, store_name: &str) -> Option<KeyPath> {
        self.engine.key_path(store_name)
    }

    fn indexes(&self, store_name: &str) -> DbResult<Vec<IndexedDBIndex>> {
        self.engine
            .indexes(store_name)
            .map_err(|err| format!("{err:?}"))
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

    fn version(&self) -> Result<u64, E::Error> {
        self.engine.version()
    }

    fn set_version(&mut self, version: u64) -> DbResult<()> {
        self.engine
            .set_version(version)
            .map_err(|err| format!("{err:?}"))
    }
}

fn backend_error_from_sqlite_error(err: RusqliteError) -> BackendError {
    if is_sqlite_disk_full_error(&err) {
        BackendError::QuotaExceeded
    } else {
        BackendError::DbErr(format!("{err:?}"))
    }
}

/// <https://w3c.github.io/IndexedDB/#request-open-request>
/// Used here to implement the
/// <https://w3c.github.io/IndexedDB/#connection-queue>
enum OpenRequest {
    Open {
        /// The callback used to send a result to script.
        sender: GenericCallback<ConnectionMsg>,

        /// The name of the database.
        db_name: String,

        /// Optionnaly, a requested db version.
        /// Note: when the open algorithm starts, this will be mutated and set to something as per the algo.
        version: Option<u64>,

        /// Optionnaly, a version pending ugrade.
        /// Used as <https://w3c.github.io/IndexedDB/#request-processed-flag>
        pending_upgrade: Option<VersionUpgrade>,

        /// This request is pending on these connections to close.
        pending_close: HashSet<Uuid>,

        /// This request is pending on these connections to fire a versionchange event.
        /// Note: This starts as equal to `pending_close`, but when all events have fired,
        /// not all connections need to have closed, in which case the `blocked` event
        /// is fired on this request.
        pending_versionchange: HashSet<Uuid>,

        id: Uuid,
    },
    Delete {
        /// The callback used to send a result to script.
        sender: GenericCallback<BackendResult<u64>>,

        /// The origin of the request.
        /// TODO: storage key.
        /// Note: will be used when the full spec is implemented.
        _origin: ImmutableOrigin,

        /// The name of the database.
        /// Note: will be used when the full spec is implemented.
        _db_name: String,

        /// <https://w3c.github.io/IndexedDB/#request-processed-flag>
        processed: bool,

        id: Uuid,
    },
}

impl OpenRequest {
    fn get_id(&self) -> Uuid {
        let id = match self {
            OpenRequest::Open {
                sender: _,
                db_name: _,
                version: _,
                pending_upgrade: _,
                pending_close: _,
                pending_versionchange: _,
                id,
            } => id,
            OpenRequest::Delete {
                sender: _,
                _origin: _,
                _db_name: _,
                processed: _,
                id,
            } => id,
        };
        *id
    }

    fn is_open(&self) -> bool {
        match self {
            OpenRequest::Open {
                sender: _,
                db_name: _,
                version: _,
                pending_upgrade: _,
                pending_close: _,
                pending_versionchange: _,
                id: _,
            } => true,
            OpenRequest::Delete {
                sender: _,
                _origin: _,
                _db_name: _,
                processed: _,
                id: _,
            } => false,
        }
    }

    /// An open request can be pending either an upgrade,
    /// or the closing of other connections.
    fn is_pending(&self) -> bool {
        match self {
            OpenRequest::Open {
                sender: _,
                db_name: _,
                version: _,
                pending_upgrade,
                pending_close,
                pending_versionchange,
                id: _,
            } => {
                pending_upgrade.is_some() ||
                    !pending_close.is_empty() ||
                    !pending_versionchange.is_empty()
            },
            OpenRequest::Delete {
                sender: _,
                _origin: _,
                _db_name: _,
                processed,
                id: _,
            } => !processed,
        }
    }

    /// Abort the open request,
    /// optionally returning a version to revert to.
    fn abort(&self) -> Option<u64> {
        match self {
            OpenRequest::Open {
                sender,
                db_name,
                version: _,
                pending_close: _,
                pending_versionchange: _,
                pending_upgrade,
                id,
            } => {
                if sender
                    .send(ConnectionMsg::AbortError {
                        name: db_name.clone(),
                        id: *id,
                    })
                    .is_err()
                {
                    error!("Failed to send ConnectionMsg::Connection to script.");
                };
                pending_upgrade.as_ref().map(|upgrade| upgrade.old)
            },
            OpenRequest::Delete {
                sender,
                _origin: _,
                _db_name: _,
                processed: _,
                id: _,
            } => {
                if sender.send(Err(BackendError::DbNotFound)).is_err() {
                    error!("Failed to send result of database delete to script.");
                };
                None
            },
        }
    }
}

struct VersionUpgrade {
    old: u64,
    new: u64,
}

/// <https://w3c.github.io/IndexedDB/#connection>
struct Connection {
    /// <https://w3c.github.io/IndexedDB/#connection-close-pending-flag>
    close_pending: bool,

    /// The callback used to send a result to script.
    sender: GenericCallback<ConnectionMsg>,
}

struct IndexedDBManager {
    port: GenericReceiver<IndexedDBThreadMsg>,
    manager_sender: GenericSender<IndexedDBThreadMsg>,
    idb_base_dir: PathBuf,
    databases: HashMap<IndexedDBDescription, IndexedDBEnvironment<SqliteEngine>>,
    thread_pool: Arc<ThreadPool>,

    /// A global counter to produce unique transaction ids.
    /// TODO: remove once db connections lifecyle is managed.
    /// A global counter is only necessary because of how deleting a db currently
    /// does not wait for connection to close and transactions to finish.
    serial_number_counter: u64,

    /// <https://w3c.github.io/IndexedDB/#connection-queue>
    connection_queues: HashMap<IndexedDBDescription, VecDeque<OpenRequest>>,

    /// <https://w3c.github.io/IndexedDB/#connection>
    connections: HashMap<IndexedDBDescription, HashMap<Uuid, Connection>>,
}

impl IndexedDBManager {
    fn new(
        port: GenericReceiver<IndexedDBThreadMsg>,
        manager_sender: GenericSender<IndexedDBThreadMsg>,
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
            manager_sender,
            idb_base_dir,
            databases: HashMap::new(),
            thread_pool: Arc::new(ThreadPool::new(thread_count, "IndexedDB".to_string())),
            serial_number_counter: 0,
            connection_queues: Default::default(),
            connections: Default::default(),
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
                IndexedDBThreadMsg::Async(
                    origin,
                    db_name,
                    store_name,
                    txn,
                    _request_id,
                    mode,
                    operation,
                ) => {
                    if let Some(db) = self.get_database_mut(origin.clone(), db_name.clone()) {
                        // Queues an operation for a transaction without starting it
                        db.queue_operation(&store_name, txn, mode, operation);
                        db.schedule_transactions(origin, &db_name);
                    }
                },
                IndexedDBThreadMsg::OpenTransactionInactive { name, origin } => {
                    self.handle_open_transaction_inactive(name, origin);
                },
                IndexedDBThreadMsg::EngineTxnBatchComplete {
                    origin,
                    db_name,
                    txn,
                } => {
                    if let Some(db) = self.get_database_mut(origin.clone(), db_name.clone()) {
                        // Decide which running flag to clear based on txn mode.
                        let mode = db.transactions.get(&txn).map(|t| t.mode.clone());

                        match mode {
                            Some(IndexedDBTxnMode::Readonly) => {
                                db.running_readonly.remove(&txn);
                            },
                            Some(_) => {
                                if db.running_readwrite == Some(txn) {
                                    db.running_readwrite = None;
                                }
                            },
                            None => {
                                // txn might have been aborted/removed; nothing to clear
                            },
                        }

                        // If more requests were queued while this batch was running,
                        // schedule again now.
                        db.schedule_transactions(origin, &db_name);
                    }
                },
            }
        }
    }

    /// Handle when an open transaction becomes inactive.
    fn handle_open_transaction_inactive(&mut self, name: String, origin: ImmutableOrigin) {
        let key = IndexedDBDescription { name, origin };
        let Some(queue) = self.connection_queues.get_mut(&key) else {
            return debug_assert!(false, "A connection queue should exist.");
        };
        let Some(open_request) = queue.pop_front() else {
            return debug_assert!(false, "A pending open request should exist.");
        };
        let OpenRequest::Open {
            sender,
            db_name,
            version: _,
            pending_upgrade,
            pending_close: _,
            pending_versionchange: _,
            id,
        } = open_request
        else {
            return;
        };
        let Some(VersionUpgrade { old: _, new }) = pending_upgrade else {
            return debug_assert!(false, "A pending version upgrade should exist.");
        };
        if sender
            .send(ConnectionMsg::Connection {
                id,
                name: db_name,
                version: new,
                upgraded: true,
            })
            .is_err()
        {
            error!("Failed to send ConnectionMsg::Connection to script.");
        };

        self.advance_connection_queue(key);
    }

    /// Run the next open request in the queue.
    fn advance_connection_queue(&mut self, key: IndexedDBDescription) {
        loop {
            let is_open = {
                let Some(queue) = self.connection_queues.get_mut(&key) else {
                    return;
                };
                if queue.is_empty() {
                    return;
                }
                queue.front().expect("Queue is not empty.").is_open()
            };

            if is_open {
                self.open_database(key.clone());
            } else {
                self.delete_database(key.clone());
            }

            let was_pruned = self.maybe_remove_front_from_queue(&key);

            if !was_pruned {
                // Note: requests to delete a database are, at this point in the implementation,
                // done in one step; so we can continue on to the next request.
                // Request to open a connection consists of multiple async steps, so we must break if
                // it is still pending.
                break;
            }
        }
    }

    /// Remove the record at the front if it is not pending.
    fn maybe_remove_front_from_queue(&mut self, key: &IndexedDBDescription) -> bool {
        let (is_empty, was_pruned) = {
            let Some(queue) = self.connection_queues.get_mut(key) else {
                debug_assert!(false, "A connection queue should exist.");
                return false;
            };
            let mut pruned = false;
            let front_is_pending = queue.front().map(|record| record.is_pending());
            if let Some(is_pending) = front_is_pending {
                if !is_pending {
                    queue.pop_front().expect("Queue has a non-pending item.");
                    pruned = true
                }
            }
            (queue.is_empty(), pruned)
        };
        if is_empty {
            self.connection_queues.remove(key);
        }
        was_pruned
    }

    fn remove_connection(&mut self, key: &IndexedDBDescription, id: &Uuid) {
        let is_empty = {
            let Some(connections) = self.connections.get_mut(key) else {
                return debug!("Connection already removed.");
            };
            connections.remove(id);
            connections.is_empty()
        };

        if is_empty {
            self.connections.remove(key);
        }
    }

    /// Aborting the current upgrade for an origin.
    // https://w3c.github.io/IndexedDB/#abort-an-upgrade-transaction
    /// Note: this only reverts the version at this point.
    fn abort_pending_upgrade(&mut self, name: String, id: Uuid, origin: ImmutableOrigin) {
        let key = IndexedDBDescription {
            name,
            origin: origin.clone(),
        };
        let old = {
            let Some(queue) = self.connection_queues.get_mut(&key) else {
                return debug_assert!(
                    false,
                    "There should be a connection queue for the aborted upgrade."
                );
            };
            let Some(open_request) = queue.pop_front() else {
                return debug_assert!(false, "There should be an open request to upgrade.");
            };
            if open_request.get_id() != id {
                return debug_assert!(
                    false,
                    "Open request to abort should be at the head of the queue."
                );
            }
            open_request.abort()
        };
        if let Some(old_version) = old {
            let Some(db) = self.databases.get_mut(&key) else {
                return debug_assert!(false, "Db should have been created");
            };
            // Step 3: Set connection’s version to database’s version if database previously existed
            //  or 0 (zero) if database was newly created.
            let res = db.set_version(old_version);
            debug_assert!(res.is_ok(), "Setting a db version should not fail.");
        }

        self.remove_connection(&key, &id);

        self.advance_connection_queue(key);
    }

    /// Aborting all upgrades for an origin
    // https://w3c.github.io/IndexedDB/#abort-an-upgrade-transaction
    /// Note: this only reverts the version at this point.
    fn abort_pending_upgrades(
        &mut self,
        pending_upgrades: HashMap<String, HashSet<Uuid>>,
        origin: ImmutableOrigin,
    ) {
        for (name, ids) in pending_upgrades.into_iter() {
            let mut version_to_revert: Option<u64> = None;
            let key = IndexedDBDescription {
                name,
                origin: origin.clone(),
            };
            for id in ids.iter() {
                self.remove_connection(&key, id);
            }
            {
                let is_empty = {
                    let Some(queue) = self.connection_queues.get_mut(&key) else {
                        continue;
                    };
                    queue.retain_mut(|open_request| {
                        if ids.contains(&open_request.get_id()) {
                            let old = open_request.abort();
                            if version_to_revert.is_none() {
                                if let Some(old) = old {
                                    version_to_revert = Some(old);
                                }
                            }
                            false
                        } else {
                            true
                        }
                    });
                    queue.is_empty()
                };
                if is_empty {
                    self.connection_queues.remove(&key);
                }
            }
            if let Some(version) = version_to_revert {
                let Some(db) = self.databases.get_mut(&key) else {
                    return debug_assert!(false, "Db should have been created");
                };
                // Step 3: Set connection’s version to database’s version if database previously existed
                //  or 0 (zero) if database was newly created.
                let res = db.set_version(version);
                debug_assert!(res.is_ok(), "Setting a db version should not fail.");
            }
        }
    }

    /// <https://w3c.github.io/IndexedDB/#open-a-database-connection>
    fn open_a_database_connection(
        &mut self,
        sender: GenericCallback<ConnectionMsg>,
        origin: ImmutableOrigin,
        db_name: String,
        version: Option<u64>,
        id: Uuid,
    ) {
        let key = IndexedDBDescription {
            name: db_name.clone(),
            origin: origin.clone(),
        };
        let open_request = OpenRequest::Open {
            sender,
            db_name,
            version,
            pending_close: Default::default(),
            pending_versionchange: Default::default(),
            pending_upgrade: None,
            id,
        };
        let should_continue = {
            // Step 1: Let queue be the connection queue for storageKey and name.
            let queue = self.connection_queues.entry(key.clone()).or_default();

            // Step 2: Add request to queue.
            queue.push_back(open_request);
            queue.len() == 1
        };

        // Step 3: Wait until all previous requests in queue have been processed.
        if should_continue {
            self.open_database(key.clone());
            self.maybe_remove_front_from_queue(&key);
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

    /// <https://w3c.github.io/IndexedDB/#upgrade-a-database>
    /// To upgrade a database with connection (a connection),
    /// a new version, and a request, run these steps:
    /// TODO: connection and request.
    fn upgrade_database(&mut self, key: IndexedDBDescription, new_version: u64) {
        let Some(queue) = self.connection_queues.get_mut(&key) else {
            return debug_assert!(false, "A connection queue should exist.");
        };
        let Some(open_request) = queue.front_mut() else {
            return debug_assert!(false, "An open request should be in the queue.");
        };
        let OpenRequest::Open {
            sender,
            db_name,
            version: _,
            id,
            pending_close: _,
            pending_versionchange: _,
            pending_upgrade,
        } = open_request
        else {
            return;
        };

        // Step 1: Let db be connection’s database.
        // TODO: connection.
        let db = self
            .databases
            .get_mut(&key)
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
            .send(ConnectionMsg::Upgrade {
                id: *id,
                name: db_name.clone(),
                version: new_version,
                old_version,
                transaction: transaction_id,
            })
            .is_err()
        {
            error!("Couldn't queue task for indexeddb upgrade event.");
        }

        // Step 11: Wait for transaction to finish.
        let _ = pending_upgrade.insert(VersionUpgrade {
            old: old_version,
            new: new_version,
        });
    }

    /// <https://w3c.github.io/IndexedDB/#open-a-database-connection>
    fn handle_version_change_done(
        &mut self,
        name: String,
        from_id: Uuid,
        old_version: u64,
        origin: ImmutableOrigin,
    ) {
        let key = IndexedDBDescription {
            name: name.clone(),
            origin: origin.clone(),
        };
        let (can_upgrade, version) = {
            let Some(queue) = self.connection_queues.get_mut(&key) else {
                return debug_assert!(false, "A connection queue should exist.");
            };
            let Some(open_request) = queue.front_mut() else {
                return debug_assert!(false, "An open request should be in the queue.");
            };
            let OpenRequest::Open {
                sender,
                db_name: _,
                version,
                id,
                pending_upgrade: _,
                pending_versionchange,
                pending_close,
            } = open_request
            else {
                return debug_assert!(
                    false,
                    "An request to open a connection should be in the queue."
                );
            };
            debug_assert!(
                pending_versionchange.contains(&from_id),
                "The open request should be pending on the versionchange event for the connection sending the message."
            );

            pending_versionchange.remove(&from_id);

            // Step 10.3: Wait for all of the events to be fired.
            if !pending_versionchange.is_empty() {
                return;
            }

            let Some(version) = *version else {
                return debug_assert!(
                    false,
                    "An upgrade version should have been determined by now."
                );
            };

            // Step 10.4: If any of the connections in openConnections are still not closed,
            // queue a database task to fire a version change event named blocked
            // at request with db’s version and version.
            if !pending_close.is_empty() &&
                sender
                    .send(ConnectionMsg::Blocked {
                        name,
                        id: *id,
                        version,
                        old_version,
                    })
                    .is_err()
            {
                return debug!("Script exit during indexeddb database open");
            }

            (pending_close.is_empty(), version)
        };

        // Step 10.5: Wait until all connections in openConnections are closed.
        // Note: if we still need to wait, the algorithm will continue in the handling of the close message.
        if can_upgrade {
            // Step 10.6: Run upgrade a database using connection, version and request.
            self.upgrade_database(key.clone(), version);

            let was_pruned = self.maybe_remove_front_from_queue(&key);
            if was_pruned {
                self.advance_connection_queue(key);
            }
        }
    }

    /// <https://w3c.github.io/IndexedDB/#open-a-database-connection>
    /// The part where the open request is ready for processing.
    fn open_database(&mut self, key: IndexedDBDescription) {
        let Some(queue) = self.connection_queues.get_mut(&key) else {
            return debug_assert!(false, "A connection queue should exist.");
        };
        let Some(open_request) = queue.front_mut() else {
            return debug_assert!(false, "An open request should be in the queue.");
        };
        let OpenRequest::Open {
            sender,
            db_name,
            version,
            id,
            pending_upgrade: _,
            pending_close,
            pending_versionchange,
        } = open_request
        else {
            return debug_assert!(
                false,
                "An request to open a connection should be in the queue."
            );
        };

        let idb_base_dir = self.idb_base_dir.as_path();
        let requested_version = *version;

        // Step 4: Let db be the database named name in origin, or null otherwise.
        let db_version = match self.databases.entry(key.clone()) {
            Entry::Vacant(e) => {
                // Step 5: If version is undefined, let version be 1 if db is null, or db’s version otherwise.
                // Note: done below with the zero as first tuple item.

                // https://www.w3.org/TR/IndexedDB/#open-a-database-connection
                // Step 6: If db is null, let db be a new database
                // with name name, version 0 (zero), and with no object stores.
                // If this fails for any reason, return an appropriate error
                // (e.g. a "QuotaExceededError" or "UnknownError" DOMException).
                let engine = match SqliteEngine::new(idb_base_dir, &key, self.thread_pool.clone()) {
                    Ok(engine) => engine,
                    Err(err) => {
                        let error = backend_error_from_sqlite_error(err);
                        if let Err(e) = sender.send(ConnectionMsg::DatabaseError {
                            id: *id,
                            name: db_name.clone(),
                            error,
                        }) {
                            debug!("Script exit during indexeddb database open {:?}", e);
                        }
                        return;
                    },
                };
                let created_db_path = engine.created_db_path();
                let db = IndexedDBEnvironment::new(engine, self.manager_sender.clone());
                let db_version = match db.version() {
                    Ok(version) => version,
                    Err(err) => {
                        let error = backend_error_from_sqlite_error(err);
                        if let Err(e) = sender.send(ConnectionMsg::DatabaseError {
                            id: *id,
                            name: db_name.clone(),
                            error,
                        }) {
                            debug!("Script exit during indexeddb database open {:?}", e);
                        }
                        return;
                    },
                };

                *version = if created_db_path {
                    Some(requested_version.unwrap_or(1))
                } else {
                    Some(requested_version.unwrap_or(db_version))
                };

                e.insert(db);
                db_version
            },
            Entry::Occupied(db) => {
                let db_version = match db.get().version() {
                    Ok(version) => version,
                    Err(err) => {
                        let error = backend_error_from_sqlite_error(err);
                        if let Err(e) = sender.send(ConnectionMsg::DatabaseError {
                            id: *id,
                            name: db_name.clone(),
                            error,
                        }) {
                            debug!("Script exit during indexeddb database open {:?}", e);
                        }
                        return;
                    },
                };
                // Step 5: If version is undefined, let version be 1 if db is null, or db’s version otherwise.
                *version = Some(requested_version.unwrap_or(db_version));
                db_version
            },
        };

        let Some(version) = *version else {
            return debug_assert!(
                false,
                "An upgrade version should have been determined by now."
            );
        };

        // Step 7: If db’s version is greater than version,
        // return a newly created "VersionError" DOMException
        // and abort these steps.
        if version < db_version {
            if sender
                .send(ConnectionMsg::VersionError {
                    name: db_name.clone(),
                    id: *id,
                })
                .is_err()
            {
                debug!("Script exit during indexeddb database open");
            }
            return;
        }

        // Step 8: Let connection be a new connection to db.
        // Step 9: Set connection’s version to version.
        let connection = Connection {
            close_pending: false,
            sender: sender.clone(),
        };
        let entry = self.connections.entry(key.clone()).or_default();
        entry.insert(*id, connection);

        // Step 10: If db’s version is less than version, then:
        if db_version < version {
            // Step 10.1: Let openConnections be the set of all connections,
            // except connection, associated with db.
            let open_connections = entry
                .iter_mut()
                .filter(|(other_id, conn)| !conn.close_pending && *other_id != id);
            for (id_to_close, conn) in open_connections {
                // Step 10.2: For each entry of openConnections
                // queue a database task to fire a version change event
                // named versionchange at entry with db’s version and version.
                if conn
                    .sender
                    .send(ConnectionMsg::VersionChange {
                        name: db_name.clone(),
                        id: *id_to_close,
                        version,
                        old_version: db_version,
                    })
                    .is_err()
                {
                    error!("Failed to send ConnectionMsg::Connection to script.");
                };
                pending_close.insert(*id_to_close);
                pending_versionchange.insert(*id_to_close);
            }
            if !pending_close.is_empty() {
                // Step 10.3: Wait for all of the events to be fired.
                return;
            }

            // Step 10.6: Run upgrade a database using connection, version and request.
            self.upgrade_database(key, version);
            return;
        }

        // Step 11:
        if sender
            .send(ConnectionMsg::Connection {
                name: db_name.clone(),
                id: *id,
                version: db_version,
                upgraded: false,
            })
            .is_err()
        {
            error!("Failed to send ConnectionMsg::Connection to script.");
        };
    }

    /// <https://www.w3.org/TR/IndexedDB/#delete-a-database>
    /// The part adding the request to the connection queue.
    fn start_delete_database(
        &mut self,
        key: IndexedDBDescription,
        id: Uuid,
        sender: GenericCallback<BackendResult<u64>>,
    ) {
        let open_request = OpenRequest::Delete {
            sender,
            _origin: key.origin.clone(),
            _db_name: key.name.clone(),
            processed: false,
            id,
        };

        let should_continue = {
            // Step 1: Let queue be the connection queue for storageKey and name.
            let queue = self.connection_queues.entry(key.clone()).or_default();

            // Step 2: Add request to queue.
            queue.push_back(open_request);
            queue.len() == 1
        };

        // Step 3: Wait until all previous requests in queue have been processed.
        if should_continue {
            self.delete_database(key.clone());
            self.maybe_remove_front_from_queue(&key);
        }
    }

    /// <https://www.w3.org/TR/IndexedDB/#delete-a-database>
    fn delete_database(&mut self, key: IndexedDBDescription) {
        let Some(queue) = self.connection_queues.get_mut(&key) else {
            return debug_assert!(false, "A connection queue should exist.");
        };
        let Some(open_request) = queue.front_mut() else {
            return debug_assert!(false, "An open request should be in the queue.");
        };
        let OpenRequest::Delete {
            sender,
            _origin: _,
            _db_name: _,
            processed,
            id: _,
        } = open_request
        else {
            return debug_assert!(
                false,
                "An request to open a connection should be in the queue."
            );
        };

        // Step4: Let db be the database named name in storageKey, if one exists. Otherwise, return 0 (zero).
        let version = if let Some(db) = self.databases.remove(&key) {
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
            let res = db.version();
            let Ok(version) = res else {
                if sender
                    .send(BackendResult::Err(BackendError::DbErr(
                        res.unwrap_err().to_string(),
                    )))
                    .is_err()
                {
                    debug!("Script went away during pending database delete.");
                }
                return;
            };

            // Step 11: Delete db.
            // If this fails for any reason,
            // return an appropriate error (e.g. a QuotaExceededError, or an "UnknownError" DOMException).
            if let Err(err) = db.delete_database() {
                if sender
                    .send(BackendResult::Err(BackendError::DbErr(err.to_string())))
                    .is_err()
                {
                    debug!("Script went away during pending database delete.");
                }
                return;
            };

            version
        } else {
            0
        };

        // step 12: Return version.
        if sender.send(BackendResult::Ok(version)).is_err() {
            debug!("Script went away during pending database delete.");
        }

        *processed = true;
    }

    /// <https://w3c.github.io/IndexedDB/#closing-connection>
    fn close_database(&mut self, origin: ImmutableOrigin, id: Uuid, name: String) {
        // Step 1: Set connection’s close pending flag to true.
        // TODO: seems like a script only flag.

        // Step 2: If the forced flag is true,
        // then for each transaction created using connection
        // run abort a transaction with transaction and newly created "AbortError" DOMException.
        // Step 3: Wait for all transactions created using connection to complete.
        // Once they are complete, connection is closed.
        // TODO: transaction lifecycle.

        // Step 4: If the forced flag is true, then fire an event named close at connection.
        // TODO: implement, probably only on the script side of things.

        // Note: below we are continuing
        // <https://w3c.github.io/IndexedDB/#open-a-database-connection>
        // in the case that an open request is waiting for connections to close.
        let key = IndexedDBDescription { origin, name };
        let (can_upgrade, version) = {
            self.remove_connection(&key, &id);

            let Some(queue) = self.connection_queues.get_mut(&key) else {
                return;
            };
            let Some(open_request) = queue.front_mut() else {
                return;
            };
            if let OpenRequest::Open {
                sender: _,
                db_name: _,
                version,
                id: _,
                pending_upgrade,
                pending_versionchange,
                pending_close,
            } = open_request
            {
                pending_close.remove(&id);
                (
                    // Note: need to exclude requests that have already started upgrading.
                    pending_close.is_empty() &&
                        pending_versionchange.is_empty() &&
                        !pending_upgrade.is_some(),
                    *version,
                )
            } else {
                (false, None)
            }
        };

        // <https://w3c.github.io/IndexedDB/#open-a-database-connection>
        // Step 10.3: Wait for all of the events to be fired.
        // Step 10.5: Wait until all connections in openConnections are closed.
        // Note: both conditions must be checked here,
        // because that is the condition enabling the upgrade to proceed.
        if can_upgrade {
            // Step 10.6: Run upgrade a database using connection, version and request.
            let Some(version) = version else {
                return debug_assert!(
                    false,
                    "An upgrade version should have been determined by now."
                );
            };
            self.upgrade_database(key.clone(), version);

            let was_pruned = self.maybe_remove_front_from_queue(&key);
            if was_pruned {
                self.advance_connection_queue(key);
            }
        }
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
            SyncOperation::CloseDatabase(origin, id, db_name) => {
                self.close_database(origin, id, db_name);
            },
            SyncOperation::OpenDatabase(sender, origin, db_name, version, id) => {
                self.open_a_database_connection(sender, origin, db_name, version, id);
            },
            SyncOperation::AbortPendingUpgrades {
                pending_upgrades,
                origin,
            } => {
                self.abort_pending_upgrades(pending_upgrades, origin);
            },
            SyncOperation::AbortPendingUpgrade { name, id, origin } => {
                self.abort_pending_upgrade(name, id, origin);
            },
            SyncOperation::DeleteDatabase(callback, origin, db_name, id) => {
                let idb_description = IndexedDBDescription {
                    origin,
                    name: db_name,
                };
                self.start_delete_database(idb_description, id, callback);
            },
            SyncOperation::GetObjectStore(sender, origin, db_name, store_name) => {
                // FIXME:(arihant2math) Should we error out more aggressively here?
                let result = self
                    .get_database(origin, db_name)
                    .map(|db| IndexedDBObjectStore {
                        key_path: db.key_path(&store_name),
                        has_key_generator: db.has_key_generator(&store_name),
                        indexes: db.indexes(&store_name).unwrap_or_default(),
                        name: store_name,
                    });
                let _ = sender.send(result.ok_or(BackendError::DbNotFound));
            },
            SyncOperation::CreateIndex(
                origin,
                db_name,
                store_name,
                index_name,
                key_path,
                unique,
                multi_entry,
            ) => {
                if let Some(db) = self.get_database(origin, db_name) {
                    let _ = db.create_index(&store_name, index_name, key_path, unique, multi_entry);
                }
            },
            SyncOperation::DeleteIndex(origin, db_name, store_name, index_name) => {
                if let Some(db) = self.get_database(origin, db_name) {
                    let _ = db.delete_index(&store_name, index_name);
                }
            },
            SyncOperation::Commit(callback, origin, db_name, txn) => {
                // https://w3c.github.io/IndexedDB/#commit-a-transaction
                // TODO: implement the commit algorithm and only reply after the backend has
                // transitioned the transaction to committed/aborted (should be atomic).
                let _ = callback.send(TxnCompleteMsg {
                    origin: origin.clone(),
                    db_name: db_name.clone(),
                    txn,
                    result: Ok(()),
                });
                if let Some(db) = self.get_database_mut(origin.clone(), db_name.clone()) {
                    db.schedule_transactions(origin, &db_name);
                }
            },
            SyncOperation::Abort(callback, origin, db_name, txn) => {
                // https://w3c.github.io/IndexedDB/#abort-a-transaction
                // “When a transaction is aborted the implementation must undo (roll back) any changes that were made to the database during that transaction.”
                // TODO: implement the abort algorithm and rollback for the engine.
                if let Some(db) = self.get_database_mut(origin.clone(), db_name.clone()) {
                    db.abort_transaction(txn);
                }
                let _ = callback.send(storage_traits::indexeddb::TxnCompleteMsg {
                    origin: origin.clone(),
                    db_name: db_name.clone(),
                    txn,
                    result: Err(BackendError::Abort),
                });
                if let Some(db) = self.get_database_mut(origin.clone(), db_name.clone()) {
                    db.schedule_transactions(origin, &db_name);
                }
            },
            SyncOperation::RequestHandled {
                origin,
                db_name,
                txn,
                request_id,
            } => {
                // https://w3c.github.io/IndexedDB/#transaction-lifecycl
                // The implementation must attempt to commit an inactive transaction
                // when all requests placed against the transaction have completed
                // and their returned results handled, no new requests have been
                // placed against the transaction, and the transaction has not been aborted

                if let Some(db) = self.get_database_mut(origin, db_name) {
                    db.mark_request_handled(txn, request_id);
                }
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
            SyncOperation::NotifyEndOfVersionChange {
                name,
                id,
                old_version,
                origin,
            } => {
                self.handle_version_change_done(name, id, old_version, origin);
            },
            SyncOperation::Exit(_) => {
                unreachable!("We must've already broken out of event loop.");
            },
        }
    }
}
