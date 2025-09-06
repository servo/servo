/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::VecDeque;
use std::sync::Arc;

use net::indexeddb::engines::{KvsEngine, KvsOperation, KvsTransaction, SqliteEngine};
use net::indexeddb::idb_thread::IndexedDBDescription;
use net::resource_thread::CoreResourceThreadPool;
use net_traits::indexeddb_thread::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, CreateObjectResult,
    IndexedDBKeyRange, IndexedDBKeyType, IndexedDBTxnMode, KeyPath, PutItemResult,
};
use serde::{Deserialize, Serialize};
use servo_url::ImmutableOrigin;
use url::Host;

fn test_origin() -> ImmutableOrigin {
    ImmutableOrigin::Tuple(
        "test_origin".to_string(),
        Host::Domain("localhost".to_string()),
        80,
    )
}

fn get_pool() -> Arc<CoreResourceThreadPool> {
    Arc::new(CoreResourceThreadPool::new(1, "test".to_string()))
}

#[test]
fn test_cycle() {
    let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let thread_pool = get_pool();
    // Test create
    let _ = SqliteEngine::new(
        base_dir.path(),
        &IndexedDBDescription {
            name: "test_db".to_string(),
            origin: test_origin(),
        },
        thread_pool.clone(),
    )
    .unwrap();
    // Test open
    let db = SqliteEngine::new(
        base_dir.path(),
        &IndexedDBDescription {
            name: "test_db".to_string(),
            origin: test_origin(),
        },
        thread_pool.clone(),
    )
    .unwrap();
    let version = db.version().expect("Failed to get version");
    assert_eq!(version, 0);
    db.set_version(5).unwrap();
    let new_version = db.version().expect("Failed to get new version");
    assert_eq!(new_version, 5);
    db.delete_database().expect("Failed to delete database");
}

#[test]
fn test_create_store() {
    let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let thread_pool = get_pool();
    let db = SqliteEngine::new(
        base_dir.path(),
        &IndexedDBDescription {
            name: "test_db".to_string(),
            origin: test_origin(),
        },
        thread_pool,
    )
    .unwrap();
    let store_name = "test_store";
    let result = db.create_store(store_name, None, true);
    assert!(result.is_ok());
    let create_result = result.unwrap();
    assert_eq!(create_result, CreateObjectResult::Created);
    // Try to create the same store again
    let result = db.create_store(store_name, None, false);
    assert!(result.is_ok());
    let create_result = result.unwrap();
    assert_eq!(create_result, CreateObjectResult::AlreadyExists);
    // Ensure store was not overwritten
    assert!(db.has_key_generator(store_name));
}

#[test]
fn test_create_store_empty_name() {
    let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let thread_pool = get_pool();
    let db = SqliteEngine::new(
        base_dir.path(),
        &IndexedDBDescription {
            name: "test_db".to_string(),
            origin: test_origin(),
        },
        thread_pool,
    )
    .unwrap();
    let store_name = "";
    let result = db.create_store(store_name, None, true);
    assert!(result.is_ok());
    let create_result = result.unwrap();
    assert_eq!(create_result, CreateObjectResult::Created);
}

#[test]
fn test_injection() {
    let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let thread_pool = get_pool();
    let db = SqliteEngine::new(
        base_dir.path(),
        &IndexedDBDescription {
            name: "test_db".to_string(),
            origin: test_origin(),
        },
        thread_pool,
    )
    .unwrap();
    // Create a normal store
    let store_name1 = "test_store";
    let result = db.create_store(store_name1, None, true);
    assert!(result.is_ok());
    let create_result = result.unwrap();
    assert_eq!(create_result, CreateObjectResult::Created);
    // Injection
    let store_name2 = "' OR 1=1 -- -";
    let result = db.create_store(store_name2, None, false);
    assert!(result.is_ok());
    let create_result = result.unwrap();
    assert_eq!(create_result, CreateObjectResult::Created);
}

#[test]
fn test_key_path() {
    let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let thread_pool = get_pool();
    let db = SqliteEngine::new(
        base_dir.path(),
        &IndexedDBDescription {
            name: "test_db".to_string(),
            origin: test_origin(),
        },
        thread_pool,
    )
    .unwrap();
    let store_name = "test_store";
    let result = db.create_store(store_name, Some(KeyPath::String("test".to_string())), true);
    assert!(result.is_ok());
    assert_eq!(
        db.key_path(store_name),
        Some(KeyPath::String("test".to_string()))
    );
}

#[test]
fn test_delete_store() {
    let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let thread_pool = get_pool();
    let db = SqliteEngine::new(
        base_dir.path(),
        &IndexedDBDescription {
            name: "test_db".to_string(),
            origin: test_origin(),
        },
        thread_pool,
    )
    .unwrap();
    db.create_store("test_store", None, false)
        .expect("Failed to create store");
    // Delete the store
    db.delete_store("test_store")
        .expect("Failed to delete store");
    // Try to delete the same store again
    let result = db.delete_store("test_store");
    assert!(result.is_err());
    // Try to delete a non-existing store
    let result = db.delete_store("test_store");
    // Should work as per spec
    assert!(result.is_err());
}

