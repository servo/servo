/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use net_traits::indexeddb_thread::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, IdbResult,
};
use sea_orm::prelude::*;
use sea_orm::{Database, NotSet, Set};
use tokio::sync::{RwLock, oneshot};

use crate::async_runtime::HANDLE;
use crate::indexeddb::engines::{KvsEngine, KvsTransaction, SanitizedName};

mod metadata_model;
mod store_model;

pub struct SqliteEngine {
    db_dir: PathBuf,
    connections: Arc<RwLock<HashMap<SanitizedName, DatabaseConnection>>>,
}

impl SqliteEngine {
    pub fn new(base_dir: &Path, db_dir_name: &Path) -> Self {
        let mut db_dir = PathBuf::new();
        db_dir.push(base_dir);
        db_dir.push(db_dir_name);
        std::fs::create_dir_all(&db_dir).expect("Could not create OS directory for idb");

        Self {
            db_dir: db_dir,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl KvsEngine for SqliteEngine {
    type Error = sea_orm::error::DbErr;

    fn create_store(
        &self,
        store_name: SanitizedName,
        auto_increment: bool,
    ) -> Result<(), Self::Error> {
        HANDLE.block_on(async {
            let conn = Database::connect(&format!(
                "sqlite://{}/{}.db",
                self.db_dir.display(),
                store_name.name
            ))
            .await?;
            let builder = conn.get_database_backend();
            let schema = sea_orm::Schema::new(builder);
            let create_table_stmt =
                builder.build(&schema.create_table_from_entity(store_model::Entity));
            conn.execute(create_table_stmt).await?;
            let create_table_stmt =
                builder.build(&schema.create_table_from_entity(metadata_model::Entity));
            conn.execute(create_table_stmt).await?;
            if auto_increment {
                let metadata = metadata_model::ActiveModel {
                    key: Set(store_name.name.clone()),
                    value: Set(1),
                };
                metadata.insert(&conn).await?;
            } else {
                let metadata = metadata_model::ActiveModel {
                    key: Set(store_name.name.clone()),
                    value: Set(0),
                };
                metadata.insert(&conn).await?;
            }

            let mut connections = self.connections.write().await;
            connections.insert(store_name.clone(), conn);
            Ok(())
        })
    }

    fn delete_store(&self, store_name: SanitizedName) -> Result<(), Self::Error> {
        HANDLE.block_on(async {
            let mut connections = self.connections.write().await;
            if let Some(conn) = connections.remove(&store_name) {
                store_model::Entity::delete_many().exec(&conn).await?;
                metadata_model::Entity::delete_many().exec(&conn).await?;
                conn.close().await?;
                // TODO: perhaps delete the database file as well? (not really needed for now)
            }
            Ok(())
        })
    }

    fn close_store(&self, store_name: SanitizedName) -> Result<(), Self::Error> {
        HANDLE.block_on(async {
            let mut connections = self.connections.write().await;
            if let Some(conn) = connections.remove(&store_name) {
                conn.close().await?;
            }
            Ok(())
        })
    }

    fn process_transaction(
        &self,
        transaction: KvsTransaction,
    ) -> oneshot::Receiver<Option<Vec<u8>>> {
        let (tx, rx) = oneshot::channel();
        let connections = self.connections.clone();

        // TODO: maybe use different pools for different transactions?
        HANDLE.spawn(async move {
            let mut results = Vec::with_capacity(transaction.requests.len());

            for request in transaction.requests {
                let connections_reader = connections.read().await;
                let conn = match connections_reader.get(&request.store_name) {
                    Some(conn) => conn,
                    None => {
                        // TODO: This is also kinda wrong, but atleast we don't panic.
                        tx.send(None).unwrap_or(());
                        return;
                    },
                };

                match request.operation {
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem(
                        key,
                        value,
                        overwrite,
                    )) => {
                        let serialized_key: Vec<u8> = bincode::serialize(&key).unwrap();
                        let store = store_model::ActiveModel {
                            id: NotSet,
                            key: Set(serialized_key.clone()),
                            value: Set(value),
                        };
                        if overwrite ||
                            store_model::Entity::find()
                                .filter(store_model::Column::Key.eq(serialized_key.clone()))
                                .one(conn)
                                .await
                                .unwrap()
                                .is_none()
                        {
                            if let Ok(_) = store.insert(conn).await {
                                results.push((request.sender, Ok(Some(IdbResult::Key(key)))));
                            } else {
                                results.push((request.sender, Err(())));
                            }
                        } else {
                            results.push((request.sender, Err(())));
                        }
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem(key)) => {
                        let serialized_key: Vec<u8> = bincode::serialize(&key).unwrap();
                        let result = store_model::Entity::find()
                            .filter(store_model::Column::Key.eq(serialized_key))
                            .one(conn)
                            .await;
                        if let Ok(result) = result {
                            results.push((
                                request.sender,
                                Ok(result.map(|blob| IdbResult::Data(blob.value.to_vec()))),
                            ));
                        } else {
                            results.push((request.sender, Err(())));
                        }
                    },
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem(key)) => {
                        let serialized_key: Vec<u8> = bincode::serialize(&key).unwrap();
                        // More ergonomic way to delete an item than querying first.
                        let result = store_model::Entity::delete_many()
                            .filter(store_model::Column::Key.eq(serialized_key))
                            .exec(conn)
                            .await
                            .map_err(|_| ())
                            .and_then(|delete_result| {
                                if delete_result.rows_affected > 0 {
                                    Ok(Some(IdbResult::Key(key)))
                                } else {
                                    Ok(None)
                                }
                            });
                        results.push((request.sender, result));
                    },
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count(key)) => {
                        let serialized_key: Vec<u8> = bincode::serialize(&key).unwrap();
                        let count = store_model::Entity::find()
                            .filter(store_model::Column::Key.eq(serialized_key))
                            .count(conn)
                            .await;
                        // TODO: return the count as an IdbResult
                        match count {
                            Ok(_count) => results.push((request.sender, Ok(None))),
                            Err(_) => results.push((request.sender, Err(()))),
                        }
                    },
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::Clear) => {
                        let result = store_model::Entity::delete_many().exec(conn).await;
                        match result {
                            Ok(_) => results.push((request.sender, Ok(None))),
                            Err(_) => results.push((request.sender, Err(()))),
                        }
                    },
                }
            }

            for (sender, result) in results {
                if let Err(_) = sender.send(result) {
                    // The receiver was dropped, we can ignore this.
                }
            }
        });
        rx
    }

    // TODO: we should be able to error out here, maybe change the trait definition?
    fn has_key_generator(&self, store_name: SanitizedName) -> bool {
        HANDLE.block_on(async {
            let connections = self.connections.clone();
            let connections = connections.read().await;
            if let Some(conn) = connections.get(&store_name) {
                let metadata = metadata_model::Entity::find()
                    .filter(metadata_model::Column::Key.eq(store_name.name.clone()))
                    .one(conn)
                    .await
                    .unwrap();
                if let Some(metadata) = metadata {
                    return metadata.value > 0;
                }
            }
            false
        })
    }
}
