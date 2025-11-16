/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use base::generic_channel::GenericSender;
#[cfg(not(feature = "indexeddb_next"))]
use ipc_channel::ipc::IpcSender;
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use storage_traits::StorageThreads;
use storage_traits::client_storage::ClientStorageThreadMsg;
#[cfg(not(feature = "indexeddb_next"))]
use storage_traits::indexeddb::IndexedDBThreadMsg;
use storage_traits::webstorage_thread::WebStorageThreadMsg;

#[cfg(not(feature = "indexeddb_next"))]
use crate::{ClientStorageThreadFactory, IndexedDBThreadFactory, WebStorageThreadFactory};
#[cfg(feature = "indexeddb_next")]
use crate::{ClientStorageThreadFactory, WebStorageThreadFactory};

#[cfg(not(feature = "indexeddb_next"))]
#[allow(clippy::too_many_arguments)]
pub fn new_storage_threads(
    mem_profiler_chan: MemProfilerChan,
    config_dir: Option<PathBuf>,
) -> (StorageThreads, StorageThreads) {
    let client_storage: GenericSender<ClientStorageThreadMsg> =
        ClientStorageThreadFactory::new(config_dir.clone());
    let idb: IpcSender<IndexedDBThreadMsg> = IndexedDBThreadFactory::new(config_dir.clone());
    let web_storage: GenericSender<WebStorageThreadMsg> =
        WebStorageThreadFactory::new(config_dir, mem_profiler_chan);
    (
        StorageThreads::new(client_storage.clone(), idb.clone(), web_storage.clone()),
        StorageThreads::new(client_storage, idb, web_storage),
    )
}

#[cfg(feature = "indexeddb_next")]
#[allow(clippy::too_many_arguments)]
pub fn new_storage_threads(
    mem_profiler_chan: MemProfilerChan,
    config_dir: Option<PathBuf>,
) -> (StorageThreads, StorageThreads) {
    let client_storage: GenericSender<ClientStorageThreadMsg> =
        ClientStorageThreadFactory::new(config_dir.clone());
    let web_storage: GenericSender<WebStorageThreadMsg> =
        WebStorageThreadFactory::new(config_dir, mem_profiler_chan);
    (
        StorageThreads::new(client_storage.clone(), web_storage.clone()),
        StorageThreads::new(client_storage, web_storage),
    )
}
