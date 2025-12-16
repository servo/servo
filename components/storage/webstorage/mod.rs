/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod engines;

use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use base::generic_channel::{self, GenericReceiver, GenericSender};
use base::id::WebViewId;
use base::threadpool::ThreadPool;
use base::{read_json_from_file, write_json_to_file};
use malloc_size_of::MallocSizeOf;
use malloc_size_of_derive::MallocSizeOf;
use profile_traits::mem::{
    ProcessReports, ProfilerChan as MemProfilerChan, Report, ReportKind, perform_memory_report,
};
use profile_traits::path;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use servo_config::pref;
use servo_url::{ImmutableOrigin, ServoUrl};
use storage_traits::webstorage_thread::{OriginDescriptor, StorageType, WebStorageThreadMsg};
use uuid::Uuid;

use crate::webstorage::engines::WebStorageEngine;
use crate::webstorage::engines::sqlite::SqliteEngine;

const QUOTA_SIZE_LIMIT: usize = 5 * 1024 * 1024;

pub trait WebStorageThreadFactory {
    fn new(config_dir: Option<PathBuf>, mem_profiler_chan: MemProfilerChan) -> Self;
}

impl WebStorageThreadFactory for GenericSender<WebStorageThreadMsg> {
    /// Create a storage thread
    fn new(
        config_dir: Option<PathBuf>,
        mem_profiler_chan: MemProfilerChan,
    ) -> GenericSender<WebStorageThreadMsg> {
        let (chan, port) = generic_channel::channel().unwrap();
        let chan2 = chan.clone();
        thread::Builder::new()
            .name("WebStorageManager".to_owned())
            .spawn(move || {
                mem_profiler_chan.run_with_memory_reporting(
                    || WebStorageManager::new(port, config_dir).start(),
                    String::from("storage-reporter"),
                    chan2,
                    WebStorageThreadMsg::CollectMemoryReport,
                );
            })
            .expect("Thread spawning failed");
        chan
    }
}

#[derive(Deserialize, Serialize)]
pub struct StorageOrigins {
    origin_descriptors: FxHashMap<String, OriginDescriptor>,
}

impl StorageOrigins {
    fn new() -> Self {
        StorageOrigins {
            origin_descriptors: FxHashMap::default(),
        }
    }

    /// Ensures that an origin descriptor exists for the given origin.
    ///
    /// Returns `true` if a new origin descriptor was created, or `false` if
    /// one already existed.
    fn ensure_origin_descriptor(&mut self, origin: &ImmutableOrigin) -> bool {
        let origin = origin.ascii_serialization();
        match self.origin_descriptors.entry(origin.clone()) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(OriginDescriptor::new(origin));
                true
            },
        }
    }

    fn origin_descriptors(&self) -> Vec<OriginDescriptor> {
        self.origin_descriptors.values().cloned().collect()
    }
}

#[derive(Clone, Default, MallocSizeOf)]
pub struct OriginEntry {
    tree: BTreeMap<String, String>,
    size: usize,
}

impl OriginEntry {
    pub fn inner(&self) -> &BTreeMap<String, String> {
        &self.tree
    }

    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        let old_value = self.tree.insert(key.clone(), value.clone());
        let size_change = match &old_value {
            Some(old) => value.len() as isize - old.len() as isize,
            None => (key.len() + value.len()) as isize,
        };
        self.size = (self.size as isize + size_change) as usize;
        old_value
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        let old_value = self.tree.remove(key);
        if let Some(old) = &old_value {
            self.size -= key.len() + old.len();
        }
        old_value
    }

    pub fn clear(&mut self) {
        self.tree.clear();
        self.size = 0;
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

struct WebStorageEnvironment<E: WebStorageEngine> {
    engine: E,
    data: OriginEntry,
}

impl<E: WebStorageEngine> MallocSizeOf for WebStorageEnvironment<E> {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        self.data.size_of(ops)
    }
}

impl<E: WebStorageEngine> WebStorageEnvironment<E> {
    fn new(engine: E) -> Self {
        WebStorageEnvironment {
            data: engine.load().unwrap_or_default(),
            engine,
        }
    }

    fn clear(&mut self) {
        self.data.clear();
        let _ = self.engine.clear();
    }

    fn delete(&mut self, key: &str) {
        let _ = self.engine.delete(key);
    }

    fn set(&mut self, key: &str, value: &str) {
        let _ = self.engine.set(key, value);
    }
}

