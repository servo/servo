/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel as base_channel;
use base::generic_channel::GenericSend;
use base::id::TEST_WEBVIEW_ID;
use profile::mem as profile_mem;
use servo_url::ServoUrl;
use storage_traits::StorageThreads;
use storage_traits::webstorage_thread::{StorageType, WebStorageThreadMsg};
use tempfile::TempDir;

fn init() -> (TempDir, StorageThreads) {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().to_path_buf();
    let mem_profiler_chan = profile_mem::Profiler::create();
    let threads = storage::new_storage_threads(mem_profiler_chan, Some(config_dir));
    (dir, threads.0)
}

fn init_with(dir: &tempfile::TempDir) -> StorageThreads {
    let config_dir = dir.path().to_path_buf();
    let mem_profiler_chan = profile_mem::Profiler::create();
    let threads = storage::new_storage_threads(mem_profiler_chan, Some(config_dir));
    threads.0
}

/// Gracefully shut down the webstorage thread to avoid dangling threads in tests.
fn shutdown(threads: &StorageThreads) {
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::Exit(sender))
        .expect("failed to send Exit");
    // Wait for acknowledgement so the thread terminates before the test ends.
    let _ = receiver.recv();
}

#[test]
fn set_and_get_item() {
    let (_tmp_dir, threads) = init();
    let url = ServoUrl::parse("https://example.com").unwrap();

    // Set a value.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::SetItem(
            sender,
            StorageType::Local,
            TEST_WEBVIEW_ID,
            url.clone(),
            "foo".into(),
            "bar".into(),
        ))
        .unwrap();
    assert_eq!(receiver.recv().unwrap(), Ok((true, None)));

    // Retrieve the value.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::GetItem(
            sender,
            StorageType::Local,
            TEST_WEBVIEW_ID,
            url.clone(),
            "foo".into(),
        ))
        .unwrap();
    assert_eq!(receiver.recv().unwrap(), Some("bar".into()));

    shutdown(&threads);
}

#[test]
fn length_key_and_keys() {
    let (_tmp_dir, threads) = init();
    let url = ServoUrl::parse("https://example.com").unwrap();

    // Insert two items.
    for (k, v) in [("foo", "v1"), ("bar", "v2")] {
        let (sender, receiver) = base_channel::channel().unwrap();
        threads
            .send(WebStorageThreadMsg::SetItem(
                sender,
                StorageType::Local,
                TEST_WEBVIEW_ID,
                url.clone(),
                k.into(),
                v.into(),
            ))
            .unwrap();
        let _ = receiver.recv().unwrap();
    }

    // Verify length.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::Length(
            sender,
            StorageType::Local,
            TEST_WEBVIEW_ID,
            url.clone(),
        ))
        .unwrap();
    assert_eq!(receiver.recv().unwrap(), 2);

    // Verify key(0) returns one of the inserted keys.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::Key(
            sender,
            StorageType::Local,
            TEST_WEBVIEW_ID,
            url.clone(),
            0,
        ))
        .unwrap();
    let key0 = receiver.recv().unwrap().unwrap();
    assert!(key0 == "foo" || key0 == "bar");

    // Verify keys vector contains both keys.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::Keys(
            sender,
            StorageType::Local,
            TEST_WEBVIEW_ID,
            url.clone(),
        ))
        .unwrap();
    let keys = receiver.recv().unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"foo".to_string()));
    assert!(keys.contains(&"bar".to_string()));

    shutdown(&threads);
}

#[test]
fn remove_item_and_clear() {
    let (_tmp_dir, threads) = init();
    let url = ServoUrl::parse("https://example.com").unwrap();

    // Insert items.
    for (k, v) in [("foo", "v1"), ("bar", "v2")] {
        let (sender, receiver) = base_channel::channel().unwrap();
        threads
            .send(WebStorageThreadMsg::SetItem(
                sender,
                StorageType::Local,
                TEST_WEBVIEW_ID,
                url.clone(),
                k.into(),
                v.into(),
            ))
            .unwrap();
        let _ = receiver.recv().unwrap();
    }

    // Remove one item and verify old value is returned.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::RemoveItem(
            sender,
            StorageType::Local,
            TEST_WEBVIEW_ID,
            url.clone(),
            "foo".into(),
        ))
        .unwrap();
    assert_eq!(receiver.recv().unwrap(), Some("v1".into()));

    // Removing again should return None.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::RemoveItem(
            sender,
            StorageType::Local,
            TEST_WEBVIEW_ID,
            url.clone(),
            "foo".into(),
        ))
        .unwrap();
    assert_eq!(receiver.recv().unwrap(), None);

    // Clear storage and verify it reported change.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::Clear(
            sender,
            StorageType::Local,
            TEST_WEBVIEW_ID,
            url.clone(),
        ))
        .unwrap();
    assert!(receiver.recv().unwrap());

    // Length should now be zero.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::Length(
            sender,
            StorageType::Local,
            TEST_WEBVIEW_ID,
            url.clone(),
        ))
        .unwrap();
    assert_eq!(receiver.recv().unwrap(), 0);

    shutdown(&threads);
}

#[test]
fn get_origin_descriptors() {
    let url = ServoUrl::parse("https://example.com").unwrap();

    // (storage_type, survives_restart)
    let cases = [(StorageType::Session, false), (StorageType::Local, true)];

    let (tmp_dir, threads) = init();

    for (storage_type, _) in cases {
        // Set a value.
        let (sender, receiver) = base_channel::channel().unwrap();
        threads
            .send(WebStorageThreadMsg::SetItem(
                sender,
                storage_type,
                TEST_WEBVIEW_ID,
                url.clone(),
                "foo".into(),
                "bar".into(),
            ))
            .unwrap();
        assert_eq!(receiver.recv().unwrap(), Ok((true, None)));

        // Get origin descriptors.
        let (sender, receiver) = base_channel::channel().unwrap();
        threads
            .send(WebStorageThreadMsg::GetOriginDescriptors(
                sender,
                storage_type,
            ))
            .unwrap();

        let descriptors = receiver.recv().unwrap();
        assert_eq!(descriptors.len(), 1);
        assert_eq!(descriptors[0].name, "https://example.com");
    }

    // Restart storage threads.
    shutdown(&threads);
    let threads = init_with(&tmp_dir);

    for (storage_type, survives_restart) in cases {
        // Get origin descriptors.
        let (sender, receiver) = base_channel::channel().unwrap();
        threads
            .send(WebStorageThreadMsg::GetOriginDescriptors(
                sender,
                storage_type,
            ))
            .unwrap();

        let descriptors = receiver.recv().unwrap();

        if survives_restart {
            assert_eq!(descriptors.len(), 1);
            assert_eq!(descriptors[0].name, "https://example.com");
        } else {
            assert!(descriptors.is_empty());
        }
    }

    shutdown(&threads);
}
