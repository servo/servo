/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ipc_channel::ipc::IpcSender;
use log::{error, info};
use net_traits::indexeddb_thread::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, BackendError, BackendResult,
    CreateObjectResult, IndexedDBKeyRange, IndexedDBTxnMode, KeyPath, PutItemResult,
};
use rusqlite::{Connection, Error, OptionalExtension, params};
use sea_query::{Condition, Expr, ExprTrait, IntoCondition, SqliteQueryBuilder};
use serde::Serialize;
use tokio::sync::oneshot;

use crate::indexeddb::engines::{KvsEngine, KvsTransaction, SanitizedName};
use crate::indexeddb::idb_thread::IndexedDBDescription;
use crate::resource_thread::CoreResourceThreadPool;

mod create;
mod database_model;
mod object_data_model;
mod object_store_index_model;
mod object_store_model;

// These pragmas need to be set once
const DB_INIT_PRAGMAS: [&str; 2] = ["PRAGMA journal_mode = WAL;", "PRAGMA encoding = 'UTF-16';"];

// These pragmas need to be run once per connection.
const DB_PRAGMAS: [&str; 4] = [
    "PRAGMA synchronous = NORMAL;",
    "PRAGMA journal_size_limit = 67108864 -- 64 megabytes;",
    "PRAGMA mmap_size = 67108864 -- 64 megabytes;",
    "PRAGMA cache_size = 2000;",
];

fn range_to_query(range: IndexedDBKeyRange) -> Condition {
    // Special case for optimization
    if let Some(singleton) = range.as_singleton() {
        let encoded = bincode::serialize(singleton).unwrap();
        return Expr::column(object_data_model::Column::Data)
            .eq(encoded)
            .into_condition();
    }
    let mut parts = vec![];
    if let Some(upper) = range.upper.as_ref() {
        let upper_bytes = bincode::serialize(upper).unwrap();
        let query = if range.upper_open {
            Expr::column(object_data_model::Column::Data).lt(upper_bytes)
        } else {
            Expr::column(object_data_model::Column::Data).lte(upper_bytes)
        };
        parts.push(query);
    }
    if let Some(lower) = range.lower.as_ref() {
        let lower_bytes = bincode::serialize(lower).unwrap();
        let query = if range.upper_open {
            Expr::column(object_data_model::Column::Data).gt(lower_bytes)
        } else {
            Expr::column(object_data_model::Column::Data).gte(lower_bytes)
        };
        parts.push(query);
    }
    let mut condition = Condition::all();
    for part in parts {
        condition = condition.add(part);
    }
    condition
}

pub struct SqliteEngine {
    db_path: PathBuf,
    connection: Connection,
    read_pool: Arc<CoreResourceThreadPool>,
    write_pool: Arc<CoreResourceThreadPool>,
}

impl SqliteEngine {
    // TODO: intake dual pools
    pub fn new(
        base_dir: &Path,
        db_info: &IndexedDBDescription,
        version: u64,
        pool: Arc<CoreResourceThreadPool>,
    ) -> Result<Self, Error> {
        let mut db_path = PathBuf::new();
        db_path.push(base_dir);
        db_path.push(db_info.as_path());
        let db_parent = db_path.clone();
        db_path.push("db.sqlite");

        if !db_path.exists() {
            std::fs::create_dir_all(db_parent).unwrap();
            std::fs::File::create(&db_path).unwrap();
        }
        let connection = Self::init_db(&db_path, db_info, version)?;

        for stmt in DB_PRAGMAS {
            // TODO: Handle errors properly
            let _ = connection.execute(stmt, ());
        }

        Ok(Self {
            connection,
            db_path,
            read_pool: pool.clone(),
            write_pool: pool,
        })
    }

