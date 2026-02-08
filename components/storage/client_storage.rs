/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::thread;

use base::generic_channel::{self, GenericReceiver, GenericSender};
use rusqlite::{Connection, OptionalExtension};
use servo_url::ImmutableOrigin;
use storage_traits::client_storage::{
    Bottle, BottleIdent, Bucket, BucketIdent, ClientStorageThreadMessage, CreateBottleError,
    CreateBucketError,
};
use uuid::Uuid;

trait RegistryEngine {
    type Error: std::fmt::Debug;

    fn create_bucket(
        &mut self,
        shelf: ImmutableOrigin,
        bucket: Bucket,
    ) -> Result<(), CreateBucketError<Self::Error>>;
    fn create_bottle(
        &mut self,
        shelf: ImmutableOrigin,
        bucket: BucketIdent,
        bottle: Bottle,
    ) -> Result<PathBuf, CreateBottleError<Self::Error>>;
    fn open_bottle(
        &mut self,
        shelf: ImmutableOrigin,
        bucket: BucketIdent,
        bottle: BottleIdent,
    ) -> Result<PathBuf, Self::Error>;
    fn delete_shelf(&mut self, shelf: ImmutableOrigin) -> Result<(), Self::Error>;
    fn delete_bucket(
        &mut self,
        shelf: ImmutableOrigin,
        bucket: BucketIdent,
    ) -> Result<(), Self::Error>;
    fn delete_bottle(
        &mut self,
        shelf: ImmutableOrigin,
        bucket: BucketIdent,
        bottle: BottleIdent,
    ) -> Result<(), Self::Error>;
    fn delete_all(&mut self) -> Result<(), Self::Error>;
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
            r#"CREATE TABLE IF NOT EXISTS shelves (
                    id INTEGER PRIMARY KEY,
                    origin TEXT NOT NULL UNIQUE
                );"#,
            [],
        )?;
        connection.execute(
            r#"CREATE TABLE IF NOT EXISTS buckets (
                    id INTEGER PRIMARY KEY,
                    shelf_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
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

    /// Creates a shelf if it does not already exist.
    /// Creates the default bucket if that does not already exist.
    fn create_shelf(&mut self, shelf: &ImmutableOrigin) -> rusqlite::Result<i64> {
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT OR IGNORE INTO shelves (origin) VALUES (?1);",
            [shelf.ascii_serialization()],
        )?;
        let shelf_id: i64 = tx.query_row(
            "SELECT id FROM shelves WHERE origin = ?1;",
            (&shelf.ascii_serialization(),),
            |row| row.get(0),
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO buckets (shelf_id, name, persisted) VALUES (?1, 'default', 1);",
            [shelf_id],
        )?;
        tx.commit()?;
        Ok(shelf_id)
    }
}

impl RegistryEngine for SqliteEngine {
    type Error = rusqlite::Error;