impl<E: WebStorageEngine> Drop for WebStorageEnvironment<E> {
    fn drop(&mut self) {
        self.engine.save(&self.data);
    }
}

struct WebStorageManager {
    port: GenericReceiver<WebStorageThreadMsg>,
    session_storage_origins: StorageOrigins,
    local_storage_origins: StorageOrigins,
    session_data: FxHashMap<WebViewId, FxHashMap<ImmutableOrigin, OriginEntry>>,
    config_dir: Option<PathBuf>,
    thread_pool: Arc<ThreadPool>,
    environments: FxHashMap<ImmutableOrigin, WebStorageEnvironment<SqliteEngine>>,
}

impl WebStorageManager {
    fn new(
        port: GenericReceiver<WebStorageThreadMsg>,
        config_dir: Option<PathBuf>,
    ) -> WebStorageManager {
        let mut local_storage_origins = StorageOrigins::new();
        if let Some(ref config_dir) = config_dir {
            read_json_from_file(&mut local_storage_origins, config_dir, "localstorage.json");
        }
        // Uses an estimate of the system cpus to process Webstorage transactions
        // See https://doc.rust-lang.org/stable/std/thread/fn.available_parallelism.html
        // If no information can be obtained about the system, uses 4 threads as a default
        let thread_count = thread::available_parallelism()
            .map(|i| i.get())
            .unwrap_or(pref!(threadpools_fallback_worker_num) as usize)
            .min(pref!(threadpools_webstorage_workers_max).max(1) as usize);
        WebStorageManager {
            port,
            session_storage_origins: StorageOrigins::new(),
            local_storage_origins,
            session_data: FxHashMap::default(),
            config_dir,
            thread_pool: Arc::new(ThreadPool::new(thread_count, "WebStorage".to_string())),
            environments: FxHashMap::default(),
        }
    }
}

impl WebStorageManager {
    fn start(&mut self) {
        loop {
            match self.port.recv().unwrap() {
                WebStorageThreadMsg::Length(sender, storage_type, webview_id, url) => {
                    self.length(sender, storage_type, webview_id, url)
                },
                WebStorageThreadMsg::Key(sender, storage_type, webview_id, url, index) => {
                    self.key(sender, storage_type, webview_id, url, index)
                },
                WebStorageThreadMsg::Keys(sender, storage_type, webview_id, url) => {
                    self.keys(sender, storage_type, webview_id, url)
                },
                WebStorageThreadMsg::SetItem(
                    sender,
                    storage_type,
                    webview_id,
                    url,
                    name,
                    value,
                ) => {
                    self.set_item(sender, storage_type, webview_id, url, name, value);
                },
                WebStorageThreadMsg::GetItem(sender, storage_type, webview_id, url, name) => {
                    self.request_item(sender, storage_type, webview_id, url, name)
                },
                WebStorageThreadMsg::RemoveItem(sender, storage_type, webview_id, url, name) => {
                    self.remove_item(sender, storage_type, webview_id, url, name);
                },
                WebStorageThreadMsg::Clear(sender, storage_type, webview_id, url) => {
                    self.clear(sender, storage_type, webview_id, url);
                },
                WebStorageThreadMsg::Clone {
                    sender,
                    src: src_webview_id,
                    dest: dest_webview_id,
                } => {
                    self.clone(src_webview_id, dest_webview_id);
                    let _ = sender.send(());
                },
                WebStorageThreadMsg::OriginDescriptors(sender, storage_type) => {
                    self.origin_descriptors(sender, storage_type);
                },
                WebStorageThreadMsg::CollectMemoryReport(sender) => {
                    let reports = self.collect_memory_reports();
                    sender.send(ProcessReports::new(reports));
                },
                WebStorageThreadMsg::Exit(sender) => {
                    // Nothing to do since we save localstorage set eagerly.
                    let _ = sender.send(());
                    break;
                },
            }
        }
    }

    fn collect_memory_reports(&self) -> Vec<Report> {
        let mut reports = vec![];
        perform_memory_report(|ops| {
            reports.push(Report {
                path: path!["storage", "local"],
                kind: ReportKind::ExplicitJemallocHeapSize,
                size: self.environments.size_of(ops),
            });

            reports.push(Report {
                path: path!["storage", "session"],
                kind: ReportKind::ExplicitJemallocHeapSize,
                size: self.session_data.size_of(ops),
            });
        });
        reports
    }

