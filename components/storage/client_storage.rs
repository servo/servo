/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::fmt::Debug;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, thread};

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

/// <https://storage.spec.whatwg.org/#storage-quota>
/// The storage quota of a storage shelf is an implementation-defined conservative estimate of the
/// total amount of byttes it can hold. We use 10 GiB per shelf, matching Firefox's documented
/// limit (<https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria>).
const STORAGE_SHELF_QUOTA_BYTES: u64 = 10 * 1024 * 1024 * 1024;

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
    fn persisted(&mut self, origin: ImmutableOrigin) -> Result<bool, String>;
    fn persist(
        &mut self,
        origin: ImmutableOrigin,
        permission_granted: bool,
    ) -> Result<bool, String>;
    fn estimate(&mut self, origin: ImmutableOrigin) -> Result<(u64, u64), String>;
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
    // Step 1. Let bucket be null.
    // Step 2. If type is "local", then set bucket to a new local storage bucket.
    let bucket_id: i64 = if let StorageType::Local = storage_type {
        tx.query_row(
            "INSERT INTO buckets (mode, shelf_id) VALUES (?1, ?2)
             ON CONFLICT(shelf_id) DO UPDATE SET shelf_id = excluded.shelf_id
             RETURNING id;",
            [Mode::default().as_str(), &shelf_id.to_string()],
            |row| row.get(0),
        )?
    } else {
        // Step 3. Otherwise:
        // Step 3.1. Assert: type is "session".
        // Step 3.2. Set bucket to a new session storage bucket.
        tx.query_row(
            "INSERT INTO buckets (shelf_id) VALUES (?1)
             ON CONFLICT(shelf_id) DO UPDATE SET shelf_id = excluded.shelf_id
             RETURNING id;",
            [&shelf_id.to_string()],
            |row| row.get(0),
        )?
    };

    // Step 4. For each endpoint of registered storage endpoints whose types contain type,
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

    // Step 5. Return bucket.
    Ok(bucket_id)
}

/// <https://storage.spec.whatwg.org/#create-a-storage-shelf>
fn create_a_storage_shelf(
    shed: i64,
    origin: &ImmutableOrigin,
    storage_type: StorageType,
    tx: &Transaction,
) -> rusqlite::Result<StorageShelf> {
    // To create a storage shelf, given a storage type type, run these steps:
    // Step 1. Let shelf be a new storage shelf.
    // Step 2.  Set shelf’s bucket map["default"] to the result of running create a storage bucket with type.
    let shelf_id: i64 = tx.query_row(
        "INSERT INTO shelves (shed_id, origin) VALUES (?1, ?2)
         ON CONFLICT(shed_id, origin) DO UPDATE SET origin = excluded.origin
         RETURNING id;",
        [&shed.to_string(), &origin.ascii_serialization()],
        |row| row.get(0),
    )?;

    // Step 3. Return shelf.
    Ok(StorageShelf {
        default_bucket_id: create_a_storage_bucket(shelf_id, storage_type, tx)?,
    })
}

/// <https://storage.spec.whatwg.org/#obtain-a-storage-shelf>
fn obtain_a_storage_shelf(
    shed: i64,
    origin: &ImmutableOrigin,
    storage_type: StorageType,
    tx: &Transaction,
) -> rusqlite::Result<StorageShelf> {
    create_a_storage_shelf(shed, origin, storage_type, tx)
}

/// <https://storage.spec.whatwg.org/#storage-shelf>
///
/// A storage shelf exists for each storage key within a storage shed. It holds a bucket map, which
/// is a map of strings to storage buckets.
struct StorageShelf {
    default_bucket_id: i64,
}

/// <https://storage.spec.whatwg.org/#obtain-a-local-storage-shelf>
///
/// To obtain a local storage shelf, given an environment settings object environment, return the
/// result of running obtain a storage shelf with the user agent’s storage shed, environment, and
/// "local".
fn obtain_a_local_storage_shelf(
    origin: &ImmutableOrigin,
    tx: &Transaction,
) -> Result<StorageShelf, String> {
    if !origin.is_tuple() {
        return Err("Storage is unavailable for opaque origins".to_owned());
    }

    let shed =
        ensure_storage_shed(&StorageType::Local, None, tx).map_err(|error| error.to_string())?;
    obtain_a_storage_shelf(shed, origin, StorageType::Local, tx).map_err(|error| error.to_string())
}