    fn create_bucket(
        &mut self,
        shelf: ImmutableOrigin,
        bucket: Bucket,
    ) -> Result<(), CreateBucketError<Self::Error>> {
        let shelf_id = self.create_shelf(&shelf)?;
        // Check if bucket already exists
        let exists: bool = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM buckets WHERE shelf_id = ?1 AND name = ?2);",
            (shelf_id, bucket.bucket_type.as_str()),
            |row| row.get(0),
        )?;
        if exists {
            return Err(CreateBucketError::BucketAlreadyExists);
        }
        self.connection.execute(
            "INSERT INTO buckets (shelf_id, name, persisted, quota, expires) VALUES (?1, ?2, ?3, ?4, ?5);",
            (
                shelf_id,
                bucket.bucket_type.as_str(),
                bucket.persistent,
                bucket.quota,
                bucket.expiration,
            ),
        )?;
        Ok(())
    }

    fn create_bottle(
        &mut self,
        shelf: ImmutableOrigin,
        bucket: BucketIdent,
        bottle: Bottle,
    ) -> Result<PathBuf, CreateBottleError<Self::Error>> {
        let shelf_id = self.create_shelf(&shelf)?;
        // Get bucket id
        let bucket_id: i64 = match self
            .connection
            .query_row(
                "SELECT id FROM buckets WHERE shelf_id = ?1 AND name = ?2;",
                (shelf_id, bucket.as_str()),
                |row| row.get(0),
            )
            .optional()
        {
            Ok(Some(id)) => id,
            Ok(None) => return Err(CreateBottleError::BucketDoesNotExist),
            Err(e) => return Err(CreateBottleError::Internal(e)),
        };

        let tx = self.connection.transaction()?;

        // Ensure no duplicate bottle
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM bottles WHERE bucket_id = ?1 AND identifier = ?2);",
            (bucket_id, bottle.bottle_type.type_str()),
            |row| row.get(0),
        )?;
        if !exists {
            tx.execute(
                "INSERT INTO bottles (bucket_id, identifier, quota) VALUES (?1, ?2, ?3);",
                (bucket_id, bottle.bottle_type.type_str(), bottle.quota),
            )?;
        }

        let bottle_id: i64 = tx.query_row(
            "SELECT id FROM bottles WHERE bucket_id = ?1 AND identifier = ?2;",
            (bucket_id, bottle.bottle_type.type_str()),
            |row| row.get(0),
        )?;

        // Ensure no duplicate database
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM databases WHERE bottle_id = ?1 AND name = ?2);",
            (bottle_id, bottle.bottle_type.database_name()),
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
            (bottle_id, bottle.bottle_type.database_name()),
        )?;

        let database_id: i64 = tx.query_row(
            "SELECT id FROM databases WHERE bottle_id = ?1 AND name = ?2;",
            (bottle_id, bottle.bottle_type.database_name()),
            |row| row.get(0),
        )?;

        let path_str = path.to_str().ok_or_else(|| {
            CreateBottleError::Internal(rusqlite::Error::InvalidParameterName(String::from("path")))
        })?;
        tx.execute(
            "INSERT INTO directories (database_id, path) VALUES (?1, ?2);",
            (database_id, path_str),
        )?;

        tx.commit()?;
        // Create the directory on disk
        std::fs::create_dir_all(&path).map_err(|_| CreateBottleError::DirectoryCreationFailed)?;
        Ok(path)
    }

    fn open_bottle(
        &mut self,
        shelf: ImmutableOrigin,
        bucket: BucketIdent,
        bottle: BottleIdent,
    ) -> Result<PathBuf, Self::Error> {
        let shelf_id: i64 = self.connection.query_row(
            "SELECT id FROM shelves WHERE origin = ?1;",
            (&shelf.ascii_serialization(),),
            |row| row.get(0),
        )?;
        let bucket_id: i64 = self.connection.query_row(
            "SELECT id FROM buckets WHERE shelf_id = ?1 AND name = ?2;",
            (shelf_id, bucket.as_str()),
            |row| row.get(0),
        )?;
        let bottle_id: i64 = self.connection.query_row(
            "SELECT id FROM bottles WHERE bucket_id = ?1 AND identifier = ?2;",
            (bucket_id, bottle.type_str()),
            |row| row.get(0),
        )?;
        let database_id: i64 = self.connection.query_row(
            "SELECT id FROM databases WHERE bottle_id = ?1 AND name = ?2;",
            (bottle_id, bottle.database_name()),
            |row| row.get(0),
        )?;
        let path_str: String = self.connection.query_row(
            "SELECT path FROM directories WHERE database_id = ?1;",
            (database_id,),
            |row| row.get(0),
        )?;
        Ok(PathBuf::from(path_str))
    }

    fn delete_shelf(&mut self, shelf: ImmutableOrigin) -> Result<(), Self::Error> {
        self.connection.execute(
            "DELETE FROM shelves WHERE origin = ?1;",
            [&shelf.ascii_serialization()],
        )?;
        Ok(())
    }

    fn delete_bucket(
        &mut self,
        shelf: ImmutableOrigin,
        bucket: BucketIdent,
    ) -> Result<(), Self::Error> {
        let shelf_id: i64 = self.connection.query_row(
            "SELECT id FROM shelves WHERE origin = ?1;",
            (&shelf.ascii_serialization(),),
            |row| row.get(0),
        )?;
        self.connection.execute(
            "DELETE FROM buckets WHERE shelf_id = ?1 AND name = ?2;",
            (shelf_id, bucket.as_str()),
        )?;
        Ok(())
    }

    fn delete_bottle(
        &mut self,
        shelf: ImmutableOrigin,
        bucket: BucketIdent,
        bottle: BottleIdent,
    ) -> Result<(), Self::Error> {
        let shelf_id: i64 = self.connection.query_row(
            "SELECT id FROM shelves WHERE origin = ?1;",
            (&shelf.ascii_serialization(),),
            |row| row.get(0),
        )?;
        let bucket_id: i64 = self.connection.query_row(
            "SELECT id FROM buckets WHERE shelf_id = ?1 AND name = ?2;",
            (shelf_id, bucket.as_str()),
            |row| row.get(0),
        )?;
        self.connection.execute(
            "DELETE FROM bottles WHERE bucket_id = ?1 AND identifier = ?2;",
            (bucket_id, bottle.type_str()),
        )?;
        Ok(())
    }

    fn delete_all(&mut self) -> Result<(), Self::Error> {
        self.connection.execute("DELETE FROM shelves;", [])?;
        Ok(())
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
        let clone_dir = storage_dir.clone();
        thread::Builder::new()
            .name("ClientStorageThread".to_owned())
            .spawn(move || {
                let engine = SqliteEngine::new(clone_dir)
                    .expect("Failed to initialize ClientStorage registry engine");
                ClientStorageThread::new(generic_receiver, engine).start();
            })
            .expect("Thread spawning failed");

        ClientStorageThreadHandle::new(generic_sender)
    }
}

struct ClientStorageThread<E: RegistryEngine> {
    receiver: GenericReceiver<ClientStorageThreadMessage>,
    engine: E,
}

