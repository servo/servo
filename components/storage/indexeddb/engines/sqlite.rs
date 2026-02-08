/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::path::{Path, PathBuf};
use std::sync::Arc;

use base::threadpool::ThreadPool;
use log::{error, info};
use rusqlite::{Connection, Error, OptionalExtension, params};
use sea_query::{Condition, Expr, ExprTrait, IntoCondition, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use storage_traits::indexeddb::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, BackendError,
    CreateObjectResult, IndexedDBKeyRange, IndexedDBKeyType, IndexedDBRecord, IndexedDBTxnMode,
    KeyPath, PutItemResult,
};
use tokio::sync::oneshot;

use crate::indexeddb::IndexedDBDescription;
use crate::indexeddb::engines::{KvsEngine, KvsTransaction};
use crate::shared::{DB_INIT_PRAGMAS, DB_PRAGMAS};

mod create;
mod database_model;
mod object_data_model;
mod object_store_index_model;
mod object_store_model;

fn range_to_query(range: IndexedDBKeyRange) -> Condition {
    // Special case for optimization
    if let Some(singleton) = range.as_singleton() {
        let encoded = postcard::to_stdvec(singleton).unwrap();
        return Expr::column(object_data_model::Column::Key)
            .eq(encoded)
            .into_condition();
    }
    let mut parts = vec![];
    if let Some(upper) = range.upper.as_ref() {
        let upper_bytes = postcard::to_stdvec(upper).unwrap();
        let query = if range.upper_open {
            Expr::column(object_data_model::Column::Key).lt(upper_bytes)
        } else {
            Expr::column(object_data_model::Column::Key).lte(upper_bytes)
        };
        parts.push(query);
    }
    if let Some(lower) = range.lower.as_ref() {
        let lower_bytes = postcard::to_stdvec(lower).unwrap();
        let query = if range.lower_open {
            Expr::column(object_data_model::Column::Key).gt(lower_bytes)
        } else {
            Expr::column(object_data_model::Column::Key).gte(lower_bytes)
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
    read_pool: Arc<ThreadPool>,
    write_pool: Arc<ThreadPool>,
    created_db_path: bool,
}

impl SqliteEngine {
    // TODO: intake dual pools
    pub fn new(
        base_dir: &Path,
        db_info: &IndexedDBDescription,
        pool: Arc<ThreadPool>,
    ) -> Result<Self, Error> {
        let mut db_path = PathBuf::new();
        db_path.push(base_dir);
        db_path.push(db_info.as_path());
        let db_parent = db_path.clone();
        db_path.push("db.sqlite");

        let created_db_path = if !db_path.exists() {
            std::fs::create_dir_all(db_parent).unwrap();
            std::fs::File::create(&db_path).unwrap();
            true
        } else {
            false
        };

        let connection = Self::init_db(&db_path, db_info)?;

        for stmt in DB_PRAGMAS {
            // TODO: Handle errors properly
            let _ = connection.execute(stmt, ());
        }

        Ok(Self {
            connection,
            db_path,
            read_pool: pool.clone(),
            write_pool: pool,
            created_db_path,
        })
    }

    /// Returns whether the physical db was created as part of `new`.
    pub(crate) fn created_db_path(&self) -> bool {
        self.created_db_path
    }

    fn init_db(path: &Path, db_info: &IndexedDBDescription) -> Result<Connection, Error> {
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
        // From https://w3c.github.io/IndexedDB/#database-version:
        // "When a database is first created, its version is 0 (zero)."
        connection.execute(
            "INSERT INTO database (name, origin, version) VALUES (?, ?, ?)",
            params![
                db_info.name.to_owned(),
                db_info.origin.to_owned().ascii_serialization(),
                i64::from_ne_bytes(0_u64.to_ne_bytes())
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
        let (sql, values) = sea_query::Query::select()
            .from(object_data_model::Column::Table)
            .columns(vec![
                object_data_model::Column::ObjectStoreId,
                object_data_model::Column::Key,
                object_data_model::Column::Data,
            ])
            .and_where(query.and(Expr::col(object_data_model::Column::ObjectStoreId).is(store.id)))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);
        connection
            .prepare(&sql)?
            .query_one(&*values.as_params(), |row| {
                object_data_model::Model::try_from(row)
            })
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

    fn get_all(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
        count: Option<u32>,
    ) -> Result<Vec<object_data_model::Model>, Error> {
        let query = range_to_query(key_range);
        let mut sql_query = sea_query::Query::select();
        sql_query
            .from(object_data_model::Column::Table)
            .columns(vec![
                object_data_model::Column::ObjectStoreId,
                object_data_model::Column::Key,
                object_data_model::Column::Data,
            ])
            .and_where(query.and(Expr::col(object_data_model::Column::ObjectStoreId).is(store.id)));
        if let Some(count) = count {
            sql_query.limit(count as u64);
        }
        let (sql, values) = sql_query.build_rusqlite(SqliteQueryBuilder);
        let mut stmt = connection.prepare(&sql)?;
        let models = stmt
            .query_and_then(&*values.as_params(), |row| {
                object_data_model::Model::try_from(row)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(models)
    }

    fn get_all_keys(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
        count: Option<u32>,
    ) -> Result<Vec<Vec<u8>>, Error> {
        Self::get_all(connection, store, key_range, count)
            .map(|models| models.into_iter().map(|m| m.key).collect())
    }

    fn get_all_items(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
        count: Option<u32>,
    ) -> Result<Vec<Vec<u8>>, Error> {
        Self::get_all(connection, store, key_range, count)
            .map(|models| models.into_iter().map(|m| m.data).collect())
    }

    #[expect(clippy::type_complexity)]
    fn get_all_records(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Error> {
        Self::get_all(connection, store, key_range, None)
            .map(|models| models.into_iter().map(|m| (m.key, m.data)).collect())
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
        key_range: IndexedDBKeyRange,
    ) -> Result<(), Error> {
        let query = range_to_query(key_range);
        let (sql, values) = sea_query::Query::delete()
            .from_table(object_data_model::Column::Table)
            .and_where(query.and(Expr::col(object_data_model::Column::ObjectStoreId).is(store.id)))
            .build_rusqlite(SqliteQueryBuilder);
        connection.prepare(&sql)?.execute(&*values.as_params())?;
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
        let (sql, values) = sea_query::Query::select()
            .expr(Expr::col(object_data_model::Column::Key).count())
            .from(object_data_model::Column::Table)
            .and_where(query.and(Expr::col(object_data_model::Column::ObjectStoreId).is(store.id)))
            .build_rusqlite(SqliteQueryBuilder);
        connection
            .prepare(&sql)?
            .query_row(&*values.as_params(), |row| row.get(0))
            .map(|count: i64| count as usize)
    }

    fn generate_key(
        connection: &Connection,
        store: &object_store_model::Model,
    ) -> Result<IndexedDBKeyType, Error> {
        if store.auto_increment == 0 {
            unreachable!("Should be caught in the script thread");
        }
        // TODO: handle overflows, this also needs to be able to handle 2^53 as per spec
        let new_key = store.auto_increment + 1;
        connection.execute(
            "UPDATE object_store SET auto_increment = ? WHERE id = ?",
            params![new_key, store.id],
        )?;
        Ok(IndexedDBKeyType::Number(new_key as f64))
    }
}

impl KvsEngine for SqliteEngine {
    type Error = Error;

    fn create_store(
        &self,
        store_name: &str,
        key_path: Option<KeyPath>,
        auto_increment: bool,
    ) -> Result<CreateObjectResult, Self::Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM object_store WHERE name = ?")?;
        if stmt.exists(params![store_name.to_string()])? {
            // Store already exists
            return Ok(CreateObjectResult::AlreadyExists);
        }
        self.connection.execute(
            "INSERT INTO object_store (name, key_path, auto_increment) VALUES (?, ?, ?)",
            params![
                store_name.to_string(),
                key_path.map(|v| postcard::to_stdvec(&v).unwrap()),
                auto_increment as i32
            ],
        )?;

        Ok(CreateObjectResult::Created)
    }

    fn delete_store(&self, store_name: &str) -> Result<(), Self::Error> {
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

    fn close_store(&self, _store_name: &str) -> Result<(), Self::Error> {
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
            let connection = match Connection::open(path) {
                Ok(connection) => connection,
                Err(error) => {
                    for request in transaction.requests {
                        request
                            .operation
                            .notify_error(BackendError::DbErr(format!("{e:?}")));
                    }
                    let _ = tx.send(None);
                    return;
                },
            };
            for request in transaction.requests {
                let object_store = connection
                    .prepare("SELECT * FROM object_store WHERE name = ?")
                    .and_then(|mut stmt| {
                        stmt.query_row(params![request.store_name.to_string()], |row| {
                            object_store_model::Model::try_from(row)
                        })
                        .optional()
                    });
                let object_store = match object_store {
                    Ok(Some(store)) => store,
                    Ok(None) => {
                        request.operation.notify_error(BackendError::StoreNotFound);
                        continue;
                    },
                    Err(error) => {
                        request
                            .operation
                            .notify_error(BackendError::DbErr(format!("{error:?}")));
                        continue;
                    },
                };

                match request.operation {
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                        callback,
                        key,
                        value,
                        should_overwrite,
                    }) => {
                        let key = match key
                            .map(Ok)
                            .unwrap_or_else(|| Self::generate_key(&connection, &object_store))
                        {
                            Ok(key) => key,
                            Err(e) => {
                                let _ = callback.send(Err(BackendError::DbErr(format!("{:?}", e))));
                                continue;
                            },
                        };
                        let serialized_key: Vec<u8> = postcard::to_stdvec(&key).unwrap();
                        let _ = callback.send(
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
                        callback,
                        key_range,
                    }) => {
                        let _ = callback.send(
                            Self::get_item(&connection, object_store, key_range)
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetAllKeys {
                        callback,
                        key_range,
                        count,
                    }) => {
                        let _ = callback.send(
                            Self::get_all_keys(&connection, object_store, key_range, count)
                                .map(|keys| {
                                    keys.into_iter()
                                        .map(|k| postcard::from_bytes(&k).unwrap())
                                        .collect()
                                })
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetAllItems {
                        callback,
                        key_range,
                        count,
                    }) => {
                        let _ = callback.send(
                            Self::get_all_items(&connection, object_store, key_range, count)
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem {
                        callback,
                        key_range,
                    }) => {
                        let _ = callback.send(
                            Self::delete_item(&connection, object_store, key_range)
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count {
                        callback,
                        key_range,
                    }) => {
                        let _ = callback.send(
                            Self::count(&connection, object_store, key_range)
                                .map(|r| r as u64)
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Iterate {
                        callback,
                        key_range,
                    }) => {
                        let _ = callback.send(
                            Self::get_all_records(&connection, object_store, key_range)
                                .map(|records| {
                                    records
                                        .into_iter()
                                        .map(|(key, data)| IndexedDBRecord {
                                            key: postcard::from_bytes(&key).unwrap(),
                                            primary_key: postcard::from_bytes(&key).unwrap(),
                                            value: data,
                                        })
                                        .collect()
                                })
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::Clear(sender)) => {
                        let _ = sender.send(
                            Self::clear(&connection, object_store)
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetKey {
                        callback,
                        key_range,
                    }) => {
                        let _ = callback.send(
                            Self::get_key(&connection, object_store, key_range)
                                .map(|key| key.map(|k| postcard::from_bytes(&k).unwrap()))
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
    fn has_key_generator(&self, store_name: &str) -> bool {
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
            .unwrap_or_default() !=
            0
    }

    fn key_path(&self, store_name: &str) -> Option<KeyPath> {
        self.connection
            .prepare("SELECT * FROM object_store WHERE name = ?")
            .and_then(|mut stmt| {
                stmt.query_row(params![store_name.to_string()], |r| {
                    let object_store = object_store_model::Model::try_from(r).unwrap();
                    Ok(object_store
                        .key_path
                        .map(|key_path| postcard::from_bytes(&key_path).unwrap()))
                })
            })
            .optional()
            .unwrap()
            // TODO: Wrong, same issues as has_key_generator
            .unwrap_or_default()
    }

    fn create_index(
        &self,
        store_name: &str,
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
            "SELECT EXISTS(SELECT * FROM object_store_index WHERE name = ? AND object_store_id = ?)",
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
                postcard::to_stdvec(&key_path).unwrap(),
                unique,
                multi_entry,
            ],
        )?;
        Ok(CreateObjectResult::Created)
    }

    fn delete_index(&self, store_name: &str, index_name: String) -> Result<(), Self::Error> {
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

    use base::generic_channel::{self, GenericReceiver, GenericSender};
    use base::threadpool::ThreadPool;
    use profile_traits::generic_callback::GenericCallback;
    use profile_traits::time::ProfilerChan;
    use serde::{Deserialize, Serialize};
    use servo_url::ImmutableOrigin;
    use storage_traits::indexeddb::{
        AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, CreateObjectResult,
        IndexedDBKeyRange, IndexedDBKeyType, IndexedDBTxnMode, KeyPath, PutItemResult,
    };
    use url::Host;

    use crate::indexeddb::IndexedDBDescription;
    use crate::indexeddb::engines::{KvsEngine, KvsOperation, KvsTransaction, SqliteEngine};

    fn test_origin() -> ImmutableOrigin {
        ImmutableOrigin::Tuple(
            "test_origin".to_string(),
            Host::Domain("localhost".to_string()),
            80,
        )
    }

    fn get_pool() -> Arc<ThreadPool> {
        Arc::new(ThreadPool::new(1, "test".to_string()))
    }

    #[test]
    fn test_cycle() {
        let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let thread_pool = get_pool();
        // Test create
        let _ = SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            thread_pool.clone(),
        )
        .unwrap();
        // Test open
        let db = SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            thread_pool.clone(),
        )
        .unwrap();
        let version = db.version().expect("Failed to get version");
        assert_eq!(version, 0);
        db.set_version(5).unwrap();
        let new_version = db.version().expect("Failed to get new version");
        assert_eq!(new_version, 5);
        db.delete_database().expect("Failed to delete database");
    }

    #[test]
    fn test_create_store() {
        let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let thread_pool = get_pool();
        let db = SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            thread_pool,
        )
        .unwrap();
        let store_name = "test_store";
        let result = db.create_store(store_name, None, true);
        assert!(result.is_ok());
        let create_result = result.unwrap();
        assert_eq!(create_result, CreateObjectResult::Created);
        // Try to create the same store again
        let result = db.create_store(store_name, None, false);
        assert!(result.is_ok());
        let create_result = result.unwrap();
        assert_eq!(create_result, CreateObjectResult::AlreadyExists);
        // Ensure store was not overwritten
        assert!(db.has_key_generator(store_name));
    }

    #[test]
    fn test_create_store_empty_name() {
        let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let thread_pool = get_pool();
        let db = SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            thread_pool,
        )
        .unwrap();
        let store_name = "";
        let result = db.create_store(store_name, None, true);
        assert!(result.is_ok());
        let create_result = result.unwrap();
        assert_eq!(create_result, CreateObjectResult::Created);
    }

    #[test]
    fn test_injection() {
        let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let thread_pool = get_pool();
        let db = SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            thread_pool,
        )
        .unwrap();
        // Create a normal store
        let store_name1 = "test_store";
        let result = db.create_store(store_name1, None, true);
        assert!(result.is_ok());
        let create_result = result.unwrap();
        assert_eq!(create_result, CreateObjectResult::Created);
        // Injection
        let store_name2 = "' OR 1=1 -- -";
        let result = db.create_store(store_name2, None, false);
        assert!(result.is_ok());
        let create_result = result.unwrap();
        assert_eq!(create_result, CreateObjectResult::Created);
    }

    #[test]
    fn test_key_path() {
        let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let thread_pool = get_pool();
        let db = SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            thread_pool,
        )
        .unwrap();
        let store_name = "test_store";
        let result = db.create_store(store_name, Some(KeyPath::String("test".to_string())), true);
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
        let db = SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            thread_pool,
        )
        .unwrap();
        db.create_store("test_store", None, false)
            .expect("Failed to create store");
        // Delete the store
        db.delete_store("test_store")
            .expect("Failed to delete store");
        // Try to delete the same store again
        let result = db.delete_store("test_store");
        assert!(result.is_err());
        // Try to delete a non-existing store
        let result = db.delete_store("test_store");
        // Should work as per spec
        assert!(result.is_err());
    }

    #[test]
    fn test_async_operations() {
        fn get_channel<T>() -> (GenericSender<T>, GenericReceiver<T>)
        where
            T: for<'de> Deserialize<'de> + Serialize,
        {
            generic_channel::channel().unwrap()
        }

        fn get_callback<T>(chan: GenericSender<T>) -> GenericCallback<T>
        where
            T: for<'de> Deserialize<'de> + Serialize + Send + Sync,
        {
            GenericCallback::new(ProfilerChan(None), move |r| {
                assert!(chan.send(r.unwrap()).is_ok());
            })
            .expect("Could not construct callback")
        }

        let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let thread_pool = get_pool();
        let db = SqliteEngine::new(
            base_dir.path(),
            &IndexedDBDescription {
                name: "test_db".to_string(),
                origin: test_origin(),
            },
            thread_pool,
        )
        .unwrap();
        let store_name = "test_store";
        db.create_store(store_name, None, false)
            .expect("Failed to create store");
        let put = get_channel();
        let put2 = get_channel();
        let put3 = get_channel();
        let put_dup = get_channel();
        let get_item_some = get_channel();
        let get_item_none = get_channel();
        let get_all_items = get_channel();
        let count = get_channel();
        let remove = get_channel();
        let clear = get_channel();
        let rx = db.process_transaction(KvsTransaction {
            mode: IndexedDBTxnMode::Readwrite,
            requests: VecDeque::from(vec![
                KvsOperation {
                    store_name: store_name.to_owned(),
                    operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                        callback: get_callback(put.0),
                        key: Some(IndexedDBKeyType::Number(1.0)),
                        value: vec![1, 2, 3],
                        should_overwrite: false,
                    }),
                },
                KvsOperation {
                    store_name: store_name.to_owned(),
                    operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                        callback: get_callback(put2.0),
                        key: Some(IndexedDBKeyType::String("2.0".to_string())),
                        value: vec![4, 5, 6],
                        should_overwrite: false,
                    }),
                },
                KvsOperation {
                    store_name: store_name.to_owned(),
                    operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                        callback: get_callback(put3.0),
                        key: Some(IndexedDBKeyType::Array(vec![
                            IndexedDBKeyType::String("3".to_string()),
                            IndexedDBKeyType::Number(0.0),
                        ])),
                        value: vec![7, 8, 9],
                        should_overwrite: false,
                    }),
                },
                // Try to put a duplicate key without overwrite
                KvsOperation {
                    store_name: store_name.to_owned(),
                    operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                        callback: get_callback(put_dup.0),
                        key: Some(IndexedDBKeyType::Number(1.0)),
                        value: vec![10, 11, 12],
                        should_overwrite: false,
                    }),
                },
                KvsOperation {
                    store_name: store_name.to_owned(),
                    operation: AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem {
                        callback: get_callback(get_item_some.0),
                        key_range: IndexedDBKeyRange::only(IndexedDBKeyType::Number(1.0)),
                    }),
                },
                KvsOperation {
                    store_name: store_name.to_owned(),
                    operation: AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem {
                        callback: get_callback(get_item_none.0),
                        key_range: IndexedDBKeyRange::only(IndexedDBKeyType::Number(5.0)),
                    }),
                },
                KvsOperation {
                    store_name: store_name.to_owned(),
                    operation: AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetAllItems {
                        callback: get_callback(get_all_items.0),
                        key_range: IndexedDBKeyRange::lower_bound(
                            IndexedDBKeyType::Number(0.0),
                            false,
                        ),
                        count: None,
                    }),
                },
                KvsOperation {
                    store_name: store_name.to_owned(),
                    operation: AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count {
                        callback: get_callback(count.0),
                        key_range: IndexedDBKeyRange::only(IndexedDBKeyType::Number(1.0)),
                    }),
                },
                KvsOperation {
                    store_name: store_name.to_owned(),
                    operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem {
                        callback: get_callback(remove.0),
                        key_range: IndexedDBKeyRange::only(IndexedDBKeyType::Number(1.0)),
                    }),
                },
                KvsOperation {
                    store_name: store_name.to_owned(),
                    operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::Clear(
                        get_callback(clear.0),
                    )),
                },
            ]),
        });
        let _ = rx.blocking_recv().unwrap();
        put.1.recv().unwrap().unwrap();
        put2.1.recv().unwrap().unwrap();
        put3.1.recv().unwrap().unwrap();
        let err = put_dup.1.recv().unwrap().unwrap();
        assert_eq!(err, PutItemResult::CannotOverwrite);
        let get_result = get_item_some.1.recv().unwrap();
        let value = get_result.unwrap();
        assert_eq!(value, Some(vec![1, 2, 3]));
        let get_result = get_item_none.1.recv().unwrap();
        let value = get_result.unwrap();
        assert_eq!(value, None);
        let all_items = get_all_items.1.recv().unwrap().unwrap();
        assert_eq!(all_items.len(), 3);
        // Check that all three items are present
        assert!(all_items.contains(&vec![1, 2, 3]));
        assert!(all_items.contains(&vec![4, 5, 6]));
        assert!(all_items.contains(&vec![7, 8, 9]));
        let amount = count.1.recv().unwrap().unwrap();
        assert_eq!(amount, 1);
        remove.1.recv().unwrap().unwrap();
        clear.1.recv().unwrap().unwrap();
    }
}