    fn save_local_storage_origins(&self) {
        if let Some(ref config_dir) = self.config_dir {
            write_json_to_file(&self.local_storage_origins, config_dir, "localstorage.json");
        }
    }

    fn get_origin_location(&self, origin: &ImmutableOrigin) -> Option<PathBuf> {
        match &self.config_dir {
            Some(config_dir) => {
                const NAMESPACE_SERVO_WEBSTORAGE: &uuid::Uuid = &Uuid::from_bytes([
                    0x37, 0x9e, 0x56, 0xb0, 0x1a, 0x76, 0x44, 0xc5, 0xa4, 0xdb, 0xe2, 0x18, 0xc5,
                    0xc8, 0xa3, 0x5d,
                ]);
                let origin_uuid = Uuid::new_v5(
                    NAMESPACE_SERVO_WEBSTORAGE,
                    origin.ascii_serialization().as_bytes(),
                );
                Some(config_dir.join("webstorage").join(origin_uuid.to_string()))
            },
            None => None,
        }
    }

    fn add_new_environment(&mut self, origin: &ImmutableOrigin) {
        let origin_location = self.get_origin_location(origin);

        let engine = SqliteEngine::new(&origin_location, self.thread_pool.clone()).unwrap();
        let environment = WebStorageEnvironment::new(engine);
        self.environments.insert(origin.clone(), environment);
    }

    fn get_environment(
        &mut self,
        origin: &ImmutableOrigin,
    ) -> &WebStorageEnvironment<SqliteEngine> {
        if self.environments.contains_key(origin) {
            return self.environments.get(origin).unwrap();
        }

        self.add_new_environment(origin);

        self.environments.get(origin).unwrap()
    }

    fn get_environment_mut(
        &mut self,
        origin: &ImmutableOrigin,
    ) -> &mut WebStorageEnvironment<SqliteEngine> {
        if self.environments.contains_key(origin) {
            return self.environments.get_mut(origin).unwrap();
        }

        self.add_new_environment(origin);

        self.environments.get_mut(origin).unwrap()
    }

    fn select_data(
        &mut self,
        storage_type: StorageType,
        webview_id: WebViewId,
        origin: ImmutableOrigin,
    ) -> Option<&OriginEntry> {
        match storage_type {
            StorageType::Session => self
                .session_data
                .get(&webview_id)
                .and_then(|origin_map| origin_map.get(&origin)),
            StorageType::Local => {
                // FIXME: Selecting data for read only operations should not
                // create a new origin descriptor. However, this currently
                // needs to happen because get_environment always creates an
                // environment, even for read only operations.
                if self.local_storage_origins.ensure_origin_descriptor(&origin) {
                    self.save_local_storage_origins();
                }
                Some(&self.get_environment(&origin).data)
            },
        }
    }

    fn select_data_mut(
        &mut self,
        storage_type: StorageType,
        webview_id: WebViewId,
        origin: ImmutableOrigin,
    ) -> Option<&mut OriginEntry> {
        match storage_type {
            StorageType::Session => self
                .session_data
                .get_mut(&webview_id)
                .and_then(|origin_map| origin_map.get_mut(&origin)),
            StorageType::Local => {
                // FIXME: Selecting data for read only operations should not
                // create a new origin descriptor. However, this currently
                // needs to happen because get_environment always creates an
                // environment, even for read only operations.
                if self.local_storage_origins.ensure_origin_descriptor(&origin) {
                    self.save_local_storage_origins();
                }
                Some(&mut self.get_environment_mut(&origin).data)
            },
        }
    }

    fn ensure_data_mut(
        &mut self,
        storage_type: StorageType,
        webview_id: WebViewId,
        origin: ImmutableOrigin,
    ) -> &mut OriginEntry {
        match storage_type {
            StorageType::Session => {
                self.session_storage_origins
                    .ensure_origin_descriptor(&origin);
                self.session_data
                    .entry(webview_id)
                    .or_default()
                    .entry(origin)
                    .or_default()
            },
            StorageType::Local => {
                if self.local_storage_origins.ensure_origin_descriptor(&origin) {
                    self.save_local_storage_origins();
                }
                &mut self.get_environment_mut(&origin).data
            },
        }
    }

    fn length(
        &mut self,
        sender: GenericSender<usize>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
    ) {
        let data = self.select_data(storage_type, webview_id, url.origin());
        sender
            .send(data.map_or(0, |entry| entry.inner().len()))
            .unwrap();
    }

