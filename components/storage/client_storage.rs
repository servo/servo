/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::fmt::Debug;
use std::path::PathBuf;
use std::thread;

use log::warn;
use rusqlite::{Connection, Transaction};
use servo_base::generic_channel::{self, GenericReceiver, GenericSender};
use servo_base::id::{BrowsingContextId, WebViewId};
use servo_url::ImmutableOrigin;
use storage_traits::client_storage::{
    ClientStorageErrorr, ClientStorageThreadHandle, ClientStorageThreadMessage, Mode,
    StorageIdentifier, StorageProxyMap, StorageType,
};
use uuid::Uuid;

trait RegistryEngine {
    type Error: Debug;
    fn create_database(
        &mut self,
        bottle_id: i64,
        name: String,
    ) -> Result<PathBuf, ClientStorageErrorr<Self::Error>>;
    fn delete_database(
        &mut self,
        bottle_id: i64,
        name: String,
    ) -> Result<(), ClientStorageErrorr<Self::Error>>;
    fn obtain_a_storage_bottle_map(
        &mut self,
        storage_type: StorageType,
        webview: WebViewId,
        storage_identifier: StorageIdentifier,
        origin: ImmutableOrigin,
        sender: &GenericSender<ClientStorageThreadMessage>,
    ) -> Result<StorageProxyMap, ClientStorageErrorr<Self::Error>>;
}

struct SqliteEngine {
    connection: Connection,
    base_dir: PathBuf,
}

impl SqliteEngine {
    fn new(base_dir: PathBuf) -> rusqlite::Result<Self> {
        let db_path = base_dir.join("reg.sqlite");
        let connection = Connection::open(db_path)?;
        Self::init(&connection)?;
        Ok(SqliteEngine {
            connection,
            base_dir,
        })
    }

    fn memory() -> rusqlite::Result<Self> {
        let connection = Connection::open_in_memory()?;
        Self::init(&connection)?;
        Ok(SqliteEngine {
            connection,
            base_dir: PathBuf::new(),
        })
    }

    fn init(connection: &Connection) -> rusqlite::Result<()> {
        connection.execute(r#"PRAGMA foreign_keys = ON;"#, [])?;
        connection.execute(
            r#"CREATE TABLE IF NOT EXISTS sheds (
            id INTEGER PRIMARY KEY,
            storage_type TEXT NOT NULL,
            browsing_context TEXT
        );"#,
            [],
        )?;

        // Note: indices required for ON CONFLICT to work.
        connection.execute(
            r#"CREATE UNIQUE INDEX IF NOT EXISTS idx_sheds_local
        ON sheds(storage_type) WHERE browsing_context IS NULL;"#,
            [],
        )?;
        connection.execute(
            r#"CREATE UNIQUE INDEX IF NOT EXISTS idx_sheds_session
        ON sheds(browsing_context) WHERE browsing_context IS NOT NULL;"#,
            [],
        )?;

        connection.execute(
            r#"CREATE TABLE IF NOT EXISTS shelves (
            id INTEGER PRIMARY KEY,
            shed_id INTEGER NOT NULL,
            origin TEXT NOT NULL,
            UNIQUE (shed_id, origin),
            FOREIGN KEY (shed_id) REFERENCES sheds(id) ON DELETE CASCADE
        );"#,
            [],
        )?;

        // Note: name is to support https://wicg.github.io/storage-buckets/
        connection.execute(
            r#"CREATE TABLE IF NOT EXISTS buckets (
            id INTEGER PRIMARY KEY,
            shelf_id INTEGER NOT NULL UNIQUE,
            persisted BOOLEAN DEFAULT 0,
            name TEXT,
            mode TEXT,
            expires DATETIME,
            FOREIGN KEY (shelf_id) REFERENCES shelves(id) ON DELETE CASCADE
        );"#,
            [],
        )?;

        // Note: quota not in db, hardcoded at https://storage.spec.whatwg.org/#storage-endpoint-quota
        connection.execute(
            r#"CREATE TABLE IF NOT EXISTS bottles (
                    id INTEGER PRIMARY KEY,
                    bucket_id INTEGER NOT NULL,
                    identifier TEXT NOT NULL,  -- "idb", "ls", "opfs", "cache"
                    UNIQUE (bucket_id, identifier),
                    FOREIGN KEY (bucket_id) REFERENCES buckets(id) ON DELETE CASCADE
                );"#,
            [],
        )?;

        connection.execute(
            r#"CREATE TABLE IF NOT EXISTS databases (
                    id INTEGER PRIMARY KEY,
                    bottle_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    UNIQUE (bottle_id, name),
                    FOREIGN KEY (bottle_id) REFERENCES bottles(id) ON DELETE CASCADE
                );
                "#,
            [],
        )?;

        connection.execute(
            r#"CREATE TABLE IF NOT EXISTS directories (
                id INTEGER PRIMARY KEY,
                database_id INTEGER NOT NULL UNIQUE,
                path TEXT NOT NULL,
                FOREIGN KEY (database_id) REFERENCES databases(id) ON DELETE CASCADE
            );"#,
            [],
        )?;

        connection.execute_batch(
            r#"
                CREATE UNIQUE INDEX IF NOT EXISTS sheds_local_identity_idx
                ON sheds(storage_type)
                WHERE storage_type = 'local' AND browsing_context IS NULL;

                CREATE UNIQUE INDEX IF NOT EXISTS sheds_session_identity_idx
                ON sheds(storage_type, browsing_context)
                WHERE storage_type = 'session' AND browsing_context IS NOT NULL;

                CREATE UNIQUE INDEX IF NOT EXISTS shelves_origin_shed_identity_idx
                ON shelves(origin, shed_id);
            "#,
        )?;
        // TODO: Delete expired and non-persistent buckets on startup
        Ok(())
    }
}