    fn init_db(
        path: &Path,
        db_info: &IndexedDBDescription,
        version: u64,
    ) -> Result<Connection, Error> {
        let connection = Connection::open(path)?;
        if connection.table_exists(None, "database")? {
            // Database already exists, no need to initialize
            return Ok(connection);
        }
        info!("Initializing indexeddb database at {:?}", path);
        for stmt in DB_INIT_PRAGMAS {
            // FIXME(arihant2math): this fails occasionally
            let _ = connection.execute(stmt, ());
        }
        create::create_tables(&connection)?;
        connection.execute(
            "INSERT INTO database (name, origin, version) VALUES (?, ?, ?)",
            params![
                db_info.name.to_owned(),
                db_info.origin.to_owned().ascii_serialization(),
                i64::from_ne_bytes(version.to_ne_bytes())
            ],
        )?;
        Ok(connection)
    }

    fn get(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
    ) -> Result<Option<object_data_model::Model>, Error> {
        let query = range_to_query(key_range);
        let stmt = sea_query::Query::select()
            .from(object_data_model::Column::Table)
            .and_where(query.and(Expr::col(object_data_model::Column::ObjectStoreId).is(store.id)))
            .to_owned();
        connection
            .prepare(&stmt.build(SqliteQueryBuilder).0)?
            .query_one((), |row| object_data_model::Model::try_from(row))
            .optional()
    }

    fn get_key(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
    ) -> Result<Option<Vec<u8>>, Error> {
        Self::get(connection, store, key_range).map(|opt| opt.map(|model| model.key))
    }

    fn get_item(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
    ) -> Result<Option<Vec<u8>>, Error> {
        Self::get(connection, store, key_range).map(|opt| opt.map(|model| model.data))
    }

    fn put_item(
        connection: &Connection,
        store: object_store_model::Model,
        serialized_key: Vec<u8>,
        value: Vec<u8>,
        should_overwrite: bool,
    ) -> Result<PutItemResult, Error> {
        let existing_item = connection
            .prepare("SELECT * FROM object_data WHERE key = ? AND object_store_id = ?")
            .and_then(|mut stmt| {
                stmt.query_row(params![serialized_key, store.id], |row| {
                    object_data_model::Model::try_from(row)
                })
                .optional()
            })?;
        if should_overwrite || existing_item.is_none() {
            connection.execute(
                "INSERT INTO object_data (object_store_id, key, data) VALUES (?, ?, ?)",
                params![store.id, serialized_key, value],
            )?;
            Ok(PutItemResult::Success)
        } else {
            Ok(PutItemResult::CannotOverwrite)
        }
    }

    fn delete_item(
        connection: &Connection,
        store: object_store_model::Model,
        serialized_key: Vec<u8>,
    ) -> Result<(), Error> {
        connection.execute(
            "DELETE FROM object_data WHERE key = ? AND object_store_id = ?",
            params![serialized_key, store.id],
        )?;
        Ok(())
    }

    fn clear(connection: &Connection, store: object_store_model::Model) -> Result<(), Error> {
        connection.execute(
            "DELETE FROM object_data WHERE object_store_id = ?",
            params![store.id],
        )?;
        Ok(())
    }

    fn count(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
    ) -> Result<usize, Error> {
        let query = range_to_query(key_range);
        let count = sea_query::Query::select()
            .expr(Expr::col(object_data_model::Column::Key).count())
            .from(object_data_model::Column::Table)
            .and_where(query.and(Expr::col(object_data_model::Column::ObjectStoreId).is(store.id)))
            .to_owned();
        connection
            .prepare(&count.build(SqliteQueryBuilder).0)?
            .query_row((), |row| row.get(0))
            .map(|count: i64| count as usize)
    }
}

impl KvsEngine for SqliteEngine {
    type Error = Error;

    fn create_store(
        &self,
        store_name: SanitizedName,
        key_path: Option<KeyPath>,
        auto_increment: bool,
    ) -> Result<CreateObjectResult, Self::Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT 1 FROM object_store WHERE name = ?")?;
        if stmt.exists(params![store_name.to_string()])? {
            // Store already exists
            return Ok(CreateObjectResult::AlreadyExists);
        }
        self.connection.execute(
            "INSERT INTO object_store (name, key_path, auto_increment) VALUES (?, ?, ?)",
            params![
                store_name.to_string(),
                key_path.map(|v| bincode::serialize(&v).unwrap()),
                auto_increment
            ],
        )?;

