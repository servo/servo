/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use profile::mem as profile_mem;
use servo_base::generic_channel::{self, GenericSend};
use storage_traits::StorageThreads;
use storage_traits::client_storage::ClientStorageThreadMessage;
use storage_traits::indexeddb::{IndexedDBThreadMsg, SyncOperation};
use storage_traits::webstorage_thread::WebStorageThreadMsg;

fn shutdown_storage_group(threads: &StorageThreads) {
    let (client_sender, client_receiver) = generic_channel::channel().unwrap();
    GenericSend::send(threads, ClientStorageThreadMessage::Exit(client_sender))
        .expect("failed to send client storage exit");
    client_receiver
        .recv()
        .expect("failed to receive client storage exit ack");

    let (idb_sender, idb_receiver) = generic_channel::channel().unwrap();
    GenericSend::send(
        threads,
        IndexedDBThreadMsg::Sync(SyncOperation::Exit(idb_sender)),
    )
    .expect("failed to send indexeddb exit");
    idb_receiver
        .recv()
        .expect("failed to receive indexeddb exit ack");

    let (web_storage_sender, web_storage_receiver) = generic_channel::channel().unwrap();
    GenericSend::send(threads, WebStorageThreadMsg::Exit(web_storage_sender))
        .expect("failed to send web storage exit");
    web_storage_receiver
        .recv()
        .expect("failed to receive web storage exit ack");
}

#[test]
fn test_new_storage_threads_create_independent_groups() {
    let mem_profiler_chan = profile_mem::Profiler::create();
    let (private_storage_threads, public_storage_threads) =
        storage::new_storage_threads(mem_profiler_chan, None);

    shutdown_storage_group(&private_storage_threads);
    shutdown_storage_group(&public_storage_threads);

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
