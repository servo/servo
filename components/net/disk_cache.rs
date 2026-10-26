/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Main Disk Cache implementation
//! The disk cache consists of two parts
//! 1. Storing
//! 2. Receiving
//!
//! ## Storing:
//! - For every request, construct a [`CacheStorage`] and [`CacheStorageReceiver`]. The [`CacheStorage`] is used to send encoded chunks to the sqlite cache in a special table `disk_cache_encoded`.
//!   The [`CacheStorage`] lives on the decoder side and sends the message to the [`CacheStorageReceiver`] which just puts them into the sqlite database.
//! - After a Request is finished, we mark all the values as valid, ready to be restored from.
//! - When a entry is evicted from the memory cache it ends up here, where we remove the body and store the rest of it in `disk_cache` table (serialized).
//!
//! ## Restoring
//! - We restore an entry by getting all the values form the `disk_cache_encoded`, concatenate them and decode the data with the same decoder.
//! - Then we get the serialized body from `disk_cache` (which contains the headers and other values), put the decoded body in it and send it to the memory cache.
//! - We remove the encoded values from `disk_cache_encoded` and the decoded body from `disk_cache`.
//!
//! ## Bookkeeping
//! - The cache has an additional in memory data structure to do the bookkeeping, i.e., knowing the size of the entries useful for removing them.

use std::collections::HashMap;
use std::ops::Sub;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use itertools::Itertools;
use log::error;
use malloc_size_of_derive::MallocSizeOf;
use net_traits::response::ResponseBody;
use sea_query::{
    ColumnDef, Expr, ExprTrait, Iden, OnConflict, Order, Query, SqliteQueryBuilder, Table,
};
use sea_query_rusqlite::RusqliteBinder;
use servo_config::pref;
use servo_url::ServoUrl;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{Mutex as TokioMutex, RwLock as TokioRwLock};
use url::Url;

use crate::async_runtime;
use crate::decoder::DecoderType;
use crate::http_cache::{
    CacheEntry, CacheKey, CachedResource, HttpCacheAssignment, MemoryCacheLifecycle,
};
use crate::http_loader::HttpState;

#[derive(Debug, Clone)]
pub(crate) enum InnerDecoderType {
    Plain,
    Gzip,
    Brotli,
    Deflate,
    Zstd,
}

impl From<Option<DecoderType>> for InnerDecoderType {
    fn from(value: Option<DecoderType>) -> Self {
        match value {
            Some(DecoderType::Gzip) => InnerDecoderType::Gzip,
            Some(DecoderType::Brotli) => InnerDecoderType::Brotli,
            Some(DecoderType::Deflate) => InnerDecoderType::Deflate,
            Some(DecoderType::Zstd) => InnerDecoderType::Zstd,
            None => InnerDecoderType::Plain,
        }
    }
}

impl InnerDecoderType {
    fn to_u64(&self) -> i32 {
        match self {
            InnerDecoderType::Plain => 0,
            InnerDecoderType::Gzip => 1,
            InnerDecoderType::Brotli => 2,
            InnerDecoderType::Deflate => 3,
            InnerDecoderType::Zstd => 4,
        }
    }

    fn from_u64(u: u64) -> Option<Self> {
        match u {
            0 => Some(InnerDecoderType::Plain),
            1 => Some(InnerDecoderType::Gzip),
            2 => Some(InnerDecoderType::Brotli),
            3 => Some(InnerDecoderType::Deflate),
            4 => Some(InnerDecoderType::Zstd),
            _ => None,
        }
    }
}

enum CacheStorageMessage {
    Data(Bytes),
    Exit,
}

/// The main class to send encoded bytes to the CacheStorage of in progress requests.
pub(crate) struct CacheStorage {
    sender: Option<UnboundedSender<CacheStorageMessage>>,
}

/// The other half of [`CacheStorage`], responsible for putting the bytes in the database.
struct CacheStorageReceiver {
    key: CacheKey,
    disk_cache: Arc<DiskCache>,
    decoder_type: InnerDecoderType,
    row_id: u32,
}