fn ensure_storage_shed(
    storage_type: &StorageType,
    browsing_context: Option<String>,
    tx: &Transaction,
) -> rusqlite::Result<i64> {
    match browsing_context {
        Some(browsing_context) => {
            tx.execute(
                "INSERT INTO sheds (storage_type, browsing_context) VALUES (?1, ?2) ON CONFLICT DO NOTHING;",
                (storage_type.as_str(), browsing_context.as_str()),
            )?;

            tx.query_row(
                "SELECT id FROM sheds WHERE storage_type = ?1 AND browsing_context = ?2;",
                (storage_type.as_str(), browsing_context.as_str()),
                |row| row.get(0),
            )
        },
        None => {
            tx.execute(
                "INSERT INTO sheds (storage_type, browsing_context) VALUES (?1, NULL) ON CONFLICT DO NOTHING;",
                [storage_type.as_str()],
            )?;

            tx.query_row(
                "SELECT id FROM sheds WHERE storage_type = ?1 AND browsing_context IS NULL;",
                [storage_type.as_str()],
                |row| row.get(0),
            )
        },
    }
}

/// <https://storage.spec.whatwg.org/#create-a-storage-bucket>
fn create_a_storage_bucket(
    shelf_id: i64,
    storage_type: StorageType,
    tx: &Transaction,
) -> rusqlite::Result<i64> {
    // Step 1: Let bucket be null.
    // Step 2: If type is "local", then set bucket to a new local storage bucket.
    let bucket_id: i64 = if let StorageType::Local = storage_type {
        tx.query_row(
            "INSERT INTO buckets (mode, shelf_id) VALUES (?1, ?2)
             ON CONFLICT(shelf_id) DO UPDATE SET shelf_id = excluded.shelf_id
             RETURNING id;",
            [Mode::default().as_str(), &shelf_id.to_string()],
            |row| row.get(0),
        )?
    } else {
        // Step 3: Otherwise:
        // Step 3.1: Assert: type is "session".
        // Step 3.2: Set bucket to a new session storage bucket.
        tx.query_row(
            "INSERT INTO buckets (shelf_id) VALUES (?1)
             ON CONFLICT(shelf_id) DO UPDATE SET shelf_id = excluded.shelf_id
             RETURNING id;",
            [&shelf_id.to_string()],
            |row| row.get(0),
        )?
    };

    // Step 4: For each endpoint of registered storage endpoints whose types contain type,
    // set bucket’s bottle map[endpoint’s identifier] to
    // a new storage bottle whose quota is endpoint’s quota.

    // <https://storage.spec.whatwg.org/#registered-storage-endpoints>
    let registered_endpoints = match storage_type {
        StorageType::Local => vec![
            StorageIdentifier::Caches,
            StorageIdentifier::IndexedDB,
            StorageIdentifier::LocalStorage,
            StorageIdentifier::ServiceWorkerRegistrations,
        ],
        StorageType::Session => vec![StorageIdentifier::SessionStorage],
    };

    for identifier in registered_endpoints {
        tx.execute(
            "INSERT INTO bottles (bucket_id, identifier) VALUES (?1, ?2)
             ON CONFLICT(bucket_id, identifier) DO NOTHING;",
            (bucket_id, identifier.as_str()),
        )?;
    }

    // Step 5: Return bucket.
    Ok(bucket_id)
}