impl<E> ClientStorageThread<E>
where
    E: RegistryEngine,
{
    pub fn new(
        receiver: GenericReceiver<ClientStorageThreadMessage>,
        engine: E,
    ) -> ClientStorageThread<E> {
        ClientStorageThread { receiver, engine }
    }

    pub fn start(&mut self) {
        while let Ok(message) = self.receiver.recv() {
            match message {
                ClientStorageThreadMessage::CreateBucket {
                    shelf,
                    bucket,
                    sender,
                } => {
                    let result = self.engine.create_bucket(shelf, bucket);
                    let _ = sender.send(result.map_err(|e| match e {
                        CreateBucketError::BucketAlreadyExists => {
                            CreateBucketError::BucketAlreadyExists
                        },
                        CreateBucketError::Internal(err) => {
                            CreateBucketError::Internal(format!("{:?}", err))
                        },
                    }));
                },
                ClientStorageThreadMessage::CreateBottle {
                    shelf,
                    bucket,
                    bottle,
                    sender,
                } => {
                    let result = self.engine.create_bottle(shelf, bucket, bottle);
                    let _ = sender.send(result.map_err(|e| match e {
                        CreateBottleError::BottleAlreadyExists => {
                            CreateBottleError::BottleAlreadyExists
                        },
                        CreateBottleError::BucketDoesNotExist => {
                            CreateBottleError::BucketDoesNotExist
                        },
                        CreateBottleError::DatabaseAlreadyExists => {
                            CreateBottleError::DatabaseAlreadyExists
                        },
                        CreateBottleError::DirectoryCreationFailed => {
                            CreateBottleError::DirectoryCreationFailed
                        },
                        CreateBottleError::Internal(err) => {
                            CreateBottleError::Internal(format!("{:?}", err))
                        },
                    }));
                },
                ClientStorageThreadMessage::OpenBottle {
                    shelf,
                    bucket,
                    bottle,
                    sender,
                } => {
                    let result = self.engine.open_bottle(shelf, bucket, bottle);
                    let _ = sender.send(result.map_err(|e| format!("{:?}", e)));
                },
                ClientStorageThreadMessage::DeleteShelf { shelf, sender } => {
                    let result = self.engine.delete_shelf(shelf);
                    let _ = sender.send(result.map_err(|e| format!("{:?}", e)));
                },
                ClientStorageThreadMessage::DeleteBucket {
                    shelf,
                    bucket,
                    sender,
                } => {
                    let result = self.engine.delete_bucket(shelf, bucket);
                    let _ = sender.send(result.map_err(|e| format!("{:?}", e)));
                },
                ClientStorageThreadMessage::DeleteBottle {
                    shelf,
                    bucket,
                    bottle,
                    sender,
                } => {
                    let result = self.engine.delete_bottle(shelf, bucket, bottle);
                    let _ = sender.send(result.map_err(|e| format!("{:?}", e)));
                },
                ClientStorageThreadMessage::DeleteAll { sender } => {
                    let result = self.engine.delete_all();
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

#[derive(Clone)]
pub struct ClientStorageThreadHandle {
    sender: GenericSender<ClientStorageThreadMessage>,
}

impl ClientStorageThreadHandle {
    pub fn new(sender: GenericSender<ClientStorageThreadMessage>) -> Self {
        ClientStorageThreadHandle { sender }
    }

    pub fn create_bottle(
        &self,
        shelf: ImmutableOrigin,
        bucket: BucketIdent,
        bottle: Bottle,
    ) -> GenericReceiver<Result<PathBuf, CreateBottleError<String>>> {
        let (sender, receiver) = generic_channel::channel().unwrap();
        let message = ClientStorageThreadMessage::CreateBottle {
            shelf,
            bucket,
            bottle,
            sender,
        };
        self.sender.send(message).unwrap();
        receiver
    }

    pub fn open_bottle(
        &self,
        shelf: ImmutableOrigin,
        bucket: BucketIdent,
        bottle: BottleIdent,
    ) -> GenericReceiver<Result<PathBuf, String>> {
        let (sender, receiver) = generic_channel::channel().unwrap();
        let message = ClientStorageThreadMessage::OpenBottle {
            shelf,
            bucket,
            bottle,
            sender,
        };
        self.sender.send(message).unwrap();
        receiver
    }

    pub fn delete_all(&self) -> GenericReceiver<Result<(), String>> {
        let (sender, receiver) = generic_channel::channel().unwrap();
        let message = ClientStorageThreadMessage::DeleteAll { sender };
        self.sender.send(message).unwrap();
        receiver
    }
}

impl From<ClientStorageThreadHandle> for GenericSender<ClientStorageThreadMessage> {
    fn from(handle: ClientStorageThreadHandle) -> Self {
        handle.sender
    }
}

impl Deref for ClientStorageThreadHandle {
    type Target = GenericSender<ClientStorageThreadMessage>;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

impl DerefMut for ClientStorageThreadHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sender
    }
}
