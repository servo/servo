/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use log::error;
use malloc_size_of_derive::MallocSizeOf;
use rusqlite::Row;
use sea_query::{ColumnDef, Expr, ExprTrait, Iden, OnConflict, Query, SqliteQueryBuilder, Table};
use sea_query_rusqlite::RusqliteBinder;
use servo_config::pref;
use servo_url::ServoUrl;
use tokio::sync::{Mutex as TokioMutex, RwLock as TokioRwLock};

use crate::http_cache::{
    CacheEntry, CacheKey, CachedResource, HttpCacheAssignment, MemoryCacheLifecycle,
};

#[derive(MallocSizeOf)]
pub(crate) struct DiskCacheMetadata {
    key: CacheKey,
    /// The size of the serialization or the exact size from the cache metadata.
    size: usize,
}

impl From<&Row<'_>> for DiskCacheMetadata {
    fn from(row: &Row) -> Self {
        let s: String = row.get_unwrap("key");
        Self {
            key: CacheKey::from_url(ServoUrl::parse(&s).unwrap()),
            size: row.get_unwrap("size"),
        }
    }
}

/// This data structure will be per [`HttpCacheAssignment`]. Currently, we only store HttpCacheAssignment::Public and otherwise return zero.
/// As this data structure stores the state of the cache, if other instances share the same sqlite database, special care has to be taken
/// to ensure conistency.
#[derive(MallocSizeOf)]
struct DiskCacheInner {
    entries: VecDeque<DiskCacheMetadata>,
    size: usize,
    #[ignore_malloc_size_of = "Find a better way"]
    db: rusqlite::Connection,
    cache_assignment: HttpCacheAssignment,
}

#[derive(MallocSizeOf)]
/// A struct representing the disk cache.
pub(crate) struct DiskCache {
    path: PathBuf,
    max_size: usize,
    // the non constant data.
    inner: TokioMutex<DiskCacheInner>,
}

/// Identifications for sea_query of the cache table.
enum DiskCacheTable {
    Table,
    Key,
    Data,
    Size,
    InsertionTimestamp,
}

// Mapping between Enum variant and its corresponding string value
impl Iden for DiskCacheTable {
    fn unquoted(&self) -> &str {
        match self {
            DiskCacheTable::Table => "disk_cache",
            DiskCacheTable::Key => "key",
            DiskCacheTable::Data => "data",
            DiskCacheTable::Size => "size",
            DiskCacheTable::InsertionTimestamp => "insertion_timestamp",
        }
    }
}

/// Get the storage path out of `network_http_disk_cache` preference and `temporary_storage` option.
fn storage_dir() -> Option<PathBuf> {
    let disk_storage_path = pref!(network_http_disk_cache);
    match (
        servo_config::opts::get().temporary_storage,
        disk_storage_path.is_empty(),
    ) {
        (false, false) => Some(disk_storage_path.into()),
        (true, true) => {
            let tmp_dir = tempfile::tempdir().unwrap();
            let mut path = tmp_dir.path().to_path_buf();
            path.set_file_name("cache.sqlite3");
            Some(path)
        },
        (true, false) => {
            error!(
                "Temporary storage cannot be set with explicit disk storage path. Disabling http_disk_cache"
            );
            None
        },
        (false, true) => None,
    }
}