        Ok(CreateObjectResult::Created)
    }

    fn delete_store(&self, store_name: SanitizedName) -> Result<(), Self::Error> {
        let result = self.connection.execute(
            "DELETE FROM object_store WHERE name = ?",
            params![store_name.to_string()],
        )?;
        if result == 0 {
            Err(Error::QueryReturnedNoRows)
        } else if result > 1 {
            Err(Error::QueryReturnedMoreThanOneRow)
        } else {
            Ok(())
        }
    }

    fn close_store(&self, _store_name: SanitizedName) -> Result<(), Self::Error> {
        // TODO: do something
        Ok(())
    }

    fn delete_database(self) -> Result<(), Self::Error> {
        // attempt to close the connection first
        let _ = self.connection.close();
        if self.db_path.exists() {
            if let Err(e) = std::fs::remove_dir_all(self.db_path.parent().unwrap()) {
                error!("Failed to delete database: {:?}", e);
            }
        }
        Ok(())
    }

    fn process_transaction(
        &self,
        transaction: KvsTransaction,
    ) -> oneshot::Receiver<Option<Vec<u8>>> {
        let (tx, rx) = oneshot::channel();

        let spawning_pool = if transaction.mode == IndexedDBTxnMode::Readonly {
            self.read_pool.clone()
        } else {
            self.write_pool.clone()
        };
        let path = self.db_path.clone();
        spawning_pool.spawn(move || {
            let connection = Connection::open(path).unwrap();
            for request in transaction.requests {
                let object_store = connection
                    .prepare("SELECT 1 FROM object_store WHERE name = ?")
                    .and_then(|mut stmt| {
                        stmt.query_row(params![request.store_name.to_string()], |row| {
                            object_store_model::Model::try_from(row)
                        })
                        .optional()
                    });
                fn process_object_store<T>(
                    object_store: Result<Option<object_store_model::Model>, Error>,
                    sender: &IpcSender<BackendResult<T>>,
                ) -> Result<object_store_model::Model, ()>
                where
                    T: Serialize,
                {
                    match object_store {
                        Ok(Some(store)) => Ok(store),
                        Ok(None) => {
                            let _ = sender.send(Err(BackendError::StoreNotFound));
                            Err(())
                        },
                        Err(e) => {
                            let _ = sender.send(Err(BackendError::DbErr(format!("{:?}", e))));
                            Err(())
                        },
                    }
                }

                match request.operation {
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                        sender,
                        key,
                        value,
                        should_overwrite,
                    }) => {
                        let Ok(object_store) = process_object_store(object_store, &sender) else {
                            continue;
                        };
                        let serialized_key: Vec<u8> = bincode::serialize(&key).unwrap();
                        let _ = sender.send(
                            Self::put_item(
                                &connection,
                                object_store,
                                serialized_key,
                                value,
                                should_overwrite,
                            )
                            .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem {
                        sender,
                        key_range,
                    }) => {
                        let Ok(object_store) = process_object_store(object_store, &sender) else {
                            continue;
                        };
                        let _ = sender.send(
                            Self::get_item(&connection, object_store, key_range)
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem {
                        sender,
                        key,
                    }) => {
                        let Ok(object_store) = process_object_store(object_store, &sender) else {
                            continue;
                        };
                        let serialized_key: Vec<u8> = bincode::serialize(&key).unwrap();
                        let _ = sender.send(
                            Self::delete_item(&connection, object_store, serialized_key)
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count {
                        sender,
                        key_range,
                    }) => {
                        let Ok(object_store) = process_object_store(object_store, &sender) else {
                            continue;
                        };
                        let _ = sender.send(
                            Self::count(&connection, object_store, key_range)
                                .map(|r| r as u64)
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::Clear(sender)) => {
                        let Ok(object_store) = process_object_store(object_store, &sender) else {
                            continue;
                        };
                        let _ = sender.send(
                            Self::clear(&connection, object_store)
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetKey {
                        sender,
                        key_range,
                    }) => {
                        let Ok(object_store) = process_object_store(object_store, &sender) else {
                            continue;
                        };
                        let _ = sender.send(
                            Self::get_key(&connection, object_store, key_range)
                                .map(|key| key.map(|k| bincode::deserialize(&k).unwrap()))
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                }
            }
            let _ = tx.send(None);
        });
        rx
    }

    // TODO: we should be able to error out here, maybe change the trait definition?
    fn has_key_generator(&self, store_name: SanitizedName) -> bool {
        self.connection
            .prepare("SELECT * FROM object_store WHERE name = ?")
            .and_then(|mut stmt| {
                stmt.query_row(params![store_name.to_string()], |r| {
                    let object_store = object_store_model::Model::try_from(r).unwrap();
                    Ok(object_store.auto_increment)
                })
            })
            .optional()
            .unwrap()
            // TODO: Wrong (change trait definition for this function)
            .unwrap_or_default()
    }

    fn key_path(&self, store_name: SanitizedName) -> Option<KeyPath> {
        self.connection
            .prepare("SELECT * FROM object_store WHERE name = ?")
            .and_then(|mut stmt| {
                stmt.query_row(params![store_name.to_string()], |r| {
                    let object_store = object_store_model::Model::try_from(r).unwrap();
                    Ok(object_store
                        .key_path
                        .map(|key_path| bincode::deserialize(&key_path).unwrap()))
                })
            })
            .optional()
            .unwrap()
            // TODO: Wrong, same issues as has_key_generator
            .unwrap_or_default()
    }

    fn create_index(
        &self,
        store_name: SanitizedName,
        index_name: String,
        key_path: KeyPath,
        unique: bool,
        multi_entry: bool,
    ) -> Result<CreateObjectResult, Self::Error> {
        let object_store = self.connection.query_row(
            "SELECT * FROM object_store WHERE name = ?",
            params![store_name.to_string()],
            |row| object_store_model::Model::try_from(row),
        )?;

        let index_exists: bool = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM object_store_index WHERE name = ? AND object_store_id = ?)",
            params![index_name.to_string(), object_store.id],
            |row| row.get(0),
        )?;
        if index_exists {
            return Ok(CreateObjectResult::AlreadyExists);
        }

        self.connection.execute(
            "INSERT INTO object_store_index (object_store_id, name, key_path, unique_index, multi_entry_index)\
            VALUES (?, ?, ?, ?, ?)",
            params![
                object_store.id,
                index_name.to_string(),
                bincode::serialize(&key_path).unwrap(),
                unique,
                multi_entry,
            ],
        )?;
        Ok(CreateObjectResult::Created)
    }

    fn delete_index(
        &self,
        store_name: SanitizedName,
        index_name: String,
    ) -> Result<(), Self::Error> {
        let object_store = self.connection.query_row(
            "SELECT * FROM object_store WHERE name = ?",
            params![store_name.to_string()],
            |r| Ok(object_store_model::Model::try_from(r).unwrap()),
        )?;

        // Delete the index if it exists
        let _ = self.connection.execute(
            "DELETE FROM object_store_index WHERE name = ? AND object_store_id = ?",
            params![index_name.to_string(), object_store.id],
        )?;
        Ok(())
    }

    fn version(&self) -> Result<u64, Self::Error> {
        let version: i64 =
            self.connection
                .query_row("SELECT version FROM database LIMIT 1", [], |row| row.get(0))?;
        Ok(u64::from_ne_bytes(version.to_ne_bytes()))
    }

    fn set_version(&self, version: u64) -> Result<(), Self::Error> {
        let rows_affected = self.connection.execute(
            "UPDATE database SET version = ?",
            params![i64::from_ne_bytes(version.to_ne_bytes())],
        )?;
        if rows_affected == 0 {
            return Err(Error::QueryReturnedNoRows);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Arc;

    use net_traits::indexeddb_thread::{
        AsyncOperation, AsyncReadWriteOperation, CreateObjectResult, IndexedDBKeyType,
        IndexedDBTxnMode, KeyPath,
    };
    use servo_url::ImmutableOrigin;
    use url::Host;

    use crate::indexeddb::engines::{KvsEngine, KvsOperation, KvsTransaction, SanitizedName};
    use crate::indexeddb::idb_thread::IndexedDBDescription;
    use crate::resource_thread::CoreResourceThreadPool;

    fn test_origin() -> ImmutableOrigin {
        ImmutableOrigin::Tuple(
            "test_origin".to_string(),
            Host::Domain("localhost".to_string()),
            80,
        )
    }

    fn get_pool() -> Arc<CoreResourceThreadPool> {
        Arc::new(CoreResourceThreadPool::new(1, "test".to_string()))
    }

    #[test]
    fn test_cycle() {
        let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let thread_pool = get_pool();
        // Test create
        let _ = super::SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            1,
            thread_pool.clone(),
        )
        .unwrap();
        // Test open
        let db = super::SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            1,
            thread_pool.clone(),
        )
        .unwrap();
        let version = db.version().expect("Failed to get version");
        assert_eq!(version, 1);
        db.set_version(5).unwrap();
        let new_version = db.version().expect("Failed to get new version");
        assert_eq!(new_version, 5);
        db.delete_database().expect("Failed to delete database");
    }

    #[test]
    fn test_create_store() {
        let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let thread_pool = get_pool();
        let db = super::SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            1,
            thread_pool,
        )
        .unwrap();
        let store_name = SanitizedName::new("test_store".to_string());
        let result = db.create_store(store_name.clone(), None, true);
        assert!(result.is_ok());
        let create_result = result.unwrap();
        assert_eq!(create_result, CreateObjectResult::Created);
        // Try to create the same store again
        let result = db.create_store(store_name.clone(), None, false);
        assert!(result.is_ok());
        let create_result = result.unwrap();
        assert_eq!(create_result, CreateObjectResult::AlreadyExists);
        // Ensure store was not overwritten
        assert!(db.has_key_generator(store_name));
    }

    #[test]
    fn test_key_path() {
        let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let thread_pool = get_pool();
        let db = super::SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            1,
            thread_pool,
        )
        .unwrap();
        let store_name = SanitizedName::new("test_store".to_string());
        let result = db.create_store(
            store_name.clone(),
            Some(KeyPath::String("test".to_string())),
            true,
        );
        assert!(result.is_ok());
        assert_eq!(
            db.key_path(store_name),
            Some(KeyPath::String("test".to_string()))
        );
    }

    #[test]
    fn test_delete_store() {
        let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let thread_pool = get_pool();
        let db = super::SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            1,
            thread_pool,
        )
        .unwrap();
        db.create_store(SanitizedName::new("test_store".to_string()), None, false)
            .expect("Failed to create store");
        // Delete the store
        db.delete_store(SanitizedName::new("test_store".to_string()))
            .expect("Failed to delete store");
        // Try to delete the same store again
        let result = db.delete_store(SanitizedName::new("test_store".into()));
        assert!(result.is_err());
        // Try to delete a non-existing store
        let result = db.delete_store(SanitizedName::new("test_store".into()));
        // Should work as per spec
        assert!(result.is_err());
    }

    #[test]
    fn test_async_operations() {
        let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let thread_pool = get_pool();
        let db = super::SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            1,
            thread_pool,
        )
        .unwrap();
        let store_name = SanitizedName::new("test_store".to_string());
        db.create_store(store_name.clone(), None, false)
            .expect("Failed to create store");
        let rx = db.process_transaction(KvsTransaction {
            mode: IndexedDBTxnMode::Readwrite,
            requests: VecDeque::from(vec![
                // TODO: Test other operations
                KvsOperation {
                    store_name: store_name.clone(),
                    operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                        sender: ipc_channel::ipc::channel().unwrap().0,
                        key: IndexedDBKeyType::Number(1.0),
                        value: vec![],
                        should_overwrite: false,
                    }),
                },
            ]),
        });
        let _ = rx.blocking_recv().unwrap();
    }
}
