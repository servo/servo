/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use base::generic_channel::GenericSender;
use ipc_channel::ipc::IpcSender;
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use storage_traits::StorageThreads;
use storage_traits::indexeddb_thread::IndexedDBThreadMsg;
use storage_traits::webstorage_thread::WebStorageThreadMsg;

use crate::indexeddb::IndexedDBThreadFactory;
use crate::webstorage::WebStorageThreadFactory;

#[allow(clippy::too_many_arguments)]
pub fn new_storage_threads(
    mem_profiler_chan: MemProfilerChan,
    config_dir: Option<PathBuf>,
) -> (StorageThreads, StorageThreads) {
    let idb: IpcSender<IndexedDBThreadMsg> = IndexedDBThreadFactory::new(config_dir.clone());
    let storage: GenericSender<WebStorageThreadMsg> =
        WebStorageThreadFactory::new(config_dir, mem_profiler_chan);
    (
        StorageThreads::new(storage.clone(), idb.clone()),
        StorageThreads::new(storage, idb),
    )
}
