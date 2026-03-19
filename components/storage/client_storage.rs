/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::fmt::Debug;
use std::path::PathBuf;
use std::thread;

use servo_base::generic_channel::{self, GenericReceiver, GenericSender};
use servo_base::id::{BrowsingContextId, WebViewId};
use rusqlite::{Connection, Transaction};
use servo_url::ImmutableOrigin;
use storage_traits::client_storage::{
    ClientStorageThreadHandle, ClientStorageThreadMessage, CreateBottleError, StorageIdentifier,
    StorageProxyMap, StorageType,
};
use uuid::Uuid;

trait RegistryEngine {
    type Error: Debug;
    fn create_database(
        &mut self,
        bottle_id: i64,
        name: String,
    ) -> Result<PathBuf, CreateBottleError<Self::Error>>;
    fn delete_database(
        &mut self,
        bottle_id: i64,
        name: String,
    ) -> Result<(), CreateBottleError<Self::Error>>;
    fn obtain_a_storage_bottle_map(
        &mut self,
        storage_type: StorageType,
        webview: WebViewId,
        storage_identifier: StorageIdentifier,
        origin: ImmutableOrigin,
        sender: &GenericSender<ClientStorageThreadMessage>,
    ) -> Result<StorageProxyMap, CreateBottleError<Self::Error>>;
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
        connection.execute(
            r#"CREATE TABLE IF NOT EXISTS shelves (
                    id INTEGER PRIMARY KEY,
                    origin TEXT NOT NULL,
                    shed_id INTEGER NOT NULL,
                );"#,
            [],
        )?;
        connection.execute(
            r#"CREATE TABLE IF NOT EXISTS buckets (
                    id INTEGER PRIMARY KEY,
                    shelf_id INTEGER NOT NULL,
                    storage_type TEXT NOT NULL,
                    persisted BOOLEAN DEFAULT 0,
                    quota INTEGER,
                    expires DATETIME,
                    UNIQUE (shelf_id, name),
                    FOREIGN KEY (shelf_id) REFERENCES shelves(id) ON DELETE CASCADE
                );"#,
            [],
        )?;
        connection.execute(
            r#"CREATE TABLE IF NOT EXISTS bottles (
                    id INTEGER PRIMARY KEY,
                    bucket_id INTEGER NOT NULL,
                    identifier TEXT NOT NULL,  -- "idb", "ls", "opfs", "cache"
                    quota INTEGER,
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
        // TODO: Delete expired and non-persistent buckets on startup
        Ok(())
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
    // Step 3: Otherwise:
    // Step 3.1: Assert: type is "session".
    // Step 3.2: Set bucket to a new session storage bucket.
    // Note: done with `StorageType`.
    tx.execute(
        "INSERT OR IGNORE INTO buckets (storage_type, shelf_id) VALUES (?1, ?2);",
        [storage_type.as_str(), &shelf_id.to_string()],
    )?;

    let bucket_id: i64 = tx.query_row(
        "SELECT id FROM buckets WHERE storage_type = ?1 AND shelf_id = ?2;",
        [storage_type.as_str(), &shelf_id.to_string()],
        |row| row.get(0),
    )?;

    // Step 4: For each endpoint of registered storage endpoints whose types contain type,
    // set bucket’s bottle map[endpoint’s identifier] to
    // a new storage bottle whose quota is endpoint’s quota.

    let registered_endpoints = match storage_type {
        StorageType::Local => vec![
            "caches",
            "indexedDB",
            "localStorage",
            "serviceWorkerRegistrations",
        ],
        StorageType::Session => vec!["sessionStorage"],
    };

    for identifier in registered_endpoints {
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM bottles WHERE bucket_id = ?1 AND identifier = ?2);",
            (bucket_id, identifier),
            |row| row.get(0),
        )?;

        // TODO: quota.
        if !exists {
            tx.execute(
                "INSERT INTO bottles (bucket_id, identifier) VALUES (?1, ?2);",
                (bucket_id, identifier),
            )?;
        }
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
    tx.execute(
        "INSERT OR IGNORE INTO shelves (origin, shed_id) VALUES (?1, ?2);",
        [origin.ascii_serialization(), shed.to_string()],
    )?;
    let shelf_id: i64 = tx.query_row(
        "SELECT id FROM shelves WHERE origin = ?1;",
        [&origin.ascii_serialization()],
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
    // Note: for now just using the origin as the key.
    // TODO: implement https://storage.spec.whatwg.org/#obtain-a-storage-key

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
    ) -> Result<PathBuf, CreateBottleError<Self::Error>> {
        let tx = self.connection.transaction()?;

        // Ensure no duplicate database
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM databases WHERE bottle_id = ?1 AND name = ?2);",
            (bottle_id, name.clone()),
            |row| row.get(0),
        )?;

        if exists {
            return Err(CreateBottleError::DatabaseAlreadyExists);
        }

        // Cluster directory path by last character of UUID
        let dir = Uuid::new_v4().to_string();
        // UUID will always have at least one character
        let cluster = dir.chars().last().unwrap();
        let path = self
            .base_dir
            .join("bottles")
            .join(cluster.to_string())
            .join(dir);

        tx.execute(
            "INSERT INTO databases (bottle_id, name) VALUES (?1, ?2);",
            (bottle_id, name.clone()),
        )?;

        let database_id: i64 = tx.query_row(
            "SELECT id FROM databases WHERE bottle_id = ?1 AND name = ?2;",
            (bottle_id, name),
            |row| row.get(0),
        )?;

        let path_str = path.to_str().ok_or_else(|| {
            CreateBottleError::Internal(rusqlite::Error::InvalidParameterName(String::from("path")))
        })?;
        tx.execute(
            "INSERT INTO directories (database_id, path) VALUES (?1, ?2);",
            (database_id, path_str),
        )?;

        // Create the directory on disk
        std::fs::create_dir_all(&path).map_err(|_| CreateBottleError::DirectoryCreationFailed)?;

        tx.commit()?;

        Ok(path)
    }

    /// Delete a database for the indexedDB endpoint.
    fn delete_database(
        &mut self,
        bottle_id: i64,
        name: String,
    ) -> Result<(), CreateBottleError<Self::Error>> {
        let tx = self.connection.transaction()?;

        // Ensure no duplicate database
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM databases WHERE bottle_id = ?1 AND name = ?2);",
            (bottle_id, name.clone()),
            |row| row.get(0),
        )?;

        if !exists {
            return Err(CreateBottleError::DatabaseDoesNotExist);
        }

        let database_id: i64 = tx.query_row(
            "SELECT id FROM databases WHERE bottle_id = ?1 AND name = ?2;",
            (bottle_id, name.clone()),
            |row| row.get(0),
        )?;

        let path: String = tx.query_row(
            "SELECT path FROM databases WHERE bottle_id = ?1 AND name = ?2;",
            (bottle_id, name.clone()),
            |row| row.get(1),
        )?;

        tx.execute(
            "DELETE FROM databases (bottle_id, name) VALUES (?1, ?2);",
            (bottle_id, name),
        )?;

        let path_str = path.to_string();
        tx.execute(
            "DELETE FROM directories (database_id, path) VALUES (?1, ?2);",
            (database_id, path_str),
        )?;

        // Delete the directory on disk
        std::fs::create_dir_all(&path).map_err(|_| CreateBottleError::DirectoryCreationFailed)?;

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
    ) -> Result<StorageProxyMap, CreateBottleError<Self::Error>> {
        let tx = self.connection.transaction()?;

        // Step 1: Let shed be null.
        let shed_id: i64 = match storage_type {
            StorageType::Local => {
                // Step 2: If type is "local", then set shed to the user agent’s storage shed.
                tx.execute(
                    "INSERT OR IGNORE INTO sheds (storage_type) VALUES (?1);",
                    [storage_type.as_str()],
                )?;
                tx.query_row(
                    "SELECT id FROM sheds WHERE storage_type = ?1;",
                    ["local"],
                    |row| row.get(0),
                )?
            },
            StorageType::Session => {
                // Step 3: Otherwise:
                // Step 3.1: Assert: type is "session".
                // Step 3.2: Set shed to environment’s global object’s associated Document’s
                // node navigable’s traversable navigable’s storage shed.
                // Note: using the browsing context of the webview as the traversable navigable.
                tx.execute(
                    "INSERT OR IGNORE INTO sheds (storage_type, browsing_context) VALUES (?1, ?2);",
                    [
                        storage_type.as_str(),
                        &Into::<BrowsingContextId>::into(webview).to_string(),
                    ],
                )?;
                tx.query_row(
                    "SELECT id FROM sheds WHERE browsing_context = ?1;",
                    [&Into::<BrowsingContextId>::into(webview).to_string()],
                    |row| row.get(0),
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
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clientstorage");
        std::fs::create_dir_all(&storage_dir)
            .expect("Failed to create ClientStorage storage directory");
        let sender_clone = generic_sender.clone();
        thread::Builder::new()
            .name("ClientStorageThread".to_owned())
            .spawn(move || {
                let engine = SqliteEngine::new(storage_dir)
                    .expect("Failed to initialize ClientStorage registry engine");
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
