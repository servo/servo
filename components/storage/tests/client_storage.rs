/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel;
use servo_url::ServoUrl;
use storage::ClientStorageThreadFactory;
use storage::client_storage::ClientStorageThreadHandle;
use storage_traits::client_storage::{
    Bottle, BottleIdent, BucketIdent, ClientStorageThreadMessage,
};

#[test]
fn test_exit() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let handle: ClientStorageThreadHandle =
        ClientStorageThreadFactory::new(Some(tmp_dir.path().to_path_buf()));

    let (sender, receiver) = generic_channel::channel().unwrap();
    handle
        .send(ClientStorageThreadMessage::Exit(sender))
        .unwrap();
    receiver.recv().unwrap();

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}

#[test]
fn test_workflow() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let handle: ClientStorageThreadHandle =
        ClientStorageThreadFactory::new(Some(tmp_dir.path().to_path_buf()));

    // Create some storage
    let url = ServoUrl::parse("https://example.com").unwrap();
    let origin = url.origin();
    let recv = handle.create_bottle(
        origin.clone(),
        BucketIdent::Default,
        Bottle {
            bottle_type: BottleIdent::LocalStorage,
            quota: None,
        },
    );
    let path = recv.recv().unwrap().unwrap();
    // Try to create the same bottle again
    let recv = handle.create_bottle(
        origin.clone(),
        BucketIdent::Default,
        Bottle {
            bottle_type: BottleIdent::LocalStorage,
            quota: None,
        },
    );
    assert!(recv.recv().unwrap().is_err());
    // Try to open a non-existing bottle
    let recv = handle.open_bottle(
        origin.clone(),
        BucketIdent::Default,
        BottleIdent::IndexedDB("not_there".to_string()),
    );
    assert!(recv.recv().unwrap().is_err());
    // Open the existing bottle
    let recv = handle.open_bottle(
        origin.clone(),
        BucketIdent::Default,
        BottleIdent::LocalStorage,
    );
    assert_eq!(recv.recv().unwrap().unwrap(), path);

    handle.delete_all().recv().unwrap().unwrap();
    let (sender, receiver) = generic_channel::channel().unwrap();
    handle
        .send(ClientStorageThreadMessage::Exit(sender))
        .unwrap();
    receiver.recv().unwrap();

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