impl CacheStorage {
    pub(crate) fn new(
        key: CacheKey,
        state: Arc<HttpState>,
        decoder_type: InnerDecoderType,
    ) -> Self {
        let disk_cache = state.http_cache.disk_cache();
        let cache_assignment = state.http_cache.cache_assignment();

        if let Some(disk_cache) = disk_cache &&
            cache_assignment == HttpCacheAssignment::Public
        {
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

            let mut store = CacheStorageReceiver {
                key,
                disk_cache,
                decoder_type,
                row_id: 0,
            };

            tokio::spawn(async move {
                if store.delete_old_data().await.is_err() {
                    return;
                }
                while let Some(msg) = receiver.recv().await {
                    match msg {
                        CacheStorageMessage::Data(bytes) => store.store_bytes(&bytes).await,
                        CacheStorageMessage::Exit => {
                            store.finish().await;
                        },
                    }
                }
            });

            CacheStorage {
                sender: Some(sender),
            }
        } else {
            CacheStorage { sender: None }
        }
    }

    /// Send data to the cache
    pub(crate) fn send(&self, bytes: &Bytes) {
        // If we have error in sending it means our channel is closed, meaning we should continue normally.
        if let Some(sender) = &self.sender {
            let _ = sender.send(CacheStorageMessage::Data(bytes.clone()));
        }
    }

    /// the request finishes successful
    pub(crate) fn finish(&self) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(CacheStorageMessage::Exit);
        }
    }
}

impl CacheStorageReceiver {
    /// Delete data that is not finished for the same [`CacheKey`]
    async fn delete_old_data(&self) -> Result<(), ()> {
        let mut inner = self.disk_cache.inner.lock().await;
        let (query, params) = Query::delete()
            .from_table(DiskCacheEncodedTable::Table)
            .and_where(Expr::col(DiskCacheEncodedTable::Key).eq(self.key.as_ref()))
            .build_rusqlite(SqliteQueryBuilder);

        let _ = inner.entries.remove(&self.key);

        if let Err(e) = inner.db.execute(query.as_str(), &*params.as_params()) {
            log::error!(
                "Could not store partials encoded values for url {:?} with error {:?}. Ignoring.",
                self.key,
                e
            );
            Err(())
        } else {
            Ok(())
        }
    }

    /// Store the given bytes in the database
    async fn store_bytes(&mut self, bytes: &Bytes) {
        let mut disk_cache = self.disk_cache.inner.lock().await;
        let serialized_bytes = postcard::to_stdvec(bytes).unwrap();
        let serialized_len = serialized_bytes.len();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let (query, params) = Query::insert()
            .into_table(DiskCacheEncodedTable::Table)
            .columns([
                DiskCacheEncodedTable::Key,
                DiskCacheEncodedTable::Index,
                DiskCacheEncodedTable::DecoderType,
                DiskCacheEncodedTable::Data,
                DiskCacheEncodedTable::InsertionTimestamp,
            ])
            .values_panic([
                self.key.as_ref().into(),
                self.row_id.into(),
                self.decoder_type.to_u64().into(),
                serialized_bytes.into(),
                timestamp.into(),
            ])
            .build_rusqlite(SqliteQueryBuilder);

        if disk_cache
            .db
            .execute(query.as_str(), &*params.as_params())
            .is_err()
        {
            log::error!("Could not store bytes in encoded table");
        }
        self.row_id += 1;

        let metadata = disk_cache.entries.entry(self.key.clone()).or_default();
        metadata.size += serialized_len;
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        metadata.last_accessed = Some(duration);
    }

    /// Mark the rows in the table as finished, i.e, ready to be restored from.
    async fn finish(&mut self) {
        let disk_cache = self.disk_cache.inner.lock().await;
        let (query, params) = Query::update()
            .table(DiskCacheEncodedTable::Table)
            .values([(DiskCacheEncodedTable::InProgress, false.into())])
            .and_where(Expr::col(DiskCacheEncodedTable::Key).eq(self.key.as_ref()))
            .build_rusqlite(SqliteQueryBuilder);

        if disk_cache
            .db
            .execute(query.as_str(), &*params.as_params())
            .is_err()
        {
            log::error!("Could not finish storing encoded table row");
        }
    }
}

#[derive(Default, MallocSizeOf)]
pub(crate) struct DiskCacheMetadata {
    /// The size of the entry in encoded bytes.
    size: usize,
    /// Last read or inserted as duration from unix epoch.
    last_accessed: Option<Duration>,
}

