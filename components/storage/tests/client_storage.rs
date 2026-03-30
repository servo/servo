/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use rusqlite::Connection;
use servo_base::generic_channel;
use servo_base::id::{BrowsingContextId, PipelineNamespace, PipelineNamespaceId, WebViewId};
use servo_url::ServoUrl;
use storage::ClientStorageThreadFactory;
use storage_traits::client_storage::{
    ClientStorageThreadHandle, ClientStorageThreadMessage, StorageIdentifier, StorageProxyMap,
    StorageType,
};

fn install_test_namespace() {
    PipelineNamespace::install(PipelineNamespaceId(1));
}

fn registry_db_path(tmp_dir: &tempfile::TempDir) -> PathBuf {
    tmp_dir.path().join("clientstorage").join("reg.sqlite")
}

fn open_registry(tmp_dir: &tempfile::TempDir) -> Connection {
    Connection::open(registry_db_path(tmp_dir)).expect("Registry database should exist")
}

fn obtain_bottle_map(
    handle: &ClientStorageThreadHandle,
    storage_type: StorageType,
    webview: WebViewId,
    storage_identifier: StorageIdentifier,
    origin: servo_url::ImmutableOrigin,
) -> StorageProxyMap {
    handle
        .obtain_a_storage_bottle_map(storage_type, webview, storage_identifier, origin)
        .recv()
        .unwrap()
        .unwrap()
}

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
    install_test_namespace();
    let tmp_dir = tempfile::tempdir().unwrap();
    let handle: ClientStorageThreadHandle =
        ClientStorageThreadFactory::new(Some(tmp_dir.path().to_path_buf()));

    let url = ServoUrl::parse("https://example.com").unwrap();

    // Obtain a first storage proxy map.
    let storage_proxy_map = handle
        .obtain_a_storage_bottle_map(
            StorageType::Local,
            WebViewId::new(servo_base::id::TEST_PAINTER_ID),
            StorageIdentifier::IndexedDB,
            url.origin(),
        )
        .recv()
        .unwrap()
        .unwrap();

    // Create a db.
    let receiver = handle.create_database(storage_proxy_map.bottle_id, "test1".to_string());
    let path = receiver.recv().unwrap().expect("Path should be created");

    assert!(std::fs::read_dir(path.clone()).is_ok());

    // Create another db with the same name.
    let receiver = handle.create_database(storage_proxy_map.bottle_id, "test1".to_string());
    assert!(receiver.recv().unwrap().is_err());

    // Create another db with a different same.
    let receiver = handle.create_database(storage_proxy_map.bottle_id, "test2".to_string());
    let yet_another_path = receiver.recv().unwrap().expect("Path should be created");
    assert_ne!(path, yet_another_path);

    // Delete the dbs.
    let receiver = handle.delete_database(storage_proxy_map.bottle_id, "test1".to_string());
    receiver.recv().unwrap().expect("Db should be deleted");
    let receiver = handle.delete_database(storage_proxy_map.bottle_id, "test2".to_string());
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

#[test]
fn test_repeated_local_obtain_reuses_same_logical_rows() {
    install_test_namespace();
    let tmp_dir = tempfile::tempdir().unwrap();
    let handle: ClientStorageThreadHandle =
        ClientStorageThreadFactory::new(Some(tmp_dir.path().to_path_buf()));

    let origin = ServoUrl::parse("https://example.com").unwrap().origin();
    let webview = WebViewId::new(servo_base::id::TEST_PAINTER_ID);

    let first = obtain_bottle_map(
        &handle,
        StorageType::Local,
        webview,
        StorageIdentifier::IndexedDB,
        origin.clone(),
    );
    let second = obtain_bottle_map(
        &handle,
        StorageType::Local,
        webview,
        StorageIdentifier::IndexedDB,
        origin.clone(),
    );

    assert_eq!(first.bottle_id, second.bottle_id);

    let registry = open_registry(&tmp_dir);
    let local_shed_count: i64 = registry
        .query_row(
            "SELECT COUNT(*) FROM sheds WHERE storage_type = 'local' AND browsing_context IS NULL;",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let shelf_count: i64 = registry
        .query_row(
            "SELECT COUNT(*) FROM shelves WHERE origin = ?1;",
            [origin.ascii_serialization()],
            |row| row.get(0),
        )
        .unwrap();
    let bucket_count: i64 = registry
        .query_row("SELECT COUNT(*) FROM buckets;", [], |row| row.get(0))
        .unwrap();
    let bottle_count: i64 = registry
        .query_row("SELECT COUNT(*) FROM bottles;", [], |row| row.get(0))
        .unwrap();

    assert_eq!(local_shed_count, 1);
    assert_eq!(shelf_count, 1);
    assert_eq!(bucket_count, 1);
    assert_eq!(bottle_count, 4);
}

#[test]
fn test_repeated_session_obtain_reuses_same_logical_rows() {
    install_test_namespace();
    let tmp_dir = tempfile::tempdir().unwrap();
    let handle: ClientStorageThreadHandle =
        ClientStorageThreadFactory::new(Some(tmp_dir.path().to_path_buf()));

    let origin = ServoUrl::parse("https://example.com").unwrap().origin();
    let webview = WebViewId::new(servo_base::id::TEST_PAINTER_ID);
    let browsing_context = Into::<BrowsingContextId>::into(webview).to_string();

    let first = obtain_bottle_map(
        &handle,
        StorageType::Session,
        webview,
        StorageIdentifier::SessionStorage,
        origin.clone(),
    );
    let second = obtain_bottle_map(
        &handle,
        StorageType::Session,
        webview,
        StorageIdentifier::SessionStorage,
        origin.clone(),
    );

    assert_eq!(first.bottle_id, second.bottle_id);

    let registry = open_registry(&tmp_dir);
    let session_shed_count: i64 = registry
        .query_row(
            "SELECT COUNT(*) FROM sheds WHERE storage_type = 'session' AND browsing_context = ?1;",
            [browsing_context],
            |row| row.get(0),
        )
        .unwrap();
    let shelf_count: i64 = registry
        .query_row(
            "SELECT COUNT(*) FROM shelves WHERE origin = ?1;",
            [origin.ascii_serialization()],
            |row| row.get(0),
        )
        .unwrap();
    let bucket_count: i64 = registry
        .query_row("SELECT COUNT(*) FROM buckets;", [], |row| row.get(0))
        .unwrap();
    let bottle_count: i64 = registry
        .query_row("SELECT COUNT(*) FROM bottles;", [], |row| row.get(0))
        .unwrap();

    assert_eq!(session_shed_count, 1);
    assert_eq!(shelf_count, 1);
    assert_eq!(bucket_count, 1);
    assert_eq!(bottle_count, 1);
}