impl DiskCache {
    /// Creates a new [`DiskCache`] if the preference if set.
    /// Creates the sqlite table if it does not exist and starts the db connection.
    /// TODO: Implement WAL and other sqlite pragma.
    pub(crate) fn new(
        cache_assignment: HttpCacheAssignment,
    ) -> (Option<Arc<DiskCache>>, MemoryCacheLifecycle) {
        // For private browsing we currently do not want to store any disk cache.
        let disk_cache_path = storage_dir();

        if let Some(disk_cache_path) = disk_cache_path &&
            cache_assignment == HttpCacheAssignment::Public
        {
            let Ok(max_disk_cache_size) = pref!(network_http_disk_cache_size).try_into() else {
                return (None, MemoryCacheLifecycle::empty());
            };

            let Ok(db) = rusqlite::Connection::open(&disk_cache_path) else {
                error!("Could not open disk cache database");
                return (None, MemoryCacheLifecycle::empty());
            };

            let _ = db.execute("PRAGMA journal_mode = WAL;", ());
            let query = Table::create()
                .table(DiskCacheTable::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(DiskCacheTable::Key)
                        .text()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(DiskCacheTable::Data).blob().not_null())
                .col(ColumnDef::new(DiskCacheTable::Size).integer().not_null())
                .col(
                    ColumnDef::new(DiskCacheTable::InsertionTimestamp)
                        .integer()
                        .not_null(),
                )
                .build(SqliteQueryBuilder);
            if let Err(e) = db.execute(query.as_str(), ()) {
                error!("Could not create table. DB Error {:?}", e);
                return (None, MemoryCacheLifecycle::empty());
            }

            let (query, values) = Query::select()
                .columns([DiskCacheTable::Key, DiskCacheTable::Size])
                .from(DiskCacheTable::Table)
                .build_rusqlite(SqliteQueryBuilder);

            let (entries, size) = {
                let Ok(mut st) = db.prepare(query.as_str()) else {
                    error!("Could not get disk data");
                    return (None, MemoryCacheLifecycle::empty());
                };
                let entries = st
                    .query_map(&*values.as_params(), |row| Ok(DiskCacheMetadata::from(row)))
                    .unwrap()
                    .map(|entry| entry.unwrap())
                    .collect::<VecDeque<_>>();

                let size = entries.iter().map(|entry| entry.size).sum();
                (entries, size)
            };
            let inner = DiskCacheInner {
                entries,
                size,
                db,
                cache_assignment,
            };
            let disk_cache_data = std::sync::Arc::new(DiskCache {
                inner: TokioMutex::new(inner),
                path: disk_cache_path,
                max_size: max_disk_cache_size,
            });

            (
                Some(disk_cache_data.clone()),
                MemoryCacheLifecycle {
                    disk_cache: Some(disk_cache_data),
                },
            )
        } else {
            (None, MemoryCacheLifecycle::empty())
        }
    }

    /// Restores a cache entry from the disk if it exists.
    /// Deletes the entry from the disk cache
    #[servo_tracing::instrument(skip(self))]
    pub(crate) async fn get(&self, key: CacheKey) -> Option<Arc<TokioRwLock<Vec<CachedResource>>>> {
        let bytes = {
            // we lock the metadata before we update the sqlite database so that
            // the database and metadata are consistent when this lock is released.
            let mut inner = self.inner.lock().await;
            let (bytes, new_size) = {
                let _span = profile_traits::trace_span!("query disk cache").entered();
                let (query, query_values) = Query::select()
                    .columns([DiskCacheTable::Data])
                    .from(DiskCacheTable::Table)
                    .and_where(Expr::col(DiskCacheTable::Key).eq(key.as_ref()))
                    .build_rusqlite(SqliteQueryBuilder);
                let (delete, delete_values) = Query::delete()
                    .from_table(DiskCacheTable::Table)
                    .and_where(Expr::col(DiskCacheTable::Key).eq(key.as_ref()))
                    .build_rusqlite(SqliteQueryBuilder);

                let mut st = inner.db.prepare(query.as_str()).ok()?;
                let data: Vec<u8> = st
                    .query_one(&*query_values.as_params(), |row| Ok(row.get_unwrap("data")))
                    .ok()?;

                if inner
                    .db
                    .execute(delete.as_str(), &*delete_values.as_params())
                    .is_err()
                {
                    error!("Could not delete cached data from disk");
                    return None;
                }

                (data, self.get_disk_cache_total_size(&inner.db))
            };

            {
                // update the metadata
                let entry_index = inner
                    .entries
                    .iter()
                    .position(|metadata| metadata.key == key);
                if let Some(entry_index) = entry_index {
                    inner.entries.remove(entry_index);
                }
                if let Some(new_size) = new_size {
                    inner.size = new_size;
                } else {
                    error!("Could not get disk cache size");
                }
            }
            bytes
        };
        let _span = profile_traits::trace_span!("deserialize cache request").entered();
        let Ok(value) = postcard::from_bytes(&bytes) else {
            error!("Could not deserialize cached resource");
            return None;
        };
        let deserialized_vec_cached_response = std::sync::Arc::new(TokioRwLock::new(value));

        Some(deserialized_vec_cached_response)
    }

