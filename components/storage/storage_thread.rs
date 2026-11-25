/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use profile_traits::mem::ProfilerChan as MemProfilerChan;
use servo_base::generic_channel::GenericSender;
use storage_traits::StorageThreads;
use storage_traits::cache_storage::CacheStorageThreadHandle;
use storage_traits::client_storage::ClientStorageThreadHandle;
use storage_traits::indexeddb::IndexedDBThreadMsg;
use storage_traits::webstorage_thread::WebStorageThreadMsg;

use crate::{
    CacheStorageThreadFactory, ClientStorageThreadFactory, DataDirectory, IndexedDBThreadFactory,
    WebStorageThreadFactory,
};

enum StorageThreadType {
    Public,
    Private,
}

impl StorageThreadType {
    fn label(&self) -> &'static str {
        match self {
            StorageThreadType::Private => "private",
            StorageThreadType::Public => "public",
        }
    }
}

fn new_storage_thread_group(
    storage_thread_type: StorageThreadType,
    storage_directory: &DataDirectory,
    mem_profiler_chan: MemProfilerChan,
) -> StorageThreads {
    let label = storage_thread_type.label();
    // IndexedDB needs to write to disk even when private storage is activated, so the private
    // `DataDirectory` is passed here when creating a private storage thread group.
    let client_storage: ClientStorageThreadHandle =
        ClientStorageThreadFactory::new(storage_directory);
    let idb: GenericSender<IndexedDBThreadMsg> = IndexedDBThreadFactory::new(
        mem_profiler_chan.clone(),
        format!("indexedDB-reporter-{label}"),
    );

    let storage_dir = match storage_thread_type {
        StorageThreadType::Public => Some(storage_directory.path()),
        StorageThreadType::Private => None,
    };
    let web_storage: GenericSender<WebStorageThreadMsg> = WebStorageThreadFactory::new(
        storage_dir,
        mem_profiler_chan,
        format!("storage-reporter-{label}"),
    );
    let cache_storage: CacheStorageThreadHandle = CacheStorageThreadFactory::new();

    StorageThreads::new(
        client_storage.into(),
        idb,
        web_storage,
        cache_storage.into(),
    )
}

pub fn new_storage_threads(
    mem_profiler_chan: MemProfilerChan,
    storage_directory: &DataDirectory,
    private_storage_directory: &DataDirectory,
) -> (StorageThreads, StorageThreads) {
    let private_storage_threads = new_storage_thread_group(
        StorageThreadType::Private,
        private_storage_directory,
        mem_profiler_chan.clone(),
    );
    let public_storage_threads = new_storage_thread_group(
        StorageThreadType::Public,
        storage_directory,
        mem_profiler_chan,
    );

    (private_storage_threads, public_storage_threads)
}