/// <https://storage.spec.whatwg.org/#create-a-storage-shelf>
fn create_a_storage_shelf(
    shed: i64,
    origin: &ImmutableOrigin,
    storage_type: StorageType,
    tx: &Transaction,
) -> rusqlite::Result<i64> {
    // Step 1: Let shelf be a new storage shelf.
    let shelf_id: i64 = tx.query_row(
        "INSERT INTO shelves (shed_id, origin) VALUES (?1, ?2)
         ON CONFLICT(shed_id, origin) DO UPDATE SET origin = excluded.origin
         RETURNING id;",
        [&shed.to_string(), &origin.ascii_serialization()],
        |row| row.get(0),
    )?;

    // Step 2: Set shelf’s bucket map["default"] to the result of running create a storage bucket with type.
    // Note: returning `shelf’s bucket map["default"]`, which is the `bucket_id`.
    create_a_storage_bucket(shelf_id, storage_type, tx)
}

/// <https://storage.spec.whatwg.org/#obtain-a-storage-shelf>
fn obtain_a_storage_shelf(
    shed: i64,
    origin: &ImmutableOrigin,
    storage_type: StorageType,
    tx: &Transaction,
) -> rusqlite::Result<i64> {
    // Step 1: Let key be the result of running obtain a storage key with environment.
    // Step 2: If key is failure, then return failure.

    // Step 3: If shed[key] does not exist,
    // then set shed[key] to the result of running create a storage shelf with type.
    // Note: method internally conditions on shed[key] not existing.
    let bucket_id = create_a_storage_shelf(shed, origin, storage_type, tx)?;

    // Step 4: Return shed[key].
    // Note: returning `shed[key]["default"]`, which is `bucket_id`.
    Ok(bucket_id)
}

impl RegistryEngine for SqliteEngine {
    type Error = rusqlite::Error;

    /// Create a database for the indexedDB endpoint.
    fn create_database(
        &mut self,
        bottle_id: i64,
        name: String,
    ) -> Result<PathBuf, ClientStorageErrorr<Self::Error>> {
        let tx = self.connection.transaction()?;

        let dir = Uuid::new_v4().to_string();
        let cluster = dir.chars().last().unwrap();
        let path = self
            .base_dir
            .join("bottles")
            .join(cluster.to_string())
            .join(dir);

        let path_str = path.to_str().ok_or_else(|| {
            ClientStorageErrorr::Internal(rusqlite::Error::InvalidParameterName(String::from(
                "path",
            )))
        })?;

        let database_id: i64 = tx
            .query_row(
                "INSERT INTO databases (bottle_id, name) VALUES (?1, ?2)
             ON CONFLICT(bottle_id, name) DO NOTHING
             RETURNING id;",
                (bottle_id, name),
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => ClientStorageErrorr::DatabaseAlreadyExists,
                e => ClientStorageErrorr::Internal(e),
            })?;

        tx.execute(
            "INSERT INTO directories (database_id, path) VALUES (?1, ?2);",
            (database_id, path_str),
        )?;

        std::fs::create_dir_all(&path).map_err(|_| ClientStorageErrorr::DirectoryCreationFailed)?;

        tx.commit()?;

        Ok(path)
    }

    /// Delete a database for the indexedDB endpoint.
    fn delete_database(
        &mut self,
        bottle_id: i64,
        name: String,
    ) -> Result<(), ClientStorageErrorr<Self::Error>> {
        let tx = self.connection.transaction()?;

        let database_id: i64 = tx.query_row(
            "SELECT id FROM databases WHERE bottle_id = ?1 AND name = ?2;",
            (bottle_id, name.clone()),
            |row| row.get(0),
        )?;

        let path: String = tx.query_row(
            "SELECT path FROM directories WHERE database_id = ?1;",
            [database_id],
            |row| row.get(0),
        )?;

        tx.execute(
            "DELETE FROM databases WHERE bottle_id = ?1 AND name = ?2;",
            (bottle_id, name),
        )?;

        if tx.changes() == 0 {
            return Err(ClientStorageErrorr::DatabaseDoesNotExist);
        }
        // Note: directory deleted through SQL cascade.

        // Delete the directory on disk
        std::fs::remove_dir_all(&path).map_err(|_| ClientStorageErrorr::DirectoryDeletionFailed)?;

        tx.commit()?;

        Ok(())
    }

    /// <https://storage.spec.whatwg.org/#obtain-a-storage-bottle-map>
    fn obtain_a_storage_bottle_map(
        &mut self,
        storage_type: StorageType,
        webview: WebViewId,
        storage_identifier: StorageIdentifier,
        origin: ImmutableOrigin,
        sender: &GenericSender<ClientStorageThreadMessage>,
    ) -> Result<StorageProxyMap, ClientStorageErrorr<Self::Error>> {
        let tx = self.connection.transaction()?;

        // Step 1: Let shed be null.
        let shed_id: i64 = match storage_type {
            StorageType::Local => {
                // Step 2: If type is "local", then set shed to the user agent’s storage shed.
                ensure_storage_shed(&storage_type, None, &tx)?
            },
            StorageType::Session => {
                // Step 3: Otherwise:
                // Step 3.1: Assert: type is "session".
                // Step 3.2: Set shed to environment’s global object’s associated Document’s
                // node navigable’s traversable navigable’s storage shed.
                // Note: using the browsing context of the webview as the traversable navigable.
                ensure_storage_shed(
                    &storage_type,
                    Some(Into::<BrowsingContextId>::into(webview).to_string()),
                    &tx,
                )?
            },
        };

        // Step 4: Let shelf be the result of running obtain a storage shelf,
        // with shed, environment, and type.
        // Step 5: If shelf is failure, then return failure.
        let bucket_id = obtain_a_storage_shelf(shed_id, &origin, storage_type, &tx)?;

        // Step 6: Let bucket be shelf’s bucket map["default"].
        // Done above with `bucket_id`.

        let bottle_id: i64 = tx.query_row(
            "SELECT id FROM bottles WHERE bucket_id = ?1 AND identifier = ?2;",
            (bucket_id, storage_identifier.as_str()),
            |row| row.get(0),
        )?;

        tx.commit()?;

        // Step 7: Let bottle be bucket’s bottle map[identifier].
        // Note: done with `bucket_id`.

        // Step 8: Let proxyMap be a new storage proxy map whose backing map is bottle’s map.
        // Step 9: Append proxyMap to bottle’s proxy map reference set.
        // Note: not doing the reference set part for now, not sure what it is useful for.

        // Step 10: Return proxyMap.
        Ok(StorageProxyMap {
            bottle_id,
            handle: ClientStorageThreadHandle::new(sender.clone()),
        })
    }
}