    /// Stores a [`CacheEntry`]` to disk.
    #[servo_tracing::instrument(skip(self))]
    pub(crate) async fn store(&self, key: CacheKey, entry: CacheEntry) {
        let entry = entry.read().await;
        let data_to_serialize: Vec<&CachedResource> = entry
            .iter()
            .filter(|cached_resource| cached_resource.is_done())
            .collect();
        let Ok(data) = postcard::to_stdvec(&*data_to_serialize) else {
            error!("Could not deserialize value");
            return;
        };

        {
            let mut inner = self.inner.lock().await;
            let data_size = data.len();

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0);
            let (query, params) = Query::insert()
                .into_table(DiskCacheTable::Table)
                .columns([
                    DiskCacheTable::Key,
                    DiskCacheTable::Data,
                    DiskCacheTable::Size,
                    DiskCacheTable::InsertionTimestamp,
                ])
                .on_conflict(
                    OnConflict::column(DiskCacheTable::Key)
                        .update_columns([
                            DiskCacheTable::Data,
                            DiskCacheTable::Data,
                            DiskCacheTable::Size,
                            DiskCacheTable::InsertionTimestamp,
                        ])
                        .to_owned(),
                )
                .values_panic([
                    key.as_ref().into(),
                    data.into(),
                    (data_size as u32).into(),
                    timestamp.into(),
                ])
                .build_rusqlite(SqliteQueryBuilder);

            if let Err(e) = inner.db.execute(query.as_str(), &*params.as_params()) {
                error!("Could not insert cache data. Error {}", e);
            }
            inner.entries.push_back(DiskCacheMetadata {
                key,
                size: data_size,
            });
            if let Some(new_cache_size) = self.get_disk_cache_total_size(&inner.db) {
                inner.size = new_cache_size;
            }
        }
        self.delete_until_cache_size().await;
    }

    /// Deletes data from the cache until the size is <= max_size
    #[servo_tracing::instrument(skip(self))]
    async fn delete_until_cache_size(&self) {
        let mut inner = self.inner.lock().await;
        let mut keys_to_delete = vec![];
        while self.max_size < inner.size {
            if let Some(metadata) = inner.entries.pop_back() {
                keys_to_delete.push(metadata.key);
                inner.size -= metadata.size;
            }
        }

        let keys_ref = keys_to_delete.iter().map(|key| key.as_ref());
        let (query, values) = Query::delete()
            .from_table(DiskCacheTable::Table)
            .and_where(Expr::col(DiskCacheTable::Key).is_in(keys_ref))
            .build_rusqlite(SqliteQueryBuilder);

        if inner
            .db
            .execute(query.as_str(), &*values.as_params())
            .is_err()
        {
            error!("Could not delete old disk cache entries");
        }
    }

    /// Queries the current disk cache size from the sql database.
    #[servo_tracing::instrument(skip(self))]
    fn get_disk_cache_total_size(&self, conn: &rusqlite::Connection) -> Option<usize> {
        let (size, size_values) = Query::select()
            .expr(Expr::col(DiskCacheTable::Size).sum())
            .from(DiskCacheTable::Table)
            .build_rusqlite(SqliteQueryBuilder);
        let Ok(mut st) = conn.prepare(size.as_str()) else {
            return None;
        };

        // According to the sqlite documentation we will return NULL on an empty table.
        let query_result =
            st.query_one(&*size_values.as_params(), |row| Ok(row.get(0).unwrap_or(0)));
        if let Err(query_result) = query_result {
            error!("Could nto get new sum size {}", query_result);
            None
        } else {
            query_result.ok()
        }
    }

    /// Clears the disk cache.
    /// Should only be called in sync context and will panic.
    #[servo_tracing::instrument(skip(self))]
    pub(crate) fn clear(&self) {
        let mut inner = self.inner.blocking_lock();
        let (query, params) = Query::delete()
            .from_table(DiskCacheTable::Table)
            .build_rusqlite(SqliteQueryBuilder);
        if inner
            .db
            .execute(query.as_str(), &*params.as_params())
            .is_err()
        {
            error!("Could not clear disk cache");
        }
        inner.entries.clear();
        inner.size = 0;
    }
}
