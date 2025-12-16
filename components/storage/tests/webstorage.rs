/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel as base_channel;
use base::generic_channel::GenericSend;
use base::id::TEST_WEBVIEW_ID;
use profile::mem as profile_mem;
use servo_url::ServoUrl;
use storage_traits::StorageThreads;
use storage_traits::webstorage_thread::{WebStorageThreadMsg, WebStorageType};
use tempfile::TempDir;

pub(crate) struct WebStorageTest {
    tmp_dir: Option<TempDir>,
    threads: StorageThreads,
}

impl WebStorageTest {
    pub(crate) fn new() -> Self {
        let tmp_dir = tempfile::tempdir().unwrap();
        let config_dir = tmp_dir.path().to_path_buf();
        let mem_profiler_chan = profile_mem::Profiler::create();
        let threads = storage::new_storage_threads(mem_profiler_chan, Some(config_dir));

        Self {
            tmp_dir: Some(tmp_dir),
            threads: threads.0,
        }
    }

    pub(crate) fn new_in_memory() -> Self {
        let mem_profiler_chan = profile_mem::Profiler::create();
        let threads = storage::new_storage_threads(mem_profiler_chan, None);

        Self {
            tmp_dir: None,
            threads: threads.0,
        }
    }

    pub(crate) fn restart(mut self) -> Self {
        let tmp_dir = self.tmp_dir.take();
        let config_dir = tmp_dir.as_ref().map(|d| d.path().to_path_buf());
        let mem_profiler_chan = profile_mem::Profiler::create();
        let threads = storage::new_storage_threads(mem_profiler_chan, config_dir);

        Self {
            tmp_dir: tmp_dir,
            threads: threads.0,
        }
    }

    pub(crate) fn threads(&self) -> StorageThreads {
        self.threads.clone()
    }

    pub(crate) fn length(&self, storage_type: WebStorageType, url: &ServoUrl) -> usize {
        let (sender, receiver) = base_channel::channel().unwrap();
        self.threads
            .send(WebStorageThreadMsg::Length(
                sender,
                storage_type,
                TEST_WEBVIEW_ID,
                url.clone(),
            ))
            .unwrap();
        receiver.recv().unwrap()
    }

    pub(crate) fn key(
        &self,
        storage_type: WebStorageType,
        url: &ServoUrl,
        index: u32,
    ) -> Option<String> {
        let (sender, receiver) = base_channel::channel().unwrap();
        self.threads
            .send(WebStorageThreadMsg::Key(
                sender,
                storage_type,
                TEST_WEBVIEW_ID,
                url.clone(),
                index,
            ))
            .unwrap();
        receiver.recv().unwrap()
    }

    pub(crate) fn keys(&self, storage_type: WebStorageType, url: &ServoUrl) -> Vec<String> {
        let (sender, receiver) = base_channel::channel().unwrap();
        self.threads
            .send(WebStorageThreadMsg::Keys(
                sender,
                storage_type,
                TEST_WEBVIEW_ID,
                url.clone(),
            ))
            .unwrap();
        receiver.recv().unwrap()
    }

    pub(crate) fn get_item(
        &self,
        storage_type: WebStorageType,
        url: &ServoUrl,
        key: &str,
    ) -> Option<String> {
        let (sender, receiver) = base_channel::channel().unwrap();
        self.threads
            .send(WebStorageThreadMsg::GetItem(
                sender,
                storage_type,
                TEST_WEBVIEW_ID,
                url.clone(),
                key.into(),
            ))
            .unwrap();
        receiver.recv().unwrap()
    }

    pub(crate) fn set_item(
        &self,
        storage_type: WebStorageType,
        url: &ServoUrl,
        key: &str,
        value: &str,
    ) -> Result<(bool, Option<String>), ()> {
        let (sender, receiver) = base_channel::channel().unwrap();
        self.threads
            .send(WebStorageThreadMsg::SetItem(
                sender,
                storage_type,
                TEST_WEBVIEW_ID,
                url.clone(),
                key.into(),
                value.into(),
            ))
            .unwrap();
        receiver.recv().unwrap()
    }

    pub(crate) fn remove_item(
        &self,
        storage_type: WebStorageType,
        url: &ServoUrl,
        key: &str,
    ) -> Option<String> {
        let (sender, receiver) = base_channel::channel().unwrap();
        self.threads
            .send(WebStorageThreadMsg::RemoveItem(
                sender,
                storage_type,
                TEST_WEBVIEW_ID,
                url.clone(),
                key.into(),
            ))
            .unwrap();
        receiver.recv().unwrap()
    }