/// <https://storage.spec.whatwg.org/#bucket-mode>
///
/// A local storage bucket has a mode, which is "best-effort" or "persistent". It is initially
/// "best-effort".
fn bucket_mode(bucket_id: i64, tx: &Transaction) -> rusqlite::Result<Mode> {
    let mode: String = tx.query_row(
        "SELECT mode FROM buckets WHERE id = ?1;",
        [bucket_id],
        |row| row.get(0),
    )?;
    Ok(Mode::from_str(&mode).unwrap_or_default())
}

/// <https://storage.spec.whatwg.org/#dom-storagemanager-persist>
///
/// Set bucket’s mode to "persistent".
fn set_bucket_mode(bucket_id: i64, mode: Mode, tx: &Transaction) -> rusqlite::Result<()> {
    tx.execute(
        "UPDATE buckets SET mode = ?1, persisted = ?2 WHERE id = ?3;",
        (mode.as_str(), matches!(mode, Mode::Persistent), bucket_id),
    )?;
    Ok(())
}

/// <https://storage.spec.whatwg.org/#storage-usage>
///
/// The storage usage of a storage shelf is an implementation-defined rough estimate of the amount
/// of bytes used by it.
///
/// This cannot be an exact amount as user agents might, and are encouraged to, use deduplication,
/// compression, and other techniques that obscure exactly how much bytes a storage shelf uses.
fn storage_usage_for_bucket(bucket_id: i64, tx: &Transaction) -> Result<u64, String> {
    let mut stmt = tx
        .prepare(
            "SELECT directories.path
             FROM directories
             JOIN databases ON directories.database_id = databases.id
             JOIN bottles ON databases.bottle_id = bottles.id
             WHERE bottles.bucket_id = ?1;",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([bucket_id], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;

    let mut usage = 0_u64;
    for path in rows {
        usage += directory_size(&PathBuf::from(path.map_err(|error| error.to_string())?))?;
    }
    Ok(usage)
}

/// <https://storage.spec.whatwg.org/#storage-quota>
///
/// The storage quota of a storage shelf is an implementation-defined conservative estimate of the
/// total amount of bytes it can hold. This amount should be less than the total storage space on
/// the device. It must not be a function of the available storage space on the device.
///
/// User agents are strongly encouraged to consider navigation frequency, recency of visits,
/// bookmarking, and permission for "persistent-storage" when determining quotas.
///
/// Directly or indirectly revealing available storage space can lead to fingerprinting and leaking
/// information outside the scope of the origin involved.
fn storage_quota_for_bucket(_bucket_id: i64, _tx: &Transaction) -> Result<u64, String> {
    Ok(STORAGE_SHELF_QUOTA_BYTES)
}

/// <https://storage.spec.whatwg.org/#storage-usage>
///
/// The storage usage of a storage shelf is an implementation-defined rough estimate of the amount
/// of bytes used by it.
fn directory_size(path: &PathBuf) -> Result<u64, String> {
    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }

    if !metadata.is_dir() {
        return Ok(0);
    }

    let mut size = 0_u64;
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        size += directory_size(&entry.path())?;
    }
    Ok(size)
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

        // Step 1. Let shed be null.
        let shed_id: i64 = match storage_type {
            StorageType::Local => {
                // Step 2. If type is "local", then set shed to the user agent’s storage shed.
                ensure_storage_shed(&storage_type, None, &tx)?
            },
            StorageType::Session => {
                // Step 3. Otherwise:
                // Step 3.1. Assert: type is "session".
                // Step 3.2. Set shed to environment’s global object’s associated Document’s node
                // navigable’s traversable navigable’s storage shed.
                ensure_storage_shed(
                    &storage_type,
                    Some(Into::<BrowsingContextId>::into(webview).to_string()),
                    &tx,
                )?
            },
        };

        // Step 4. Let shelf be the result of running obtain a storage shelf, with shed,
        // environment, and type.
        // Step 5. If shelf is failure, then return failure.
        let shelf = obtain_a_storage_shelf(shed_id, &origin, storage_type, &tx)?;

        // Step 6. Let bucket be shelf’s bucket map["default"].
        let bucket_id = shelf.default_bucket_id;

        let bottle_id: i64 = tx.query_row(
            "SELECT id FROM bottles WHERE bucket_id = ?1 AND identifier = ?2;",
            (bucket_id, storage_identifier.as_str()),
            |row| row.get(0),
        )?;

        tx.commit()?;

        // Step 7. Let bottle be bucket’s bottle map[identifier].

        // Step 8. Let proxyMap be a new storage proxy map whose backing map is bottle’s map.
        // Step 9. Append proxyMap to bottle’s proxy map reference set.
        // Step 10. Return proxyMap.
        Ok(StorageProxyMap {
            bottle_id,
            handle: ClientStorageThreadHandle::new(sender.clone()),
        })
    }

    fn persisted(&mut self, origin: ImmutableOrigin) -> Result<bool, String> {
        let tx = self
            .connection
            .transaction()
            .map_err(|error| error.to_string())?;

        // <https://storage.spec.whatwg.org/#dom-storagemanager-persisted>
        // Let shelf be the result of running obtain a local storage shelf with this’s relevant
        // settings object.
        let shelf = obtain_a_local_storage_shelf(&origin, &tx)?;

        // Let persisted be true if shelf’s bucket map["default"]'s mode is "persistent";
        // otherwise false.
        // It will be false when there’s an internal error.
        let persisted = bucket_mode(shelf.default_bucket_id, &tx)
            .is_ok_and(|mode| mode == Mode::Persistent) &&
            tx.commit().is_ok();

        Ok(persisted)
    }

    fn persist(
        &mut self,
        origin: ImmutableOrigin,
        permission_granted: bool,
    ) -> Result<bool, String> {
        let tx = self
            .connection
            .transaction()
            .map_err(|error| error.to_string())?;

        // <https://storage.spec.whatwg.org/#dom-storagemanager-persist>
        // Let shelf be the result of running obtain a local storage shelf with this’s relevant
        // settings object.
        let shelf = obtain_a_local_storage_shelf(&origin, &tx)?;

        // Let bucket be shelf’s bucket map["default"].
        let bucket_id = shelf.default_bucket_id;

        // Let persisted be true if bucket’s mode is "persistent"; otherwise false.
        // It will be false when there’s an internal error.
        let mut persisted = bucket_mode(bucket_id, &tx).is_ok_and(|mode| mode == Mode::Persistent);

        // If persisted is false and permission is "granted", then:
        // Set bucket’s mode to "persistent".
        // If there was no internal error, then set persisted to true.
        if !persisted && permission_granted {
            persisted = set_bucket_mode(bucket_id, Mode::Persistent, &tx).is_ok();
        }

        if tx.commit().is_err() {
            persisted = false;
        }

        Ok(persisted)
    }

    fn estimate(&mut self, origin: ImmutableOrigin) -> Result<(u64, u64), String> {
        let tx = self
            .connection
            .transaction()
            .map_err(|error| error.to_string())?;

        // <https://storage.spec.whatwg.org/#dom-storagemanager-estimate>
        // Let shelf be the result of running obtain a local storage shelf with this’s relevant
        // settings object.
        let shelf = obtain_a_local_storage_shelf(&origin, &tx)?;

        // Let usage be storage usage for shelf.
        let usage = storage_usage_for_bucket(shelf.default_bucket_id, &tx)?;
        // Let quota be storage quota for shelf.
        let quota = storage_quota_for_bucket(shelf.default_bucket_id, &tx)?;

        tx.commit().map_err(|error| error.to_string())?;

        Ok((usage, quota))
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
                ClientStorageThreadMessage::Persisted { origin, sender } => {
                    let _ = sender.send(self.engine.persisted(origin));
                },
                ClientStorageThreadMessage::Persist {
                    origin,
                    permission_granted,
                    sender,
                } => {
                    let _ = sender.send(self.engine.persist(origin, permission_granted));
                },
                ClientStorageThreadMessage::Estimate { origin, sender } => {
                    let _ = sender.send(self.engine.estimate(origin));
                },
                ClientStorageThreadMessage::Exit(sender) => {
                    let _ = sender.send(());
                    break;
                },
            }
        }
    }
}
