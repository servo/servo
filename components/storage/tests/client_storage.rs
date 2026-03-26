/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo_base::generic_channel;
use servo_base::id::{PipelineNamespace, PipelineNamespaceId, WebViewId};
use servo_url::ServoUrl;
use storage::ClientStorageThreadFactory;
use storage_traits::client_storage::{
    ClientStorageThreadHandle, ClientStorageThreadMessage, StorageIdentifier, StorageType,
};

#[test]
fn test_exit() {
    let handle: ClientStorageThreadHandle = ClientStorageThreadFactory::new(None);

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
    PipelineNamespace::install(PipelineNamespaceId(1));
    let handle: ClientStorageThreadHandle = ClientStorageThreadFactory::new(None);

    let url = ServoUrl::parse("https://example.com").unwrap();

    // Obtain a first storage proxy map.
    let storage_proxy_map = handle.obtain_a_storage_bottle_map(
        StorageType::Local,
        WebViewId::new(servo_base::id::TEST_PAINTER_ID),
        StorageIdentifier::IndexedDB,
        url.origin(),
    ).recv().unwrap().unwrap();

    // Create a db.
    let receiver = handle.create_database(storage_proxy_map.bottle_id, "test1".to_string());
    let path = receiver.recv().unwrap().expect("Path should be created");

    assert!(std::fs::read_dir(path.clone()).is_ok());

    // Delete the db.
    let receiver = handle.delete_database(storage_proxy_map.bottle_id, "test1".to_string());
    receiver.recv().unwrap().expect("Db should be deleted");

    assert!(std::fs::read_dir(path).is_err());

    // Get another proxy map fro the same parameters.
    let second_proxy_map = handle
        .obtain_a_storage_bottle_map(
            StorageType::Local,
            WebViewId::new(servo_base::id::TEST_PAINTER_ID),
            StorageIdentifier::IndexedDB,
            url.origin(),
        )
        .recv()
        .unwrap()
        .unwrap();

    // Bottle id should be the same, because the manager returned the existing id.
    assert_eq!(second_proxy_map.bottle_id, storage_proxy_map.bottle_id);

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