    fn key(
        &mut self,
        sender: GenericSender<Option<String>>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
        index: u32,
    ) {
        let data = self.select_data(storage_type, webview_id, url.origin());
        let key = data
            .and_then(|entry| entry.inner().keys().nth(index as usize))
            .cloned();
        sender.send(key).unwrap();
    }

    fn keys(
        &mut self,
        sender: GenericSender<Vec<String>>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
    ) {
        let data = self.select_data(storage_type, webview_id, url.origin());
        let keys = data.map_or(vec![], |entry| entry.inner().keys().cloned().collect());

        sender.send(keys).unwrap();
    }

    /// Sends Ok(changed, Some(old_value)) in case there was a previous
    /// value with the same key name but with different value name
    /// otherwise sends Err(()) to indicate that the operation would result in
    /// exceeding the quota limit
    fn set_item(
        &mut self,
        sender: GenericSender<Result<(bool, Option<String>), ()>>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
        name: String,
        value: String,
    ) {
        let (this_storage_size, other_storage_size) = {
            let local_data = self.select_data(StorageType::Local, webview_id, url.origin());
            let local_data_size = local_data.map_or(0, OriginEntry::size);
            let session_data = self.select_data(StorageType::Session, webview_id, url.origin());
            let session_data_size = session_data.map_or(0, OriginEntry::size);
            match storage_type {
                StorageType::Local => (local_data_size, session_data_size),
                StorageType::Session => (session_data_size, local_data_size),
            }
        };

        let entry = self.ensure_data_mut(storage_type, webview_id, url.origin());

        let mut new_total_size = this_storage_size + value.len();
        if let Some(old_value) = entry.inner().get(&name) {
            new_total_size -= old_value.len();
        } else {
            new_total_size += name.len();
        }

        let message = if (new_total_size + other_storage_size) > QUOTA_SIZE_LIMIT {
            Err(())
        } else {
            let result =
                entry
                    .insert(name.clone(), value.clone())
                    .map_or(Ok((true, None)), |old| {
                        if old == value {
                            Ok((false, None))
                        } else {
                            Ok((true, Some(old)))
                        }
                    });
            // XXX Should this be scoped to localStorage only?
            // Tracked in issue #41324.
            let env = self.get_environment_mut(&url.origin());
            env.set(&name, &value);
            result
        };
        sender.send(message).unwrap();
    }

    fn request_item(
        &mut self,
        sender: GenericSender<Option<String>>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
        name: String,
    ) {
        let data = self.select_data(storage_type, webview_id, url.origin());
        sender
            .send(data.and_then(|entry| entry.inner().get(&name)).cloned())
            .unwrap();
    }

    /// Sends Some(old_value) in case there was a previous value with the key name, otherwise sends None
    fn remove_item(
        &mut self,
        sender: GenericSender<Option<String>>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
        name: String,
    ) {
        let data = self.select_data_mut(storage_type, webview_id, url.origin());
        let old_value = data.and_then(|entry| entry.remove(&name));
        sender.send(old_value).unwrap();
        let env = self.get_environment_mut(&url.origin());
        env.delete(&name);
    }

    fn clear(
        &mut self,
        sender: GenericSender<bool>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
    ) {
        let data = self.select_data_mut(storage_type, webview_id, url.origin());
        sender
            .send(data.is_some_and(|entry| {
                if !entry.inner().is_empty() {
                    entry.clear();
                    true
                } else {
                    false
                }
            }))
            .unwrap();
        let env = self.get_environment_mut(&url.origin());
        env.clear();
    }

    fn clone(&mut self, src_webview_id: WebViewId, dest_webview_id: WebViewId) {
        let Some(src_origin_entries) = self.session_data.get(&src_webview_id) else {
            return;
        };

        let dest_origin_entries = src_origin_entries.clone();
        self.session_data
            .insert(dest_webview_id, dest_origin_entries);
    }

    fn origin_descriptors(
        &mut self,
        sender: GenericSender<Vec<OriginDescriptor>>,
        storage_type: StorageType,
    ) {
        let origin_descriptors = match storage_type {
            StorageType::Session => self.session_storage_origins.origin_descriptors(),
            StorageType::Local => self.local_storage_origins.origin_descriptors(),
        };
        let _ = sender.send(origin_descriptors);
    }
}
