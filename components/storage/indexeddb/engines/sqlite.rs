/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::path::{Path, PathBuf};
use std::sync::Arc;

use log::{error, info, warn};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use rusqlite::{Connection, Error, OptionalExtension, params};
use sea_query::{
    Condition, Expr, ExprTrait, IntoColumnRef, IntoCondition, Order, SqliteQueryBuilder,
};
use sea_query_rusqlite::RusqliteBinder;
use servo_base::threadpool::ThreadPool;
use storage_traits::indexeddb::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, BackendError,
    CreateObjectResult, IndexedDBIndex, IndexedDBKeyRange, IndexedDBKeyType, IndexedDBRecord,
    IndexedDBTxnMode, KeyPath, PutItemResult,
};

use crate::indexeddb::IndexedDBDescription;
use crate::indexeddb::engines::{KvsEngine, KvsTransaction};
use crate::shared::{DB_INIT_PRAGMAS, DB_PRAGMAS};

mod create;
mod database_model;
mod encoding;
pub mod index_data_model;
mod object_data_model;
mod object_store_index_model;
mod object_store_model;

fn range_to_query<T: Copy + IntoColumnRef>(col: T, range: IndexedDBKeyRange) -> Condition {
    // Special case for optimization
    if let Some(singleton) = range.as_singleton() {
        let encoded = encoding::serialize(singleton);
        return Expr::column(col).eq(encoded).into_condition();
    }
    let mut parts = vec![];
    if let Some(upper) = range.upper.as_ref() {
        let upper_bytes = encoding::serialize(upper);
        let query = if range.upper_open {
            Expr::column(col).lt(upper_bytes)
        } else {
            Expr::column(col).lte(upper_bytes)
        };
        parts.push(query);
    }
    if let Some(lower) = range.lower.as_ref() {
        let lower_bytes = encoding::serialize(lower);
        let query = if range.lower_open {
            Expr::column(col).gt(lower_bytes)
        } else {
            Expr::column(col).gte(lower_bytes)
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
    fn object_store_by_name(
        connection: &Connection,
        store_name: &str,
    ) -> Result<object_store_model::Model, Error> {
        connection.query_row(
            "SELECT * FROM object_store WHERE name = ?",
            params![store_name.to_string()],
            |row| object_store_model::Model::try_from(row),
        )
    }

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

    /// Gets the associated index model for a given object store and index name
    /// Returns `Ok(None)` if no such index exists.
    fn get_index_model(
        connection: &Connection,
        store: object_store_model::Model,
        index: String,
    ) -> Result<Option<object_store_index_model::Model>, Error> {
        connection
            .query_one(
                "SELECT * FROM object_store_index WHERE object_store_id = ? AND name = ?",
                params![store.id, index],
                |row| object_store_index_model::Model::try_from(row),
            )
            .optional()
    }

    fn get(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
    ) -> Result<Option<object_data_model::Model>, Error> {
        let query = range_to_query(object_data_model::Column::Key, key_range);
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

    fn index_get(
        connection: &Connection,
        store: object_store_model::Model,
        index: String,
        key_range: IndexedDBKeyRange,
    ) -> Result<Option<index_data_model::Model>, Error> {
        let Some(index_model) = Self::get_index_model(connection, store, index)? else {
            return Ok(None);
        };
        let query = range_to_query(index_data_model::Column::IndexKey, key_range);
        let (sql, values) = sea_query::Query::select()
            .from(index_data_model::Column::Table)
            .columns(vec![
                index_data_model::Column::IndexId,
                index_data_model::Column::IndexKey,
                index_data_model::Column::ObjectKey,
            ])
            .and_where(query.and(Expr::col(index_data_model::Column::IndexId).is(index_model.id)))
            .order_by(index_data_model::Column::IndexKey, Order::Asc)
            .order_by(index_data_model::Column::ObjectKey, Order::Asc)
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);
        connection
            .prepare(&sql)?
            .query_one(&*values.as_params(), |row| {
                index_data_model::Model::try_from(row)
            })
            .optional()
    }

    fn get_by_serialized_key(
        connection: &Connection,
        store: object_store_model::Model,
        key: &[u8],
    ) -> Result<Option<object_data_model::Model>, Error> {
        connection
            .prepare("SELECT * FROM object_data WHERE object_store_id = ? AND key = ?")?
            .query_row(params![store.id, key], |row| {
                object_data_model::Model::try_from(row)
            })
            .optional()
    }

    fn delete_index_entries_for_object_key(
        connection: &Connection,
        store: object_store_model::Model,
        object_key: &[u8],
    ) -> Result<(), Error> {
        connection.execute(
            "DELETE FROM index_data WHERE object_key = ? AND index_id IN \
             (SELECT id FROM object_store_index WHERE object_store_id = ?)",
            params![object_key, store.id],
        )?;
        Ok(())
    }

    fn get_key(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
    ) -> Result<Option<Vec<u8>>, Error> {
        Self::get(connection, store, key_range).map(|opt| opt.map(|model| model.key))
    }

    fn index_get_key(
        connection: &Connection,
        store: object_store_model::Model,
        index_name: String,
        key_range: IndexedDBKeyRange,
    ) -> Result<Option<Vec<u8>>, Error> {
        Self::index_get(connection, store, index_name, key_range)
            .map(|opt| opt.map(|model| model.object_key))
    }

    fn get_item(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
    ) -> Result<Option<Vec<u8>>, Error> {
        Self::get(connection, store, key_range).map(|opt| opt.map(|model| model.data))
    }

    fn index_get_item(
        connection: &Connection,
        store: object_store_model::Model,
        index_name: String,
        key_range: IndexedDBKeyRange,
    ) -> Result<Option<Vec<u8>>, Error> {
        let Some(index_data) = Self::index_get(connection, store.clone(), index_name, key_range)?
        else {
            return Ok(None);
        };
        Self::get_by_serialized_key(connection, store, &index_data.object_key)
            .map(|opt| opt.map(|model| model.data))
    }

    fn get_all(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
        count: Option<u32>,
    ) -> Result<Vec<object_data_model::Model>, Error> {
        let query = range_to_query(object_data_model::Column::Key, key_range);
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

    fn index_get_all(
        connection: &Connection,
        store: object_store_model::Model,
        index_name: String,
        key_range: IndexedDBKeyRange,
        count: Option<u32>,
    ) -> Result<Vec<index_data_model::Model>, Error> {
        let Some(index_model) = Self::get_index_model(connection, store, index_name)? else {
            return Ok(vec![]);
        };
        let query = range_to_query(index_data_model::Column::IndexKey, key_range);
        let mut sql_query = sea_query::Query::select();
        sql_query
            .from(index_data_model::Column::Table)
            .columns(vec![
                index_data_model::Column::IndexId,
                index_data_model::Column::IndexKey,
                index_data_model::Column::ObjectKey,
            ])
            .and_where(query.and(Expr::col(index_data_model::Column::IndexId).is(index_model.id)))
            .order_by(index_data_model::Column::IndexKey, Order::Asc)
            .order_by(index_data_model::Column::ObjectKey, Order::Asc);
        if let Some(count) = count {
            sql_query.limit(count as u64);
        }
        let (sql, values) = sql_query.build_rusqlite(SqliteQueryBuilder);
        let mut stmt = connection.prepare(&sql)?;
        let models = stmt
            .query_and_then(&*values.as_params(), |row| {
                index_data_model::Model::try_from(row)
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

    fn index_get_all_keys(
        connection: &Connection,
        store: object_store_model::Model,
        index: String,
        key_range: IndexedDBKeyRange,
        count: Option<u32>,
    ) -> Result<Vec<Vec<u8>>, Error> {
        Self::index_get_all(connection, store, index, key_range, count)
            .map(|models| models.into_iter().map(|m| m.object_key).collect())
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

    fn index_get_all_items(
        connection: &Connection,
        store: object_store_model::Model,
        index: String,
        key_range: IndexedDBKeyRange,
        count: Option<u32>,
    ) -> Result<Vec<Vec<u8>>, Error> {
        let index_models = Self::index_get_all(connection, store.clone(), index, key_range, count)?;
        let mut items = Vec::with_capacity(index_models.len());
        for model in index_models {
            if let Some(item) =
                Self::get_by_serialized_key(connection, store.clone(), &model.object_key)?
            {
                items.push(item.data);
            }
        }
        Ok(items)
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
        key: IndexedDBKeyType,
        value: Vec<u8>,
        should_overwrite: bool,
        key_generator_current_number: Option<i32>,
        index_key_value: Vec<(String, bool, IndexedDBKeyType)>,
    ) -> Result<PutItemResult, Error> {
        let no_overwrite = !should_overwrite;
        let serialized_key: Vec<u8> = encoding::serialize(&key);
        let existing_item = connection
            .prepare("SELECT * FROM object_data WHERE key = ? AND object_store_id = ?")
            .and_then(|mut stmt| {
                stmt.query_row(params![serialized_key, store.id], |row| {
                    object_data_model::Model::try_from(row)
                })
                .optional()
            })?;
        if existing_item.is_some() && no_overwrite {
            return Ok(PutItemResult::CannotOverwrite);
        }

        let mut index_entries = Vec::with_capacity(index_key_value.len());
        for (index_name, _unique, index_key) in index_key_value {
            let index_model = Self::get_index_model(connection, store.clone(), index_name)?
                .ok_or(Error::QueryReturnedNoRows)?;
            let serialized_index_key = encoding::serialize(&index_key);
            if index_model.unique_index {
                let has_conflict: bool = connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM index_data WHERE index_id = ? AND index_key = ? AND object_key != ?)",
                    params![index_model.id, &serialized_index_key, &serialized_key],
                    |row| row.get(0),
                )?;
                if has_conflict {
                    return Err(Error::SqliteFailure(
                        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE),
                        Some("Unique index constraint violated".to_string()),
                    ));
                }
            }
            index_entries.push((index_model.id, serialized_index_key));
        }

        if existing_item.is_some() {
            // Preserve `put()` semantics by replacing the stored value when the primary
            // key already exists.
            connection.execute(
                "UPDATE object_data SET data = ? WHERE object_store_id = ? AND key = ?",
                params![value, store.id, serialized_key],
            )?;
            Self::delete_index_entries_for_object_key(connection, store.clone(), &serialized_key)?;
        } else {
            connection.execute(
                "INSERT INTO object_data (object_store_id, key, data) VALUES (?, ?, ?)",
                params![store.id, serialized_key, value],
            )?;
        }

        for (index_id, serialized_index_key) in index_entries {
            connection.execute(
                "INSERT INTO index_data (index_id, index_key, object_key) VALUES (?, ?, ?)",
                params![index_id, serialized_index_key, &serialized_key],
            )?;
        }
        if let Some(next_key_generator_current_number) = key_generator_current_number {
            connection.execute(
                "UPDATE object_store SET auto_increment = ? WHERE id = ?",
                params![next_key_generator_current_number, store.id],
            )?;
        }
        Ok(PutItemResult::Key(key))
    }

    fn delete_item(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
    ) -> Result<(), Error> {
        let keys = Self::get_all_keys(connection, store.clone(), key_range.clone(), None)?;
        for key in &keys {
            Self::delete_index_entries_for_object_key(connection, store.clone(), key)?;
        }
        let query = range_to_query(object_data_model::Column::Key, key_range);
        let (sql, values) = sea_query::Query::delete()
            .from_table(object_data_model::Column::Table)
            .and_where(query.and(Expr::col(object_data_model::Column::ObjectStoreId).is(store.id)))
            .build_rusqlite(SqliteQueryBuilder);
        connection.prepare(&sql)?.execute(&*values.as_params())?;
        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB/#clear-an-object-store>
    fn clear(connection: &Connection, store: object_store_model::Model) -> Result<(), Error> {
        // Step 1. Remove all records from store.
        connection.execute(
            "DELETE FROM object_data WHERE object_store_id = ?",
            params![store.id],
        )?;
        // Step 2. In all indexes which reference store, remove all records.
        connection.execute(
            "DELETE FROM index_data WHERE index_id IN \
             (SELECT id FROM object_store_index WHERE object_store_id = ?)",
            params![store.id],
        )?;
        // Step 3. Return undefined.
        Ok(())
    }

    fn count(
        connection: &Connection,
        store: object_store_model::Model,
        key_range: IndexedDBKeyRange,
    ) -> Result<usize, Error> {
        let query = range_to_query(object_data_model::Column::Key, key_range);
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

    fn index_count(
        connection: &Connection,
        store: object_store_model::Model,
        index_name: String,
        key_range: IndexedDBKeyRange,
    ) -> Result<usize, Error> {
        let index = Self::get_index_model(connection, store, index_name)?
            .ok_or_else(|| Error::QueryReturnedNoRows)?;
        let query = range_to_query(index_data_model::Column::IndexKey, key_range);
        let (sql, values) = sea_query::Query::select()
            .expr(Expr::col(index_data_model::Column::IndexKey).count())
            .from(index_data_model::Column::Table)
            .and_where(query.and(Expr::col(index_data_model::Column::IndexId).is(index.id)))
            .build_rusqlite(SqliteQueryBuilder);
        connection
            .prepare(&sql)?
            .query_row(&*values.as_params(), |row| row.get(0))
            .map(|count: i64| count as usize)
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
        // https://www.w3.org/TR/IndexedDB-3/#dom-idbdatabase-deleteobjectstore
        // Step 7. Destroy store.
        let object_store = Self::object_store_by_name(&self.connection, store_name)?;

        self.connection.execute(
            "DELETE FROM index_data WHERE index_id IN \
             (SELECT id FROM object_store_index WHERE object_store_id = ?)",
            params![object_store.id],
        )?;
        self.connection.execute(
            "DELETE FROM object_store_index WHERE object_store_id = ?",
            params![object_store.id],
        )?;
        self.connection.execute(
            "DELETE FROM object_data WHERE object_store_id = ?",
            params![object_store.id],
        )?;
        let result = self.connection.execute(
            "DELETE FROM object_store WHERE id = ?",
            params![object_store.id],
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
        on_complete: Box<dyn FnOnce() + Send + 'static>,
    ) {
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
                            .notify_error(BackendError::DbErr(format!("{error:?}")));
                    }
                    on_complete();
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
                        key_generator_current_number,
                        index_key_value
                    }) => {
                        let (key, key_generator_current_number) = match key {
                            Some(key) => (key, key_generator_current_number),
                            None => {
                                if object_store.auto_increment == 0 {
                                    if let Err(error) = callback.send(Err(BackendError::DbErr(
                                        "Missing key for PutItem request".to_string(),
                                    ))) {
                                        warn!("Failed to send PutItem missing key error: {error:?}");
                                    }
                                    continue;
                                }
                                let Some(next_key_generator_current_number) =
                                    object_store.auto_increment.checked_add(1)
                                else {
                                    if let Err(error) = callback.send(Err(BackendError::DbErr(
                                        "Key generator overflow".to_string(),
                                    ))) {
                                        warn!(
                                            "Failed to send PutItem key generator overflow error: {error:?}"
                                        );
                                    }
                                    continue;
                                };
                                (
                                    IndexedDBKeyType::Number(object_store.auto_increment as f64),
                                    Some(next_key_generator_current_number),
                                )
                            },
                        };
                        let _ = callback.send(
                            Self::put_item(
                                &connection,
                                object_store,
                                key,
                                value,
                                should_overwrite,
                                key_generator_current_number,
                                index_key_value
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
                                        .map(|k| encoding::deserialize(&k).unwrap())
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
                                            key: encoding::deserialize(&key).unwrap(),
                                            primary_key: encoding::deserialize(&key).unwrap(),
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
                                .map(|key| key.map(|k| encoding::deserialize(&k).unwrap()))
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::IndexGetKey {
                                                 callback,
                                                 index_name,
                                                 key_range,
                                             }) => {
                        let _ = callback.send(
                            Self::index_get_key(&connection, object_store, index_name, key_range)
                                .map(|key| key.map(|k| encoding::deserialize(&k).unwrap()))
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::IndexGetItem {
                                                 callback,
                                                 index_name,
                                                 key_range,
                                             }) => {
                        let _ = callback.send(
                            Self::index_get_item(&connection, object_store, index_name, key_range)
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::IndexGetAllKeys {
                                                 callback,
                                                 index_name,
                                                 key_range,
                                                 count,
                                             }) => {
                        let _ = callback.send(
                            Self::index_get_all_keys(
                                &connection,
                                object_store,
                                index_name,
                                key_range,
                                count,
                            )
                                .map(|keys| {
                                    keys.into_iter()
                                        .map(|k| encoding::deserialize(&k).unwrap())
                                        .collect()
                                })
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::IndexGetAllItems {
                                                 callback,
                                                 index_name,
                                                 key_range,
                                                 count,
                                             }) => {
                        let _ = callback.send(
                            Self::index_get_all_items(
                                &connection,
                                object_store,
                                index_name,
                                key_range,
                                count,
                            )
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::IndexCount {
                                                 callback,
                                                 index_name,
                                                 key_range,
                                             }) => {
                        let _ = callback.send(
                            Self::index_count(&connection, object_store, index_name, key_range)
                                .map(|r| r as u64)
                                .map_err(|e| BackendError::DbErr(format!("{:?}", e))),
                        );
                    },
                }
            }
            on_complete();
        });
    }

    fn key_generator_current_number(&self, store_name: &str) -> Option<i32> {
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
            .and_then(|current_number| (current_number != 0).then_some(current_number))
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

    fn object_store_names(&self) -> Result<Vec<String>, Self::Error> {
        let mut stmt = self.connection.prepare("SELECT name FROM object_store")?;
        stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()
    }

    fn indexes(&self, store_name: &str) -> Result<Vec<IndexedDBIndex>, Self::Error> {
        let object_store = self.connection.query_row(
            "SELECT * FROM object_store WHERE name = ?",
            params![store_name.to_string()],
            |row| object_store_model::Model::try_from(row),
        )?;

        let mut stmt = self
            .connection
            .prepare("SELECT * FROM object_store_index WHERE object_store_id = ?")?;
        let indexes = stmt
            .query_map(params![object_store.id], |row| {
                let model = object_store_index_model::Model::try_from(row)?;
                Ok(IndexedDBIndex {
                    name: model.name,
                    key_path: postcard::from_bytes(&model.key_path).unwrap(),
                    unique: model.unique_index,
                    multi_entry: model.multi_entry_index,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(indexes)
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
            params![index_name, object_store.id],
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
                index_name,
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

        let index_id: Option<i32> = self
            .connection
            .query_row(
                "SELECT id FROM object_store_index WHERE name = ? AND object_store_id = ?",
                params![index_name, object_store.id],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(index_id) = index_id {
            self.connection.execute(
                "DELETE FROM index_data WHERE index_id = ?",
                params![index_id],
            )?;
            self.connection.execute(
                "DELETE FROM object_store_index WHERE id = ?",
                params![index_id],
            )?;
        }
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

fn get_db_status(connection: &Connection, op: i32) -> Result<i32, i32> {
    let mut p_curr = 0;
    let mut p_hiwater = 0;
    let res = unsafe {
        rusqlite::ffi::sqlite3_db_status(connection.handle(), op, &mut p_curr, &mut p_hiwater, 0)
    };
    if res != 0 { Err(res) } else { Ok(p_curr) }
}

impl MallocSizeOf for SqliteEngine {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        // 48 KB (3.3.1 at https://sqlite.org/malloc.html)
        const DEFAULT_LOOKASIDE_SIZE: usize = 48 * 1024;
        self.created_db_path.size_of(ops) +
            DEFAULT_LOOKASIDE_SIZE +
            get_db_status(
                &self.connection,
                rusqlite::ffi::SQLITE_DBSTATUS_CACHE_USED_SHARED,
            )
            .unwrap_or_default() as usize +
            get_db_status(&self.connection, rusqlite::ffi::SQLITE_DBSTATUS_SCHEMA_USED)
                .unwrap_or_default() as usize +
            get_db_status(&self.connection, rusqlite::ffi::SQLITE_DBSTATUS_STMT_USED)
                .unwrap_or_default() as usize
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Arc;

    use profile_traits::generic_callback::GenericCallback;
    use profile_traits::time::ProfilerChan;
    use serde::{Deserialize, Serialize};
    use servo_base::generic_channel::{self, GenericReceiver, GenericSender};
    use servo_base::threadpool::ThreadPool;
    use servo_url::ImmutableOrigin;
    use storage_traits::indexeddb::{
        AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, CreateObjectResult,
        IndexedDBKeyRange, IndexedDBKeyType, IndexedDBTxnMode, KeyPath, PutItemResult,
    };
    use url::Host;

    use crate::indexeddb::IndexedDBDescription;
    use crate::indexeddb::engines::sqlite::encoding;
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
        assert!(db.key_generator_current_number(store_name).is_some());
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
    fn test_delete_store_removes_store_records() {
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
        db.create_index(
            "test_store",
            "by_value".to_string(),
            KeyPath::String("value".to_string()),
            false,
            false,
        )
        .expect("Failed to create index");
        let object_store = SqliteEngine::object_store_by_name(&db.connection, "test_store")
            .expect("Failed to fetch store metadata");
        SqliteEngine::put_item(
            &db.connection,
            object_store.clone(),
            IndexedDBKeyType::Number(1.0),
            vec![1, 2, 3],
            true,
            None,
            vec![(
                "by_value".to_string(),
                false,
                IndexedDBKeyType::String("alpha".to_string()),
            )],
        )
        .expect("Failed to insert item");

        let row_count_before: i64 = db
            .connection
            .query_row(
                "SELECT COUNT(*) FROM object_data WHERE object_store_id = ?",
                rusqlite::params![object_store.id],
                |row| row.get(0),
            )
            .expect("Failed to count rows before delete");
        assert_eq!(row_count_before, 1);

        let index_row_count_before: i64 = db
            .connection
            .query_row("SELECT COUNT(*) FROM index_data", [], |row| row.get(0))
            .expect("Failed to count index rows before delete");
        assert_eq!(index_row_count_before, 1);

        db.delete_store("test_store")
            .expect("Failed to delete store");

        let row_count_after: i64 = db
            .connection
            .query_row(
                "SELECT COUNT(*) FROM object_data WHERE object_store_id = ?",
                rusqlite::params![object_store.id],
                |row| row.get(0),
            )
            .expect("Failed to count rows after delete");
        assert_eq!(row_count_after, 0);

        let index_row_count_after: i64 = db
            .connection
            .query_row("SELECT COUNT(*) FROM index_data", [], |row| row.get(0))
            .expect("Failed to count index rows after delete");
        assert_eq!(index_row_count_after, 0);
    }

    #[test]
    fn test_index_reads_and_unique_constraints() {
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
        assert_eq!(
            db.create_index(
                "test_store",
                "by_value".to_string(),
                KeyPath::String("value".to_string()),
                false,
                false,
            )
            .expect("Failed to create value index"),
            CreateObjectResult::Created
        );
        assert_eq!(
            db.create_index(
                "test_store",
                "by_unique".to_string(),
                KeyPath::String("unique".to_string()),
                true,
                false,
            )
            .expect("Failed to create unique index"),
            CreateObjectResult::Created
        );

        let store = SqliteEngine::object_store_by_name(&db.connection, "test_store")
            .expect("Failed to get object store");

        SqliteEngine::put_item(
            &db.connection,
            store.clone(),
            IndexedDBKeyType::Number(1.0),
            vec![1],
            false,
            None,
            vec![
                (
                    "by_value".to_string(),
                    false,
                    IndexedDBKeyType::String("alpha".to_string()),
                ),
                (
                    "by_unique".to_string(),
                    true,
                    IndexedDBKeyType::String("email-1".to_string()),
                ),
            ],
        )
        .expect("Failed to insert first indexed item");
        SqliteEngine::put_item(
            &db.connection,
            store.clone(),
            IndexedDBKeyType::Number(2.0),
            vec![2],
            false,
            None,
            vec![
                (
                    "by_value".to_string(),
                    false,
                    IndexedDBKeyType::String("alpha".to_string()),
                ),
                (
                    "by_unique".to_string(),
                    true,
                    IndexedDBKeyType::String("email-2".to_string()),
                ),
            ],
        )
        .expect("Failed to insert second indexed item");

        let key = SqliteEngine::index_get_key(
            &db.connection,
            store.clone(),
            "by_value".to_string(),
            IndexedDBKeyRange::only(IndexedDBKeyType::String("alpha".to_string())),
        )
        .expect("Failed to get indexed key")
        .map(|key| encoding::deserialize(&key).unwrap());
        assert_eq!(key, Some(IndexedDBKeyType::Number(1.0)));

        let item = SqliteEngine::index_get_item(
            &db.connection,
            store.clone(),
            "by_value".to_string(),
            IndexedDBKeyRange::only(IndexedDBKeyType::String("alpha".to_string())),
        )
        .expect("Failed to get indexed item");
        assert_eq!(item, Some(vec![1]));

        let keys = SqliteEngine::index_get_all_keys(
            &db.connection,
            store.clone(),
            "by_value".to_string(),
            IndexedDBKeyRange::only(IndexedDBKeyType::String("alpha".to_string())),
            None,
        )
        .expect("Failed to get indexed keys")
        .into_iter()
        .map(|key| encoding::deserialize(&key).unwrap())
        .collect::<Vec<_>>();
        assert_eq!(
            keys,
            vec![IndexedDBKeyType::Number(1.0), IndexedDBKeyType::Number(2.0)]
        );

        let items = SqliteEngine::index_get_all_items(
            &db.connection,
            store.clone(),
            "by_value".to_string(),
            IndexedDBKeyRange::only(IndexedDBKeyType::String("alpha".to_string())),
            None,
        )
        .expect("Failed to get indexed items");
        assert_eq!(items, vec![vec![1], vec![2]]);

        let count = SqliteEngine::index_count(
            &db.connection,
            store.clone(),
            "by_value".to_string(),
            IndexedDBKeyRange::only(IndexedDBKeyType::String("alpha".to_string())),
        )
        .expect("Failed to count indexed entries");
        assert_eq!(count, 2);

        SqliteEngine::put_item(
            &db.connection,
            store.clone(),
            IndexedDBKeyType::Number(1.0),
            vec![10],
            true,
            None,
            vec![
                (
                    "by_value".to_string(),
                    false,
                    IndexedDBKeyType::String("beta".to_string()),
                ),
                (
                    "by_unique".to_string(),
                    true,
                    IndexedDBKeyType::String("email-1b".to_string()),
                ),
            ],
        )
        .expect("Failed to overwrite indexed item");

        let alpha_count = SqliteEngine::index_count(
            &db.connection,
            store.clone(),
            "by_value".to_string(),
            IndexedDBKeyRange::only(IndexedDBKeyType::String("alpha".to_string())),
        )
        .expect("Failed to count alpha indexed entries");
        assert_eq!(alpha_count, 1);

        let beta_item = SqliteEngine::index_get_item(
            &db.connection,
            store.clone(),
            "by_value".to_string(),
            IndexedDBKeyRange::only(IndexedDBKeyType::String("beta".to_string())),
        )
        .expect("Failed to get beta indexed item");
        assert_eq!(beta_item, Some(vec![10]));

        let duplicate_unique_result = SqliteEngine::put_item(
            &db.connection,
            store,
            IndexedDBKeyType::Number(3.0),
            vec![3],
            false,
            None,
            vec![(
                "by_unique".to_string(),
                true,
                IndexedDBKeyType::String("email-2".to_string()),
            )],
        );
        assert!(duplicate_unique_result.is_err());
    }

    #[test]
    fn test_index_cleanup_and_store_scoped_names() {
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

        db.create_store("store_a", None, false)
            .expect("Failed to create first store");
        db.create_store("store_b", None, false)
            .expect("Failed to create second store");
        assert_eq!(
            db.create_index(
                "store_a",
                "shared_name".to_string(),
                KeyPath::String("a".to_string()),
                false,
                false,
            )
            .expect("Failed to create first shared-name index"),
            CreateObjectResult::Created
        );
        assert_eq!(
            db.create_index(
                "store_b",
                "shared_name".to_string(),
                KeyPath::String("b".to_string()),
                false,
                false,
            )
            .expect("Failed to create second shared-name index"),
            CreateObjectResult::Created
        );

        let store_a = SqliteEngine::object_store_by_name(&db.connection, "store_a")
            .expect("Failed to get first store");
        SqliteEngine::put_item(
            &db.connection,
            store_a.clone(),
            IndexedDBKeyType::Number(1.0),
            vec![1],
            false,
            None,
            vec![(
                "shared_name".to_string(),
                false,
                IndexedDBKeyType::String("alpha".to_string()),
            )],
        )
        .expect("Failed to insert indexed item");

        let index_rows_before_delete: i64 = db
            .connection
            .query_row("SELECT COUNT(*) FROM index_data", [], |row| row.get(0))
            .expect("Failed to count index rows before index delete");
        assert_eq!(index_rows_before_delete, 1);

        db.delete_index("store_a", "shared_name".to_string())
            .expect("Failed to delete index");

        let index_rows_after_delete: i64 = db
            .connection
            .query_row("SELECT COUNT(*) FROM index_data", [], |row| row.get(0))
            .expect("Failed to count index rows after index delete");
        assert_eq!(index_rows_after_delete, 0);

        SqliteEngine::put_item(
            &db.connection,
            store_a.clone(),
            IndexedDBKeyType::Number(2.0),
            vec![2],
            false,
            None,
            vec![],
        )
        .expect("Failed to insert item before clear");
        SqliteEngine::clear(&db.connection, store_a.clone()).expect("Failed to clear store");
        let remaining_records =
            SqliteEngine::count(&db.connection, store_a, IndexedDBKeyRange::default())
                .expect("Failed to count records after clear");
        assert_eq!(remaining_records, 0);
    }

    #[test]
    fn test_delete_item_removes_index_entries() {
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
        db.create_index(
            "test_store",
            "by_value".to_string(),
            KeyPath::String("value".to_string()),
            false,
            false,
        )
        .expect("Failed to create index");
        let store = SqliteEngine::object_store_by_name(&db.connection, "test_store")
            .expect("Failed to get object store");

        SqliteEngine::put_item(
            &db.connection,
            store.clone(),
            IndexedDBKeyType::Number(1.0),
            vec![1],
            false,
            None,
            vec![(
                "by_value".to_string(),
                false,
                IndexedDBKeyType::String("alpha".to_string()),
            )],
        )
        .expect("Failed to insert indexed item");

        SqliteEngine::delete_item(
            &db.connection,
            store.clone(),
            IndexedDBKeyRange::only(IndexedDBKeyType::Number(1.0)),
        )
        .expect("Failed to delete indexed item");

        let indexed_key = SqliteEngine::index_get_key(
            &db.connection,
            store.clone(),
            "by_value".to_string(),
            IndexedDBKeyRange::only(IndexedDBKeyType::String("alpha".to_string())),
        )
        .expect("Failed to get indexed key after delete");
        assert_eq!(indexed_key, None);

        let index_rows_after_delete: i64 = db
            .connection
            .query_row("SELECT COUNT(*) FROM index_data", [], |row| row.get(0))
            .expect("Failed to count index rows after item delete");
        assert_eq!(index_rows_after_delete, 0);
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
        let put_overwrite = get_channel();
        let get_item_some = get_channel();
        let get_item_none = get_channel();
        let get_all_items = get_channel();
        let count = get_channel();
        let remove = get_channel();
        let clear = get_channel();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        db.process_transaction(
            KvsTransaction {
                mode: IndexedDBTxnMode::Readwrite,
                requests: VecDeque::from(vec![
                    KvsOperation {
                        store_name: store_name.to_owned(),
                        operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                            callback: get_callback(put.0),
                            key: Some(IndexedDBKeyType::Number(1.0)),
                            value: vec![1, 2, 3],
                            should_overwrite: false,
                            key_generator_current_number: None,
                            index_key_value: vec![],
                        }),
                    },
                    KvsOperation {
                        store_name: store_name.to_owned(),
                        operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                            callback: get_callback(put2.0),
                            key: Some(IndexedDBKeyType::String("2.0".to_string())),
                            value: vec![4, 5, 6],
                            should_overwrite: false,
                            key_generator_current_number: None,
                            index_key_value: vec![],
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
                            key_generator_current_number: None,
                            index_key_value: vec![],
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
                            key_generator_current_number: None,
                            index_key_value: vec![],
                        }),
                    },
                    KvsOperation {
                        store_name: store_name.to_owned(),
                        operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                            callback: get_callback(put_overwrite.0),
                            key: Some(IndexedDBKeyType::Number(1.0)),
                            value: vec![13, 14, 15],
                            should_overwrite: true,
                            key_generator_current_number: None,
                            index_key_value: vec![],
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
            },
            Box::new(move || {
                let _ = done_tx.send(());
            }),
        );
        let _ = done_rx.recv().unwrap();
        put.1.recv().unwrap().unwrap();
        put2.1.recv().unwrap().unwrap();
        put3.1.recv().unwrap().unwrap();
        let err = put_dup.1.recv().unwrap().unwrap();
        assert_eq!(err, PutItemResult::CannotOverwrite);
        let overwritten = put_overwrite.1.recv().unwrap().unwrap();
        assert_eq!(
            overwritten,
            PutItemResult::Key(IndexedDBKeyType::Number(1.0))
        );
        let get_result = get_item_some.1.recv().unwrap();
        let value = get_result.unwrap();
        assert_eq!(value, Some(vec![13, 14, 15]));
        let get_result = get_item_none.1.recv().unwrap();
        let value = get_result.unwrap();
        assert_eq!(value, None);
        let all_items = get_all_items.1.recv().unwrap().unwrap();
        assert_eq!(all_items.len(), 3);
        // Check that all three items are present
        assert!(all_items.contains(&vec![13, 14, 15]));
        assert!(all_items.contains(&vec![4, 5, 6]));
        assert!(all_items.contains(&vec![7, 8, 9]));
        let amount = count.1.recv().unwrap().unwrap();
        assert_eq!(amount, 1);
        remove.1.recv().unwrap().unwrap();
        clear.1.recv().unwrap().unwrap();
    }

    #[test]
    fn test_delete_item_range_respects_open_bounds() {
        fn remaining_keys_after_delete(
            lower: i32,
            upper: i32,
            lower_open: bool,
            upper_open: bool,
        ) -> Vec<i32> {
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
            let store = SqliteEngine::object_store_by_name(&db.connection, store_name)
                .expect("Failed to get object store");

            for key in 1..=10 {
                SqliteEngine::put_item(
                    &db.connection,
                    store.clone(),
                    IndexedDBKeyType::Number(key as f64),
                    vec![key as u8],
                    false,
                    None,
                    vec![],
                )
                .expect("Failed to seed object store");
            }

            SqliteEngine::delete_item(
                &db.connection,
                store.clone(),
                IndexedDBKeyRange::new(
                    Some(IndexedDBKeyType::Number(lower as f64)),
                    Some(IndexedDBKeyType::Number(upper as f64)),
                    lower_open,
                    upper_open,
                ),
            )
            .expect("Failed to delete key range");

            SqliteEngine::get_all_keys(&db.connection, store, IndexedDBKeyRange::default(), None)
                .expect("Failed to read remaining keys")
                .into_iter()
                .map(|raw_key| match encoding::deserialize(&raw_key).unwrap() {
                    IndexedDBKeyType::Number(number) => number as i32,
                    other => panic!("Expected numeric key, got {other:?}"),
                })
                .collect()
        }

        assert_eq!(
            remaining_keys_after_delete(3, 8, false, false),
            vec![1, 2, 9, 10]
        );
        assert_eq!(
            remaining_keys_after_delete(3, 8, true, false),
            vec![1, 2, 3, 9, 10]
        );
        assert_eq!(
            remaining_keys_after_delete(3, 8, false, true),
            vec![1, 2, 8, 9, 10]
        );
        assert_eq!(
            remaining_keys_after_delete(3, 8, true, true),
            vec![1, 2, 3, 8, 9, 10]
        );
    }
}