pub trait ClientStorageThreadFactory {
    fn new(config_dir: Option<PathBuf>) -> Self;
}

impl ClientStorageThreadFactory for ClientStorageThreadHandle {
    fn new(config_dir: Option<PathBuf>) -> ClientStorageThreadHandle {
        let (generic_sender, generic_receiver) = generic_channel::channel().unwrap();

        let storage_dir = config_dir
            .unwrap_or_else(|| {
                let tmp_dir = tempfile::tempdir().unwrap();
                tmp_dir.path().to_path_buf()
            })
            .join("clientstorage");
        std::fs::create_dir_all(&storage_dir)
            .expect("Failed to create ClientStorage storage directory");
        let sender_clone = generic_sender.clone();
        thread::Builder::new()
            .name("ClientStorageThread".to_owned())
            .spawn(move || {
                let engine = SqliteEngine::new(storage_dir).unwrap_or_else(|error| {
                    warn!("Failed to initialize ClientStorage engine into storage dir: {error:?}");
                    SqliteEngine::memory().unwrap()
                });
                ClientStorageThread::new(sender_clone, generic_receiver, engine).start();
            })
            .expect("Thread spawning failed");

        ClientStorageThreadHandle::new(generic_sender)
    }
}

struct ClientStorageThread<E: RegistryEngine> {
    receiver: GenericReceiver<ClientStorageThreadMessage>,
    sender: GenericSender<ClientStorageThreadMessage>,
    engine: E,
}

impl<E> ClientStorageThread<E>
where
    E: RegistryEngine,
{
    pub fn new(
        sender: GenericSender<ClientStorageThreadMessage>,
        receiver: GenericReceiver<ClientStorageThreadMessage>,
        engine: E,
    ) -> ClientStorageThread<E> {
        ClientStorageThread {
            sender,
            receiver,
            engine,
        }
    }

    pub fn start(&mut self) {
        while let Ok(message) = self.receiver.recv() {
            match message {
                ClientStorageThreadMessage::ObtainBottleMap {
                    storage_type,
                    storage_identifier,
                    webview,
                    origin,
                    sender,
                } => {
                    let result = self.engine.obtain_a_storage_bottle_map(
                        storage_type,
                        webview,
                        storage_identifier,
                        origin,
                        &self.sender,
                    );
                    let _ = sender.send(result.map_err(|e| format!("{:?}", e)));
                },
                ClientStorageThreadMessage::CreateDatabase {
                    bottle_id,
                    name,
                    sender,
                } => {
                    let result = self.engine.create_database(bottle_id, name);
                    let _ = sender.send(result.map_err(|e| format!("{:?}", e)));
                },
                ClientStorageThreadMessage::DeleteDatabase {
                    bottle_id,
                    name,
                    sender,
                } => {
                    let result = self.engine.delete_database(bottle_id, name);
                    let _ = sender.send(result.map_err(|e| format!("{:?}", e)));
                },
                ClientStorageThreadMessage::Exit(sender) => {
                    let _ = sender.send(());
                    break;
                },
            }
        }
    }
}
