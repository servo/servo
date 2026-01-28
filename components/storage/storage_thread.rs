/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use base::generic_channel::GenericSender;
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use storage_traits::StorageThreads;
use storage_traits::indexeddb::IndexedDBThreadMsg;
use storage_traits::webstorage_thread::WebStorageThreadMsg;

use crate::client_storage::ClientStorageThreadHandle;
use crate::{ClientStorageThreadFactory, IndexedDBThreadFactory, WebStorageThreadFactory};

pub fn new_storage_threads(
    mem_profiler_chan: MemProfilerChan,
    config_dir: Option<PathBuf>,
) -> (StorageThreads, StorageThreads) {
    let client_storage: ClientStorageThreadHandle =
        ClientStorageThreadFactory::new(config_dir.clone());
    let idb: GenericSender<IndexedDBThreadMsg> = IndexedDBThreadFactory::new(config_dir.clone());
    let web_storage: GenericSender<WebStorageThreadMsg> =
        WebStorageThreadFactory::new(config_dir, mem_profiler_chan);
    (
        StorageThreads::new(
            client_storage.clone().into(),
            idb.clone(),
            web_storage.clone(),
        ),
        StorageThreads::new(client_storage.into(), idb, web_storage),
    )
}