#[test]
fn test_async_operations() {
    fn get_channel<T>() -> (
        ipc_channel::ipc::IpcSender<T>,
        ipc_channel::ipc::IpcReceiver<T>,
    )
    where
        T: for<'de> Deserialize<'de> + Serialize,
    {
        ipc_channel::ipc::channel().unwrap()
    }

    let base_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let thread_pool = get_pool();
    let db = SqliteEngine::new(
        base_dir.path(),
        &IndexedDBDescription {
            name: "test_db".to_string(),
            origin: test_origin(),
        },
        thread_pool,
    )
    .unwrap();
    let store_name = "test_store";
    db.create_store(store_name, None, false)
        .expect("Failed to create store");
    let put = get_channel();
    let put2 = get_channel();
    let put3 = get_channel();
    let put_dup = get_channel();
    let get_item_some = get_channel();
    let get_item_none = get_channel();
    let get_all_items = get_channel();
    let count = get_channel();
    let remove = get_channel();
    let clear = get_channel();
    let rx = db.process_transaction(KvsTransaction {
        mode: IndexedDBTxnMode::Readwrite,
        requests: VecDeque::from(vec![
            KvsOperation {
                store_name: store_name.to_owned(),
                operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                    sender: put.0,
                    key: Some(IndexedDBKeyType::Number(1.0)),
                    value: vec![1, 2, 3],
                    should_overwrite: false,
                }),
            },
            KvsOperation {
                store_name: store_name.to_owned(),
                operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                    sender: put2.0,
                    key: Some(IndexedDBKeyType::String("2.0".to_string())),
                    value: vec![4, 5, 6],
                    should_overwrite: false,
                }),
            },
            KvsOperation {
                store_name: store_name.to_owned(),
                operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                    sender: put3.0,
                    key: Some(IndexedDBKeyType::Array(vec![
                        IndexedDBKeyType::String("3".to_string()),
                        IndexedDBKeyType::Number(0.0),
                    ])),
                    value: vec![7, 8, 9],
                    should_overwrite: false,
                }),
            },
            // Try to put a duplicate key without overwrite
            KvsOperation {
                store_name: store_name.to_owned(),
                operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                    sender: put_dup.0,
                    key: Some(IndexedDBKeyType::Number(1.0)),
                    value: vec![10, 11, 12],
                    should_overwrite: false,
                }),
            },
            KvsOperation {
                store_name: store_name.to_owned(),
                operation: AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem {
                    sender: get_item_some.0,
                    key_range: IndexedDBKeyRange::only(IndexedDBKeyType::Number(1.0)),
                }),
            },
            KvsOperation {
                store_name: store_name.to_owned(),
                operation: AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem {
                    sender: get_item_none.0,
                    key_range: IndexedDBKeyRange::only(IndexedDBKeyType::Number(5.0)),
                }),
            },
            KvsOperation {
                store_name: store_name.to_owned(),
                operation: AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetAllItems {
                    sender: get_all_items.0,
                    key_range: IndexedDBKeyRange::lower_bound(IndexedDBKeyType::Number(0.0), false),
                    count: None,
                }),
            },
            KvsOperation {
                store_name: store_name.to_owned(),
                operation: AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count {
                    sender: count.0,
                    key_range: IndexedDBKeyRange::only(IndexedDBKeyType::Number(1.0)),
                }),
            },
            KvsOperation {
                store_name: store_name.to_owned(),
                operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem {
                    sender: remove.0,
                    key: IndexedDBKeyType::Number(1.0),
                }),
            },
            KvsOperation {
                store_name: store_name.to_owned(),
                operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::Clear(clear.0)),
            },
        ]),
    });
    let _ = rx.blocking_recv().unwrap();
    put.1.recv().unwrap().unwrap();
    put2.1.recv().unwrap().unwrap();
    put3.1.recv().unwrap().unwrap();
    let err = put_dup.1.recv().unwrap().unwrap();
    assert_eq!(err, PutItemResult::CannotOverwrite);
    let get_result = get_item_some.1.recv().unwrap();
    let value = get_result.unwrap();
    assert_eq!(value, Some(vec![1, 2, 3]));
    let get_result = get_item_none.1.recv().unwrap();
    let value = get_result.unwrap();
    assert_eq!(value, None);
    let all_items = get_all_items.1.recv().unwrap().unwrap();
    assert_eq!(all_items.len(), 3);
    // Check that all three items are present
    assert!(all_items.contains(&vec![1, 2, 3]));
    assert!(all_items.contains(&vec![4, 5, 6]));
    assert!(all_items.contains(&vec![7, 8, 9]));
    let amount = count.1.recv().unwrap().unwrap();
    assert_eq!(amount, 1);
    remove.1.recv().unwrap().unwrap();
    clear.1.recv().unwrap().unwrap();
}