fn restore_metadata_from_disk(
    entries: &mut HashMap<CacheKey, DiskCacheMetadata>,
    db: &rusqlite::Connection,
) {
    let (query, _params) = Query::select()
        .columns([
            DiskCacheEncodedTable::Index,
            DiskCacheEncodedTable::DecoderType,
            DiskCacheEncodedTable::Data,
            DiskCacheEncodedTable::InsertionTimestamp,
        ])
        .from(DiskCacheEncodedTable::Table)
        .build_rusqlite(SqliteQueryBuilder);
    let Ok(mut statement) = db.prepare(query.as_str()) else {
        // Database probably not initialized yet
        return;
    };
    let value_iter = statement
        .query_map([], |row| {
            Ok((
                row.get_unwrap::<_, usize>(DiskCacheEncodedTable::InsertionTimestamp.unquoted()),
                row.get_unwrap::<_, String>(DiskCacheEncodedTable::Key.unquoted()),
                row.get_unwrap::<_, Vec<u8>>(DiskCacheEncodedTable::Data.unquoted()),
            ))
        })
        .unwrap();
    for (timestamp, key, data) in value_iter.flatten() {
        let key_servo_url: ServoUrl = Url::parse(&key).unwrap().into();
        entries.insert(
            key_servo_url.into(),
            DiskCacheMetadata {
                size: data.len(),
                last_accessed: Some(Duration::from_secs(timestamp as u64)),
            },
        );
    }
}

/// This data structure will be per [`HttpCacheAssignment`]. Currently, we only store HttpCacheAssignment::Public and otherwise return zero.
/// As this data structure stores the state of the cache, if other instances share the same sqlite database, special care has to be taken
/// to ensure conistency.
#[derive(MallocSizeOf)]
struct DiskCacheInner {
    entries: HashMap<CacheKey, DiskCacheMetadata>,
    #[ignore_malloc_size_of = "Find a better way"]
    db: rusqlite::Connection,
}

#[derive(MallocSizeOf)]
/// A struct representing the disk cache.
pub(crate) struct DiskCache {
    path: PathBuf,
    max_size: usize,
    cache_assignment: HttpCacheAssignment,
    // the non constant data.
    inner: TokioMutex<DiskCacheInner>,
}

enum DiskCacheEncodedTable {
    Table,
    Key,
    Index,
    DecoderType,
    Data,
    InsertionTimestamp,
    InProgress,
}

impl Iden for DiskCacheEncodedTable {
    fn unquoted(&self) -> &str {
        match self {
            DiskCacheEncodedTable::Table => "disk_cache_encoded",
            DiskCacheEncodedTable::Key => "key",
            DiskCacheEncodedTable::Index => "index",
            DiskCacheEncodedTable::DecoderType => "decoder_type",
            DiskCacheEncodedTable::Data => "data",
            DiskCacheEncodedTable::InsertionTimestamp => "insertion_timestamp",
            DiskCacheEncodedTable::InProgress => "in_progress",
        }
    }
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

            let _ = db.execute_batch("PRAGMA journal_mode = WAL;PRAGMA synchronous =  NORMAL");
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

