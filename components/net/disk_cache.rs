/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;
use std::sync::Arc;

use log::error;
use malloc_size_of_derive::MallocSizeOf;
use quick_cache::Lifecycle;
use rusqlite::Row;
use sea_query::{Expr, ExprTrait, Iden, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use servo_config::pref;
use servo_url::ServoUrl;
use tokio::sync::{Mutex as TokioMutex, RwLock as TokioRwLock};

use crate::http_cache::{CacheEntry, CacheKey, CachedResource};

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

#[derive(Clone)]
/// The lifecycle hooks of the HttpCache.
/// Responsible for moving data to the disk.
pub struct DiskLifecycle {
    disk_cache: Option<Arc<DiskCache>>,
}

impl DiskLifecycle {
    fn empty() -> DiskLifecycle {
        DiskLifecycle { disk_cache: None }
    }
}

impl Lifecycle<CacheKey, CacheEntry> for DiskLifecycle {
    type RequestState = ();

    fn begin_request(&self) -> Self::RequestState {}

    fn on_evict(&self, _state: &mut Self::RequestState, key: CacheKey, value: CacheEntry) {
        if let Some(disk_cache_data) = &self.disk_cache {
            let disk_cache_data = disk_cache_data.clone();
            tokio::spawn(async move { disk_cache_data.store(key, value).await });
        }
    }
}

#[derive(MallocSizeOf)]
struct DiskCacheInner {
    entries: VecDeque<DiskCacheMetadata>,
    size: usize,
    #[ignore_malloc_size_of = "Find a better way"]
    db: rusqlite::Connection,
}

#[derive(MallocSizeOf)]
/// A struct representing the disk cache.
pub(crate) struct DiskCache {
    path: String,
    max_size: usize,
    // the non constant data.
    inner: TokioMutex<DiskCacheInner>,
}

#[derive(Iden)]
/// Identifications for sea_query of the cache table.
enum DiskCacheTable {
    Table,
    Key,
    Data,
    Size,
}

impl DiskCache {
    /// Creates a new [`DiskCache`] if the preference if set.
    /// Creates the sqlite table if it does not exist and starts the db connection.
    fn new() -> (Option<Arc<DiskCache>>, DiskLifecycle) {
        let disk_cache_path = pref!(network_http_disk_cache);
        if disk_cache_path.is_empty() {
            (None, DiskLifecycle::empty())
        } else {
            let max_disk_cache_size = pref!(network_http_disk_cache_size).try_into().unwrap();

            let Ok(db) = rusqlite::Connection::open(&disk_cache_path) else {
                error!("Could not open disk cache database");
                return (None, DiskLifecycle::empty());
            };

            let Ok(table_exists) = db.table_exists(None, "disk_cache_table") else {
                return (None, DiskLifecycle::empty());
            };

            if !table_exists {
                if let Err(e) = db.execute(
                    "CREATE TABLE disk_cache_table (
                key VARCHAR PRIMARY KEY,
                data VARCHAR NOT NULL,
                size INTEGER NOT NULL);",
                    [],
                ) {
                    error!("Could not create table. DB Error {:?}", e);
                    return (None, DiskLifecycle::empty());
                }
            }

            let (query, values) = Query::select()
                .columns([DiskCacheTable::Key, DiskCacheTable::Size])
                .from(DiskCacheTable::Table)
                .build_rusqlite(SqliteQueryBuilder);

            let (entries, size) = {
                let Ok(mut st) = db.prepare(query.as_str()) else {
                    error!("Could not get disk data");
                    return (None, DiskLifecycle::empty());
                };

                let entries = st
                    .query_map(&*values.as_params(), |row| Ok(DiskCacheMetadata::from(row)))
                    .unwrap()
                    .map(|entry| entry.unwrap())
                    .collect::<VecDeque<_>>();

                let size = entries.iter().map(|entry| entry.size).sum();
                (entries, size)
            };
            let inner = DiskCacheInner { entries, size, db };
            let disk_cache_data = std::sync::Arc::new(DiskCache {
                inner: TokioMutex::new(inner),
                path: disk_cache_path,
                max_size: max_disk_cache_size,
            });

            (
                Some(disk_cache_data.clone()),
                DiskLifecycle {
                    disk_cache: Some(disk_cache_data),
                },
            )
        }
    }

    /// Create a [`DiskCache`] from the disk if enabled in preferences.
    /// It is filled with all
    /// the responses except `number_of_responses` which were read from the
    /// [`DiskCache`] and then removed and returned for adding to the memory cache.
    pub(crate) fn maybe_from_disk(
        _number_of_responses: usize,
    ) -> (
        Option<Arc<DiskCache>>,
        DiskLifecycle,
        Vec<(CacheKey, Vec<CachedResource>)>,
    ) {
        let (disk_cache, lifecycle) = DiskCache::new();

        // Notice that CachedResponse have a duration which needs to be adjusted for restoring.
        (disk_cache, lifecycle, vec![])
    }

    /// Stores the given responses to the disk cache, assuming the cache will not be used afterwards.
    pub(crate) fn store_cache_to_disk<
        T: Iterator<Item = (CacheKey, Arc<TokioRwLock<Vec<CachedResource>>>)>,
    >(
        &self,
        _resources: T,
    ) {
    }

    /// Restores a cache entry from the disk if it exists.
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
        let value: Vec<CachedResource> = postcard::from_bytes(&bytes).unwrap();
        let deserialized_vec_cached_response = std::sync::Arc::new(TokioRwLock::new(value));

        Some(deserialized_vec_cached_response)
    }

    /// Stores a [`CacheEntry`]` to disk.
    #[servo_tracing::instrument(skip(self))]
    pub(crate) async fn store(&self, key: CacheKey, entry: CacheEntry) {
        let data_to_serialize = entry.read().await;
        let Ok(data) = postcard::to_stdvec(&*data_to_serialize) else {
            error!("Could not deserialize value");
            return;
        };

        {
            let mut inner = self.inner.lock().await;
            let data_size = data.len();

            let (query, params) = Query::insert()
                .into_table(DiskCacheTable::Table)
                .columns([
                    DiskCacheTable::Key,
                    DiskCacheTable::Data,
                    DiskCacheTable::Size,
                ])
                .values_panic([key.as_ref().into(), data.into(), (data_size as u32).into()])
                .build_rusqlite(SqliteQueryBuilder);

            if let Err(e) = inner.db.execute(query.as_str(), &*params.as_params()) {
                error!("Could not insert cache data. Error {}", e);
            }
            if let Some(key_position) = inner
                .entries
                .iter()
                .position(|metadata| metadata.key == key)
            {
                inner.entries.remove(key_position);
            }
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
        let mut st = conn.prepare(size.as_str()).unwrap();

        error!("Size Query {}", size.to_string());
        let query_result = st.query_one(&*size_values.as_params(), |row| Ok(row.get_unwrap(0)));
        if let Err(query_result) = query_result {
            error!("Could nto get new sum size {}", query_result);
            None
        } else {
            query_result.ok()
        }
    }

    /// Clears the disk cache.
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
