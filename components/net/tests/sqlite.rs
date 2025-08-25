/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::VecDeque;
use std::sync::Arc;

use net::indexeddb::engines::{
    KvsEngine, KvsOperation, KvsTransaction, SanitizedName, SqliteEngine,
};
use net::indexeddb::idb_thread::IndexedDBDescription;
use net::resource_thread::CoreResourceThreadPool;
use net_traits::indexeddb_thread::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, CreateObjectResult,
    IndexedDBKeyRange, IndexedDBKeyType, IndexedDBTxnMode, KeyPath,
};
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
    let store_name = SanitizedName::new("test_store".to_string());
    let result = db.create_store(store_name.clone(), None, true);
    assert!(result.is_ok());
    let create_result = result.unwrap();
    assert_eq!(create_result, CreateObjectResult::Created);
    // Try to create the same store again
    let result = db.create_store(store_name.clone(), None, false);
    assert!(result.is_ok());
    let create_result = result.unwrap();
    assert_eq!(create_result, CreateObjectResult::AlreadyExists);
    // Ensure store was not overwritten
    assert!(db.has_key_generator(store_name));
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
    let store_name = SanitizedName::new("test_store".to_string());
    let result = db.create_store(
        store_name.clone(),
        Some(KeyPath::String("test".to_string())),
        true,
    );
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
    db.create_store(SanitizedName::new("test_store".to_string()), None, false)
        .expect("Failed to create store");
    // Delete the store
    db.delete_store(SanitizedName::new("test_store".to_string()))
        .expect("Failed to delete store");
    // Try to delete the same store again
    let result = db.delete_store(SanitizedName::new("test_store".into()));
    assert!(result.is_err());
    // Try to delete a non-existing store
    let result = db.delete_store(SanitizedName::new("test_store".into()));
    // Should work as per spec
    assert!(result.is_err());
}

#[test]
fn test_async_operations() {
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
    let store_name = SanitizedName::new("test_store".to_string());
    db.create_store(store_name.clone(), None, false)
        .expect("Failed to create store");
    let channel = ipc_channel::ipc::channel().unwrap();
    let channel2 = ipc_channel::ipc::channel().unwrap();
    let channel3 = ipc_channel::ipc::channel().unwrap();
    let channel4 = ipc_channel::ipc::channel().unwrap();
    let channel5 = ipc_channel::ipc::channel().unwrap();
    let channel6 = ipc_channel::ipc::channel().unwrap();
    let rx = db.process_transaction(KvsTransaction {
        mode: IndexedDBTxnMode::Readwrite,
        requests: VecDeque::from(vec![
            KvsOperation {
                store_name: store_name.clone(),
                operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                    sender: channel.0,
                    key: IndexedDBKeyType::Number(1.0),
                    value: vec![1, 2, 3],
                    should_overwrite: false,
                }),
            },
            KvsOperation {
                store_name: store_name.clone(),
                operation: AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem {
                    sender: channel2.0,
                    key_range: IndexedDBKeyRange::only(IndexedDBKeyType::Number(1.0)),
                }),
            },
            KvsOperation {
                store_name: store_name.clone(),
                operation: AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem {
                    sender: channel3.0,
                    key_range: IndexedDBKeyRange::only(IndexedDBKeyType::Number(5.0)),
                }),
            },
            KvsOperation {
                store_name: store_name.clone(),
                operation: AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count {
                    sender: channel4.0,
                    key_range: IndexedDBKeyRange::only(IndexedDBKeyType::Number(1.0)),
                }),
            },
            KvsOperation {
                store_name: store_name.clone(),
                operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem {
                    sender: channel5.0,
                    key: IndexedDBKeyType::Number(1.0),
                }),
            },
            KvsOperation {
                store_name: store_name.clone(),
                operation: AsyncOperation::ReadWrite(AsyncReadWriteOperation::Clear(channel6.0)),
            },
        ]),
    });
    let _ = rx.blocking_recv().unwrap();
    channel.1.recv().unwrap().unwrap();
    let get_result = channel2.1.recv().unwrap();
    let value = get_result.unwrap();
    assert_eq!(value, Some(vec![1, 2, 3]));
    let get_result = channel3.1.recv().unwrap();
    let value = get_result.unwrap();
    assert_eq!(value, None);
    let amount = channel4.1.recv().unwrap().unwrap();
    assert_eq!(amount, 1);
    channel5.1.recv().unwrap().unwrap();
    channel6.1.recv().unwrap().unwrap();
}