            // Making blob table
            let query = Table::create()
                .table(DiskCacheEncodedTable::Table)
                .if_not_exists()
                .col(ColumnDef::new(DiskCacheEncodedTable::Key).text().not_null())
                .col(
                    ColumnDef::new(DiskCacheEncodedTable::Index)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DiskCacheEncodedTable::DecoderType)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DiskCacheEncodedTable::Data)
                        .blob()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DiskCacheEncodedTable::InsertionTimestamp)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DiskCacheEncodedTable::InProgress)
                        .boolean()
                        .not_null()
                        .default(true),
                )
                .build(SqliteQueryBuilder);
            if let Err(e) = db.execute(query.as_str(), ()) {
                error!("Could not create table. DB Error {:?}", e);
                return (None, MemoryCacheLifecycle::empty());
            }

            let mut entries = Default::default();
            restore_metadata_from_disk(&mut entries, &db);

            let inner = DiskCacheInner { entries, db };
            let disk_cache_data = std::sync::Arc::new(DiskCache {
                inner: TokioMutex::new(inner),
                path: disk_cache_path,
                cache_assignment,
                max_size: max_disk_cache_size,
            });

            let disk_cache_data_clone = disk_cache_data.clone();
            async_runtime::spawn_task(
                async move { disk_cache_data.clean_stale_encoded_values().await },
            );
            (
                Some(disk_cache_data_clone.clone()),
                MemoryCacheLifecycle {
                    disk_cache: Some(disk_cache_data_clone),
                },
            )
        } else {
            (None, MemoryCacheLifecycle::empty())
        }
    }

    /// Removes encoded values that are not finished and older than a day.
    async fn clean_stale_encoded_values(&self) {
        let inner = self.inner.lock().await;
        let time_trim = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .sub(Duration::from_secs(24 * 60 * 60));
        let (query, params) = Query::delete()
            .from_table(DiskCacheEncodedTable::Table)
            .and_where(Expr::col(DiskCacheEncodedTable::InProgress).eq(true))
            .and_where(
                Expr::col(DiskCacheEncodedTable::InsertionTimestamp).lte(time_trim.as_secs()),
            )
            .build_rusqlite(SqliteQueryBuilder);

        let _ = inner.db.execute(&query, &*params.as_params());
    }

    /// Restores a cache entry from the disk if it exists.
    /// This does _not_ delete the entry from disk.
    #[servo_tracing::instrument(skip(self))]
    pub(crate) async fn get(&self, key: CacheKey) -> Option<Arc<TokioRwLock<Vec<CachedResource>>>> {
        let decoded_data = {
            let mut decoder_type = InnerDecoderType::Plain;
            let encoded_data: Vec<u8> = {
                let inner = self.inner.lock().await;
                let (query, query_values) = Query::select()
                    .columns([
                        DiskCacheEncodedTable::Index,
                        DiskCacheEncodedTable::DecoderType,
                        DiskCacheEncodedTable::Data,
                        DiskCacheEncodedTable::InsertionTimestamp,
                    ])
                    .from(DiskCacheEncodedTable::Table)
                    .and_where(Expr::col(DiskCacheEncodedTable::Key).eq(key.as_ref()))
                    .and_where(Expr::col(DiskCacheEncodedTable::InProgress).eq(false))
                    .order_by_columns([(DiskCacheEncodedTable::Index, Order::Asc)])
                    .build_rusqlite(SqliteQueryBuilder);

                let mut st = inner.db.prepare(query.as_str()).ok()?;
                let data_iter = st
                    .query_map(&*query_values.as_params(), |row| {
                        let data: Vec<u8> =
                            row.get_unwrap(DiskCacheEncodedTable::Data.quoted().as_ref());
                        decoder_type =
                            InnerDecoderType::from_u64(row.get_unwrap("decoder_type")).unwrap();
                        Ok(data)
                    })
                    .unwrap();

                data_iter
                    .map(|bytes| postcard::from_bytes(&bytes.unwrap()).unwrap())
                    .flat_map(|bytes: Bytes| bytes)
                    .collect()
            };

            if encoded_data.is_empty() {
                return None;
            }

            match decoder_type {
                InnerDecoderType::Plain => encoded_data,
                InnerDecoderType::Gzip => {
                    let mut final_data = Vec::new();
                    async_compression::tokio::bufread::GzipDecoder::new(&*encoded_data)
                        .read_to_end(&mut final_data)
                        .await
                        .ok()?;
                    final_data
                },
                InnerDecoderType::Brotli => {
                    let mut final_data = Vec::new();
                    async_compression::tokio::bufread::BrotliDecoder::new(&*encoded_data)
                        .read_to_end(&mut final_data)
                        .await
                        .ok()?;
                    final_data
                },
                InnerDecoderType::Deflate => {
                    let mut final_data = Vec::new();
                    async_compression::tokio::bufread::ZlibDecoder::new(&*encoded_data)
                        .read_to_end(&mut final_data)
                        .await
                        .ok()?;
                    final_data
                },
                InnerDecoderType::Zstd => {
                    let mut final_data = Vec::new();
                    async_compression::tokio::bufread::ZstdDecoder::new(&*encoded_data)
                        .read_to_end(&mut final_data)
                        .await
                        .ok()?;
                    final_data
                },
            }
        };

        let deserialized_vec_cached_response = {
            let inner = self.inner.lock().await;
            let (query, query_values) = Query::select()
                .columns([DiskCacheTable::Data])
                .from(DiskCacheTable::Table)
                .and_where(Expr::col(DiskCacheTable::Key).eq(key.as_ref()))
                .build_rusqlite(SqliteQueryBuilder);

            let mut st = inner.db.prepare(query.as_str()).ok()?;
            let cached_resource_data: Vec<u8> = st
                .query_one(&*query_values.as_params(), |row| {
                    Ok(row.get_unwrap(DiskCacheEncodedTable::Data.unquoted()))
                })
                .ok()?;

            let Ok(value) = postcard::from_bytes::<Vec<CachedResource>>(&cached_resource_data)
            else {
                log::error!("Could not deserialize cached resource");
                return None;
            };
            std::sync::Arc::new(TokioRwLock::new(value))
        };
        {
            let v = deserialized_vec_cached_response.read().await;
            // The resources are more than one vector, so we stored them completely decoded.
            if v.len() == 1 {
                *v[0].body().lock() = ResponseBody::Done(decoded_data);
            }
        }

        // update times
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let inner = self.inner.lock().await;
        let (query, query_values) = Query::update()
            .table(DiskCacheEncodedTable::Table)
            .values([(DiskCacheEncodedTable::InsertionTimestamp, now.into())])
            .and_where(Expr::col(DiskCacheEncodedTable::Key).eq(key.as_ref()))
            .build_rusqlite(SqliteQueryBuilder);

        if inner
            .db
            .execute(query.as_str(), &*query_values.as_params())
            .is_err()
        {
            log::error!("Could not update timestamp in cache");
        }

        Some(deserialized_vec_cached_response)
    }

    /// Stores a [`CacheEntry`]` to disk.
    #[servo_tracing::instrument(skip(self))]
    pub(crate) async fn store(&self, key: CacheKey, entry: CacheEntry) {
        let entry = entry.read().await;
        if entry
            .iter()
            .any(|cached_resource| !cached_resource.is_done())
        {
            return;
        }

        let Ok(data) = postcard::to_stdvec(&*entry) else {
            error!("Could not deserialize value");
            return;
        };

        {
            let inner = self.inner.lock().await;
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
        }
        self.delete_until_cache_size().await;
    }

    /// Deletes data from the cache until the size is <= max_size
    #[servo_tracing::instrument(skip(self))]
    async fn delete_until_cache_size(&self) {
        let mut inner = self.inner.lock().await;
        let mut current_size: usize = inner.entries.values().map(|metadata| metadata.size).sum();
        if current_size <= self.max_size {
            return;
        }

        let sorted_by_data = inner
            .entries
            .iter()
            .map(|(key, metadata)| (metadata.last_accessed.unwrap(), key))
            .sorted_by_cached_key(|(last_accessed, _key)| *last_accessed)
            .map(|(_last_accessed, key)| key)
            .cloned()
            .collect::<Vec<_>>();

        for cache_key in sorted_by_data {
            if current_size <= self.max_size {
                return;
            }

            let (query, values) = Query::delete()
                .from_table(DiskCacheTable::Table)
                .and_where(Expr::col(DiskCacheTable::Key).eq(cache_key.as_ref()))
                .build_rusqlite(SqliteQueryBuilder);
            if inner
                .db
                .execute(query.as_str(), &*values.as_params())
                .is_err()
            {
                log::error!("Could not delete disk cache for {:?}", cache_key);
            }
            let (query, values) = Query::delete()
                .from_table(DiskCacheEncodedTable::Table)
                .and_where(Expr::col(DiskCacheEncodedTable::Key).eq(cache_key.as_ref()))
                .build_rusqlite(SqliteQueryBuilder);
            if inner
                .db
                .execute(query.as_str(), &*values.as_params())
                .is_err()
            {
                log::error!("Could not delete encoded disk cache for {:?}", cache_key);
            }

            let removed_entry = inner.entries.remove(&cache_key);
            current_size -= removed_entry.unwrap().size;
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

        let (query, params) = Query::delete()
            .from_table(DiskCacheEncodedTable::Table)
            .build_rusqlite(SqliteQueryBuilder);
        if inner
            .db
            .execute(query.as_str(), &*params.as_params())
            .is_err()
        {
            error!("Could not delete encoded part of disk cache");
        }

        inner.entries.clear();
    }
}