    pub(crate) fn clear(&self, storage_type: WebStorageType, url: &ServoUrl) -> bool {
        let (sender, receiver) = base_channel::channel().unwrap();
        self.threads
            .send(WebStorageThreadMsg::Clear(
                sender,
                storage_type,
                TEST_WEBVIEW_ID,
                url.clone(),
            ))
            .unwrap();
        receiver.recv().unwrap()
    }

    /// Gracefully shut down the webstorage thread to avoid dangling threads in tests.
    fn shutdown(&self) {
        let (sender, receiver) = base_channel::channel().unwrap();
        self.threads
            .send(WebStorageThreadMsg::Exit(sender))
            .expect("failed to send Exit");
        // Wait for acknowledgement so the thread terminates before the test ends.
        let _ = receiver.recv();
    }
}

impl Drop for WebStorageTest {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[test]
fn set_and_get_item() {
    let test = WebStorageTest::new();
    let url = ServoUrl::parse("https://example.com").unwrap();

    // Set a value.
    let result = test.set_item(WebStorageType::Local, &url, "foo", "bar");
    assert_eq!(result, Ok((true, None)));

    // Retrieve the value.
    let result = test.get_item(WebStorageType::Local, &url, "foo");
    assert_eq!(result, Some("bar".into()));
}

#[test]
fn set_and_get_item_in_memory() {
    let test = WebStorageTest::new_in_memory();
    let url = ServoUrl::parse("https://example.com").unwrap();

    // Set a value.
    let result = test.set_item(WebStorageType::Local, &url, "foo", "bar");
    assert_eq!(result, Ok((true, None)));

    // Retrieve the value.
    let result = test.get_item(WebStorageType::Local, &url, "foo");
    assert_eq!(result, Some("bar".into()));
}

#[test]
fn length_key_and_keys() {
    let test = WebStorageTest::new();
    let url = ServoUrl::parse("https://example.com").unwrap();

    // Insert two items.
    for (k, v) in [("foo", "v1"), ("bar", "v2")] {
        let _ = test.set_item(WebStorageType::Local, &url, k, v);
    }

    // Verify length.
    let result = test.length(WebStorageType::Local, &url);
    assert_eq!(result, 2);

    // Verify key(0) returns one of the inserted keys.
    let result = test.key(WebStorageType::Local, &url, 0);
    let key0 = result.unwrap();
    assert!(key0 == "foo" || key0 == "bar");

    // Verify keys vector contains both keys.
    let result = test.keys(WebStorageType::Local, &url);
    assert_eq!(result.len(), 2);
    assert!(result.contains(&"foo".to_string()));
    assert!(result.contains(&"bar".to_string()));
}

#[test]
fn remove_item_and_clear() {
    let test = WebStorageTest::new();
    let url = ServoUrl::parse("https://example.com").unwrap();

    // Insert items.
    for (k, v) in [("foo", "v1"), ("bar", "v2")] {
        let _ = test.set_item(WebStorageType::Local, &url, k, v);
    }

    // Remove one item and verify old value is returned.
    let result = test.remove_item(WebStorageType::Local, &url, "foo");
    assert_eq!(result, Some("v1".into()));

    // Removing again should return None.
    let result = test.remove_item(WebStorageType::Local, &url, "foo");
    assert_eq!(result, None);

    // Clear storage and verify it reported change.
    let result = test.clear(WebStorageType::Local, &url);
    assert!(result);

    // Length should now be zero.
    let result = test.length(WebStorageType::Local, &url);
    assert_eq!(result, 0);
}

fn test_origin_descriptors(
    test: WebStorageTest,
    storage_type: WebStorageType,
    survives_restart: bool,
) {
    let threads = test.threads();
    let url = ServoUrl::parse("https://example.com").unwrap();

    // Set a value.
    let _ = test.set_item(storage_type, &url, "foo", "bar");

    // Verify descriptors.
    let descriptors = threads.webstorage_origins(storage_type);
    assert_eq!(descriptors.len(), 1);
    assert_eq!(descriptors[0].name, "https://example.com");

    // Restart storage threads.
    let test = test.restart();
    let threads = test.threads();

    // There should still be descriptors.
    let descriptors = threads.webstorage_origins(storage_type);
    if survives_restart {
        assert_eq!(descriptors.len(), 1);
        assert_eq!(descriptors[0].name, "https://example.com");
    } else {
        assert!(descriptors.is_empty());
    }
}

#[test]
fn origin_descriptors_session() {
    let test = WebStorageTest::new();
    test_origin_descriptors(
        test,
        WebStorageType::Session,
        /* survives_restart */ false,
    );
}

#[test]
fn origin_descriptors_local() {
    let test = WebStorageTest::new();
    test_origin_descriptors(
        test,
        WebStorageType::Local,
        /* survives_restart */ true,
    );
}

fn test_clear_data_for_sites(test: WebStorageTest, storage_type: WebStorageType) {
    let threads = test.threads();
    let url = ServoUrl::parse("https://example.com").unwrap();

    // Set a value.
    let _ = test.set_item(storage_type, &url, "foo", "bar");

    // Verify length.
    let result = test.length(storage_type, &url);
    assert_eq!(result, 1);

    // Verify descriptors.
    let descriptors = threads.webstorage_origins(storage_type);
    assert_eq!(descriptors.len(), 1);

    // Clear site.
    threads.clear_webstorage_for_sites(storage_type, &["example.com"]);

    // Length should now be zero.
    let result = test.length(storage_type, &url);
    assert_eq!(result, 0);

    // There should now be no descriptors.
    let descriptors = threads.webstorage_origins(storage_type);
    match storage_type {
        WebStorageType::Session => assert_eq!(descriptors.len(), 0),
        WebStorageType::Local =>
        // TODO: Fix localStorage to not create origin descriptors for
        // read only operations (the length check above).
        {
            assert_eq!(descriptors.len(), 1)
        },
    }

    // Restart storage threads.
    let test = test.restart();
    let threads = test.threads();

    // Length should still be zero.
    let result = test.length(storage_type, &url);
    assert_eq!(result, 0);

    // There should still be no descriptors.
    let descriptors = threads.webstorage_origins(storage_type);
    match storage_type {
        WebStorageType::Session => assert_eq!(descriptors.len(), 0),
        WebStorageType::Local =>
        // TODO: Fix localStorage to not create origin descriptors for
        // read only operations (the length check above).
        {
            assert_eq!(descriptors.len(), 1)
        },
    }

    // Set a different value.
    let _ = test.set_item(storage_type, &url, "foo2", "bar2");

    // Verify the original value doesn't exist.
    let result = test.get_item(storage_type, &url, "foo");
    assert_eq!(result, None);
}

#[test]
fn clear_data_for_sites_session() {
    let test = WebStorageTest::new();
    test_clear_data_for_sites(test, WebStorageType::Session);
}

#[test]
fn clear_data_for_sites_local() {
    let test = WebStorageTest::new();
    test_clear_data_for_sites(test, WebStorageType::Local);
}

#[test]
fn clear_data_for_sites_local_in_memory() {
    let test = WebStorageTest::new_in_memory();
    test_clear_data_for_sites(test, WebStorageType::Local);
}

#[test]
fn no_storage_type_conflict() {
    // Ensures that editing session storage does not affect local storage and vice versa.
    let (tmp_dir, threads) = init();
    let url = ServoUrl::parse("https://example.com").unwrap();
    // Set local storage item.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::SetItem(
            sender,
            StorageType::Local,
            TEST_WEBVIEW_ID,
            url.clone(),
            "key".into(),
            "local_value".into(),
        ))
        .unwrap();
    let _ = receiver.recv().unwrap();
    // Set session storage item.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::SetItem(
            sender,
            StorageType::Session,
            TEST_WEBVIEW_ID,
            url.clone(),
            "key".into(),
            "session_value".into(),
        ))
        .unwrap();
    let _ = receiver.recv().unwrap();
    // Shutdown threads to ensure data is cleared from session storage and local storage is loaded from disk
    shutdown(&threads);
    let threads = init_with(&tmp_dir);
    // Get local storage item.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::GetItem(
            sender,
            StorageType::Local,
            TEST_WEBVIEW_ID,
            url.clone(),
            "key".into(),
        ))
        .unwrap();
    assert_eq!(receiver.recv().unwrap(), Some("local_value".into()));
    shutdown(&threads);
    let threads = init_with(&tmp_dir);
    // Get session storage item.
    let (sender, receiver) = base_channel::channel().unwrap();
    threads
        .send(WebStorageThreadMsg::GetItem(
            sender,
            StorageType::Session,
            TEST_WEBVIEW_ID,
            url.clone(),
            "key".into(),
        ))
        .unwrap();
    assert_eq!(receiver.recv().unwrap(), None);
}
