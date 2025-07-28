/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::path::{Path, PathBuf};

use net_traits::indexeddb_thread::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, BackendError,
    CreateObjectResult, IndexedDBKeyRange, KeyPath, PutItemResult,
};
use sea_orm::prelude::*;
use sea_orm::sea_query::IntoCondition;
use sea_orm::{Condition, Database, IntoActiveModel, NotSet, Set};
use tokio::sync::oneshot;

use crate::async_runtime::HANDLE;
use crate::indexeddb::engines::{KvsEngine, KvsTransaction, SanitizedName};
use crate::indexeddb::idb_thread::IndexedDBDescription;

mod database_model;
mod object_data_model;
mod object_store_index_model;
mod object_store_model;

// These pragmas need to be set once
const DB_INIT_PRAGMAS: [&str; 2] = ["PRAGMA journal_mode = WAL;", "PRAGMA encoding = 'UTF-16';"];

// These pragmas need to be run once a connection.
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
        return object_data_model::Column::Data.eq(encoded).into_condition();
    }
    let mut parts = vec![];
    if let Some(upper) = range.upper.as_ref() {
        let upper_bytes = bincode::serialize(upper).unwrap();
        let query = if range.upper_open {
            object_data_model::Column::Data.lt(upper_bytes)
        } else {
            object_data_model::Column::Data.lte(upper_bytes)
        };
        parts.push(query);
    }
    if let Some(lower) = range.lower.as_ref() {
        let lower_bytes = bincode::serialize(lower).unwrap();
        let query = if range.upper_open {
            object_data_model::Column::Data.gt(lower_bytes)
        } else {
            object_data_model::Column::Data.gte(lower_bytes)
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
    connection: DatabaseConnection,
}

impl SqliteEngine {
    pub fn new(base_dir: &Path, db_info: &IndexedDBDescription, version: u64) -> Self {
        let mut db_path = PathBuf::new();
        db_path.push(base_dir);
        db_path.push(db_info.as_path());
        db_path.push("db.sqlite");

        let connection = if db_path.exists() {
            HANDLE.block_on(async { Self::get_connection(&db_path).await })
        } else {
            std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
            std::fs::File::create(&db_path).unwrap();
            HANDLE.block_on(async { Self::init_db(&db_path, db_info, version).await })
        }
        .unwrap();

        for stmt in DB_PRAGMAS {
            HANDLE
                .block_on(connection.execute_unprepared(stmt))
                .unwrap();
        }

        Self {
            connection,
            db_path,
        }
    }

    async fn get_connection(path: &Path) -> Result<DatabaseConnection, DbErr> {
        Database::connect(format!("sqlite://{}", path.display())).await
    }

    async fn init_db(
        path: &Path,
        db_info: &IndexedDBDescription,
        version: u64,
    ) -> Result<DatabaseConnection, DbErr> {
        async fn create_table<T: EntityTrait>(
            connection: &DatabaseConnection,
            p: T,
        ) -> Result<(), DbErr> {
            let builder = connection.get_database_backend();
            let schema = sea_orm::Schema::new(builder);
            let create_table_stmt = builder.build(&schema.create_table_from_entity(p));
            connection.execute(create_table_stmt).await?;
            Ok(())
        }

        let connection = Self::get_connection(path).await?;
        for stmt in DB_INIT_PRAGMAS {
            connection.execute_unprepared(stmt).await?;
        }
        create_table(&connection, database_model::Entity).await?;
        create_table(&connection, object_data_model::Entity).await?;
        create_table(&connection, object_store_index_model::Entity).await?;
        create_table(&connection, object_store_model::Entity).await?;
        let info = database_model::ActiveModel {
            name: Set(db_info.name.to_owned()),
            origin: Set(db_info.origin.to_owned().ascii_serialization()),
            version: Set(i64::from_ne_bytes(version.to_ne_bytes())),
        };
        info.insert(&connection).await?;
        Ok(connection)
    }
}

impl KvsEngine for SqliteEngine {
    type Error = DbErr;

    fn create_store(
        &self,
        store_name: SanitizedName,
        key_path: Option<KeyPath>,
        auto_increment: bool,
    ) -> Result<CreateObjectResult, Self::Error> {
        HANDLE.block_on(async {
            if object_store_model::Entity::find()
                .filter(object_store_model::Column::Name.eq(store_name.to_string()))
                .one(&self.connection)
                .await?
                .is_some()
            {
                return Ok(CreateObjectResult::AlreadyExists);
            }
            let model = object_store_model::ActiveModel {
                id: NotSet,
                name: Set(store_name.to_string()),
                key_path: Set(key_path.map(|v| bincode::serialize(&v).unwrap())),
                auto_increment: Set(auto_increment),
            };
            model.insert(&self.connection).await?;

            Ok(CreateObjectResult::Created)
        })
    }

    fn delete_store(&self, store_name: SanitizedName) -> Result<(), Self::Error> {
        HANDLE.block_on(async {
            if let Some(store) = object_store_model::Entity::find()
                .filter(object_store_model::Column::Name.eq(store_name.to_string()))
                .one(&self.connection)
                .await?
            {
                object_store_index_model::Entity::delete_many()
                    .filter(object_store_index_model::Column::ObjectStoreId.eq(store.id))
                    .exec(&self.connection)
                    .await?;
                object_data_model::Entity::delete_many()
                    .filter(object_data_model::Column::ObjectStoreId.eq(store.id))
                    .exec(&self.connection)
                    .await?;
                store.delete(&self.connection).await?;
            }
            Ok(())
        })
    }

    fn close_store(&self, _store_name: SanitizedName) -> Result<(), Self::Error> {
        HANDLE.block_on(async {
            // TODO: do something
            Ok(())
        })
    }

    fn delete_database(self) -> Result<(), Self::Error> {
        HANDLE.block_on(async {
            self.connection.close().await?;
            if self.db_path.exists() {
                // TODO: make custom error type instead.
                tokio::fs::remove_dir_all(
                    self.db_path
                        .parent()
                        .ok_or(Self::Error::Custom("No parent".to_string()))?,
                )
                .await
                .map_err(|e| Self::Error::Custom(format!("{e:?}")))?;
            }
            Ok(())
        })
    }

    fn process_transaction(
        &self,
        transaction: KvsTransaction,
    ) -> oneshot::Receiver<Option<Vec<u8>>> {
        let (tx, rx) = oneshot::channel();
        let connection = self.connection.clone();

        // TODO: maybe use different pools for different transactions?
        HANDLE.spawn(async move {
            for request in transaction.requests {
                let conn = connection.clone();
                let object_store = object_store_model::Entity::find()
                    .filter(object_store_model::Column::Name.eq(request.store_name.to_string()))
                    .one(&conn)
                    .await;
                macro_rules! process_object_store {
                    ($object_store:ident, $sender:ident) => {
                        match $object_store {
                            Ok(Some(store)) => store,
                            Ok(None) => {
                                let _ = $sender.send(Err(BackendError::StoreNotFound));
                                continue;
                            },
                            Err(e) => {
                                let _ = $sender.send(Err(BackendError::DbErr(format!("{:?}", e))));
                                continue;
                            },
                        }
                    };
                }

                match request.operation {
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                        sender,
                        key,
                        value,
                        should_overwrite,
                    }) => {
                        let object_store = process_object_store!(object_store, sender);
                        let serialized_key: Vec<u8> = bincode::serialize(&key).unwrap();
                        let store = object_data_model::ActiveModel {
                            object_store_id: Set(object_store.id),
                            key: Set(serialized_key.clone()),
                            data: Set(value),
                        };
                        let existing_item = match object_data_model::Entity::find()
                            .filter(
                                object_data_model::Column::Key
                                    .eq(serialized_key.clone())
                                    .and(
                                        object_data_model::Column::ObjectStoreId
                                            .eq(object_store.id),
                                    ),
                            )
                            .one(&conn)
                            .await
                        {
                            Ok(t) => t,
                            Err(e) => {
                                let _ = sender.send(Err(BackendError::DbErr(format!("{:?}", e))));
                                continue;
                            },
                        };
                        if should_overwrite || existing_item.is_none() {
                            match store.insert(&conn).await {
                                Ok(_) => {
                                    let _ = sender.send(Ok(PutItemResult::Success));
                                },
                                Err(e) => {
                                    let _ =
                                        sender.send(Err(BackendError::DbErr(format!("{:?}", e))));
                                },
                            }
                        } else {
                            let _ = sender.send(Ok(PutItemResult::CannotOverwrite));
                        }
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem {
                        sender,
                        key_range,
                    }) => {
                        let object_store = process_object_store!(object_store, sender);
                        let result = object_data_model::Entity::find()
                            .filter(object_data_model::Column::ObjectStoreId.eq(object_store.id))
                            .filter(range_to_query(key_range))
                            .one(&conn)
                            .await;

                        match result {
                            Ok(result) => {
                                let _ = sender.send(Ok(result.map(|blob| blob.data.to_vec())));
                            },
                            Err(e) => {
                                let _ = sender.send(Err(BackendError::DbErr(format!("{:?}", e))));
                            },
                        }
                    },
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem {
                        sender,
                        key,
                    }) => {
                        let object_store = process_object_store!(object_store, sender);
                        let serialized_key: Vec<u8> = bincode::serialize(&key).unwrap();
                        // More ergonomic way to delete an item than querying first.
                        let result =
                            object_data_model::Entity::delete_many()
                                .filter(object_data_model::Column::Key.eq(serialized_key).and(
                                    object_data_model::Column::ObjectStoreId.eq(object_store.id),
                                ))
                                .exec(&conn)
                                .await;
                        if let Err(err) = result {
                            let _ = sender.send(Err(BackendError::DbErr(format!("{:?}", err))));
                        } else {
                            let _ = sender.send(Ok(()));
                        }
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count {
                        sender,
                        key_range,
                    }) => {
                        let object_store = process_object_store!(object_store, sender);
                        let res = object_data_model::Entity::find()
                            .filter(object_data_model::Column::ObjectStoreId.eq(object_store.id))
                            .filter(range_to_query(key_range))
                            .all(&conn)
                            .await;
                        match res {
                            Ok(list) => {
                                // TODO: make that return usize instead of u64
                                let _ = sender.send(Ok(list.len() as u64));
                            },
                            Err(e) => {
                                let _ = sender.send(Err(BackendError::DbErr(format!("{:?}", e))));
                            },
                        }
                    },
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::Clear(sender)) => {
                        let object_store = process_object_store!(object_store, sender);
                        let result = object_data_model::Entity::delete_many()
                            .filter(object_data_model::Column::ObjectStoreId.eq(object_store.id))
                            .exec(&conn)
                            .await;
                        let _ = match result {
                            Ok(_) => sender.send(Ok(())),
                            Err(e) => sender.send(Err(BackendError::DbErr(format!("{:?}", e)))),
                        };
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetKey {
                        sender,
                        key_range,
                    }) => {
                        let object_store = process_object_store!(object_store, sender);
                        let result = object_data_model::Entity::find()
                            .filter(object_data_model::Column::ObjectStoreId.eq(object_store.id))
                            .filter(range_to_query(key_range))
                            .one(&conn)
                            .await;

                        match result {
                            Ok(result) => {
                                let _ =
                                    sender
                                        .send(Ok(result
                                            .map(|blob| bincode::deserialize(&blob.key).unwrap())));
                            },
                            Err(e) => {
                                let _ = sender.send(Err(BackendError::DbErr(format!("{:?}", e))));
                            },
                        }
                    },
                }
            }
            let _ = tx.send(None);
        });
        rx
    }

    // TODO: we should be able to error out here, maybe change the trait definition?
    fn has_key_generator(&self, store_name: SanitizedName) -> bool {
        HANDLE.block_on(async {
            if let Some(model) = object_store_model::Entity::find()
                .filter(object_store_model::Column::Name.eq(store_name.to_string()))
                .one(&self.connection)
                .await
                .unwrap()
            {
                model.auto_increment
            } else {
                false
            }
        })
    }

    fn key_path(&self, store_name: SanitizedName) -> Option<KeyPath> {
        HANDLE.block_on(async {
            if let Some(model) = object_store_model::Entity::find()
                .filter(object_store_model::Column::Name.eq(store_name.to_string()))
                .one(&self.connection)
                .await
                .unwrap()
            {
                model
                    .key_path
                    .map(|key_path| bincode::deserialize(&key_path).unwrap())
            } else {
                None
            }
        })
    }

    fn create_index(
        &self,
        store_name: SanitizedName,
        index_name: String,
        key_path: KeyPath,
        unique: bool,
        multi_entry: bool,
    ) -> Result<CreateObjectResult, Self::Error> {
        HANDLE.block_on(async {
            let object_store = match object_store_model::Entity::find()
                .filter(object_store_model::Column::Name.eq(store_name.to_string()))
                .one(&self.connection)
                .await?
            {
                Some(model) => model,
                None => {
                    return Err(Self::Error::Custom("No such store".to_string()));
                },
            };
            if object_store_index_model::Entity::find()
                .filter(
                    object_store_index_model::Column::Name
                        .eq(index_name.to_string())
                        .and(object_store_index_model::Column::ObjectStoreId.eq(object_store.id)),
                )
                .one(&self.connection)
                .await?
                .is_some()
            {
                return Ok(CreateObjectResult::AlreadyExists);
            }
            let model = object_store_index_model::ActiveModel {
                id: Default::default(),
                object_store_id: Set(object_store.id),
                name: Set(index_name),
                key_path: Set(bincode::serialize(&key_path).unwrap()),
                unique_index: Set(unique),
                multi_entry_index: Set(multi_entry),
            };
            model.insert(&self.connection).await?;
            Ok(CreateObjectResult::Created)
        })
    }

    fn delete_index(
        &self,
        store_name: SanitizedName,
        index_name: String,
    ) -> Result<(), Self::Error> {
        HANDLE.block_on(async {
            let object_store = match object_store_model::Entity::find()
                .filter(object_store_model::Column::Name.eq(store_name.to_string()))
                .one(&self.connection)
                .await?
            {
                Some(model) => model,
                None => {
                    return Err(Self::Error::Custom("No such store".to_string()));
                },
            };
            if let Some(model) = object_store_index_model::Entity::find()
                .filter(
                    object_store_index_model::Column::Name
                        .eq(index_name.to_string())
                        .and(object_store_index_model::Column::ObjectStoreId.eq(object_store.id)),
                )
                .one(&self.connection)
                .await?
            {
                model.delete(&self.connection).await?;
            }
            Ok(())
        })
    }

    fn version(&self) -> Result<u64, Self::Error> {
        HANDLE.block_on(async {
            let db_info = database_model::Entity::find()
                .one(&self.connection)
                .await?
                .unwrap();
            Ok(u64::from_ne_bytes(db_info.version.to_ne_bytes()))
        })
    }

    fn set_version(&self, version: u64) -> Result<(), Self::Error> {
        HANDLE.block_on(async {
            let db_info = database_model::Entity::find()
                .one(&self.connection)
                .await?
                .unwrap();
            let mut db_info = db_info.into_active_model();
            db_info.version = Set(i64::from_ne_bytes(version.to_ne_bytes()));
            db_info.save(&self.connection).await?;
            Ok(())
        })
    }
}
