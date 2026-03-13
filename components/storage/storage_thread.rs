/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use base::generic_channel::GenericSender;
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use storage_traits::StorageThreads;
use storage_traits::client_storage::ClientStorageThreadMessage;
use storage_traits::indexeddb::IndexedDBThreadMsg;
use storage_traits::webstorage_thread::WebStorageThreadMsg;

use crate::{ClientStorageThreadFactory, IndexedDBThreadFactory, WebStorageThreadFactory};

fn new_storage_thread_group(
    mem_profiler_chan: MemProfilerChan,
    config_dir: Option<PathBuf>,
    label: &str,
) -> StorageThreads {
    let client_storage: GenericSender<ClientStorageThreadMessage> =
        ClientStorageThreadFactory::new(config_dir.clone());
    let idb: GenericSender<IndexedDBThreadMsg> = IndexedDBThreadFactory::new(
        config_dir.clone(),
        mem_profiler_chan.clone(),
        format!("indexedDB-reporter-{label}"),
    );
    let web_storage: GenericSender<WebStorageThreadMsg> = WebStorageThreadFactory::new(
        config_dir,
        mem_profiler_chan,
        format!("storage-reporter-{label}"),
    );

    StorageThreads::new(client_storage, idb, web_storage)
}

pub fn new_storage_threads(
    mem_profiler_chan: MemProfilerChan,
    config_dir: Option<PathBuf>,
) -> (StorageThreads, StorageThreads) {
    let private_storage_threads =
        new_storage_thread_group(mem_profiler_chan.clone(), config_dir.clone(), "private");
    let public_storage_threads = new_storage_thread_group(mem_profiler_chan, config_dir, "public");

    (private_storage_threads, public_storage_threads)
}
