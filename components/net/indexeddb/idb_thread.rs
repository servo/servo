/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::borrow::ToOwned;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::thread;

use ipc_channel::ipc::{self, IpcError, IpcReceiver, IpcSender};
use log::{error, warn};
use net_traits::indexeddb_thread::{
    AsyncOperation, IndexedDBThreadMsg, IndexedDBThreadReturnType, IndexedDBTxnMode, SyncOperation,
};
use servo_config::pref;
use servo_url::origin::ImmutableOrigin;

use crate::indexeddb::engines::{
    HeedEngine, KvsEngine, KvsOperation, KvsTransaction, SanitizedName,
};

pub trait IndexedDBThreadFactory {
    fn new(config_dir: Option<PathBuf>) -> Self;
}

impl IndexedDBThreadFactory for IpcSender<IndexedDBThreadMsg> {
    fn new(config_dir: Option<PathBuf>) -> IpcSender<IndexedDBThreadMsg> {
        let (chan, port) = ipc::channel().unwrap();

        let mut idb_base_dir = PathBuf::new();
        config_dir.map(|p| idb_base_dir.push(p));
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
        sender: IpcSender<Option<Vec<u8>>>,
        store_name: SanitizedName,
        serial_number: u64,
        mode: IndexedDBTxnMode,
        operation: AsyncOperation,
    ) {
        self.transactions
            .entry(serial_number)
            .or_insert(KvsTransaction {
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
        // FIXME:(arihant2math) find a way to optimizations in this function rather than on the engine level code (less repetition)
        self.transactions.remove(&txn).map(|txn| {
            self.engine.process_transaction(txn).blocking_recv();
        });

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
        self.engine.create_store(store_name, auto_increment);

        sender.send(Ok(())).unwrap();
    }
}

struct IndexedDBManager {
    port: IpcReceiver<IndexedDBThreadMsg>,
    idb_base_dir: PathBuf,
    databases: HashMap<IndexedDBDescription, IndexedDBEnvironment<HeedEngine>>,
}

impl IndexedDBManager {
    fn new(port: IpcReceiver<IndexedDBThreadMsg>, idb_base_dir: PathBuf) -> IndexedDBManager {
        IndexedDBManager {
            port,
            idb_base_dir,
            databases: HashMap::new(),
        }
    }
}

impl IndexedDBManager {
    fn start(&mut self) {
        if pref!(dom.indexeddb.enabled) {
            loop {
                // FIXME:(arihant2math) No message *most likely* means that
                // the ipc sender has been dropped, so we break the look
                let message = match self.port.recv() {
                    Ok(msg) => msg,
                    Err(e) => match e {
                        IpcError::Disconnected => {
                            error!("indexeddb ipc channel has been dropped, breaking loop");
                            break;
                        },
                        other => Err(other).unwrap(),
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
                        self.get_database_mut(origin, db_name).map(|db| {
                            // Queues an operation for a transaction without starting it
                            db.queue_operation(sender, store_name, txn, mode, operation);
                            // FIXME:(arihant2math) Schedule transactions properly:
                            // for now, we start them directly.
                            db.start_transaction(txn, None);
                        });
                    },
                }
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
            SyncOperation::OpenDatabase(sender, origin, db_name, version) => {
                let idb_description = IndexedDBDescription {
                    origin,
                    name: db_name,
                };

                let idb_base_dir = self.idb_base_dir.as_path();

                match self.databases.entry(idb_description.clone()) {
                    Entry::Vacant(e) => {
                        let db = IndexedDBEnvironment::new(
                            HeedEngine::new(idb_base_dir, &idb_description.as_path()),
                            version.unwrap_or(0),
                        );
                        sender.send(db.version).unwrap();
                        e.insert(db);
                    },
                    Entry::Occupied(db) => {
                        sender.send(db.get().version).unwrap();
                    },
                }
            },
            SyncOperation::DeleteDatabase(sender, origin, db_name) => {
                let idb_description = IndexedDBDescription {
                    origin,
                    name: db_name,
                };
                self.databases.remove(&idb_description);

                // FIXME:(rasviitanen) Possible security issue?
                let mut db_dir = self.idb_base_dir.clone();
                db_dir.push(&idb_description.as_path());
                if std::fs::remove_dir_all(&db_dir).is_err() {
                    sender.send(Err(())).unwrap();
                } else {
                    sender.send(Ok(())).unwrap();
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
                sender
                    .send(IndexedDBThreadReturnType::Commit(Err(())))
                    .expect("Could not send commit status");
            },
            SyncOperation::UpgradeVersion(sender, origin, db_name, _txn, version) => {
                self.get_database_mut(origin, db_name).map(|db| {
                    db.version = version;
                });

                // FIXME:(arihant2math) Get the version from the database instead
                // We never fail as of now, so we can just return it like this
                // for now...
                sender
                    .send(IndexedDBThreadReturnType::UpgradeVersion(Ok(version)))
                    .expect("Could not upgrade version");
            },
            SyncOperation::CreateObjectStore(
                sender,
                origin,
                db_name,
                store_name,
                auto_increment,
            ) => {
                let store_name = SanitizedName::new(store_name);
                self.get_database_mut(origin, db_name)
                    .map(|db| db.create_object_store(sender, store_name, auto_increment));
            },
            SyncOperation::StartTransaction(sender, origin, db_name, txn) => {
                self.get_database_mut(origin, db_name).map(|db| {
                    db.start_transaction(txn, Some(sender));
                });
            },
            SyncOperation::Version(sender, origin, db_name) => {
                self.get_database(origin, db_name)
                    .map(|db| {
                        sender.send(db.version).unwrap();
                    })
                    .unwrap();
            },
            SyncOperation::RegisterNewTxn(sender, origin, db_name) => self
                .get_database_mut(origin, db_name)
                .map(|db| {
                    db.serial_number_counter += 1;
                    sender
                        .send(db.serial_number_counter)
                        .expect("Could not send serial number");
                })
                .unwrap(),
            SyncOperation::Exit(sender) => {
                // FIXME:(rasviitanen) Nothing to do?
                let _ = sender.send(IndexedDBThreadReturnType::Exit).unwrap();
            },
        }
    }
}
