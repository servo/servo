/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::thread;

use base::id::WebViewId;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use malloc_size_of::MallocSizeOf;
use net_traits::storage_thread::{StorageThreadMsg, StorageType};
use profile_traits::mem::{
    ProcessReports, ProfilerChan as MemProfilerChan, Report, ReportKind, perform_memory_report,
};
use profile_traits::path;
use servo_url::ServoUrl;

use crate::resource_thread;

const QUOTA_SIZE_LIMIT: usize = 5 * 1024 * 1024;

pub trait StorageThreadFactory {
    fn new(config_dir: Option<PathBuf>, mem_profiler_chan: MemProfilerChan) -> Self;
}

impl StorageThreadFactory for IpcSender<StorageThreadMsg> {
    /// Create a storage thread
    fn new(
        config_dir: Option<PathBuf>,
        mem_profiler_chan: MemProfilerChan,
    ) -> IpcSender<StorageThreadMsg> {
        let (chan, port) = ipc::channel().unwrap();
        let chan2 = chan.clone();
        thread::Builder::new()
            .name("StorageManager".to_owned())
            .spawn(move || {
                mem_profiler_chan.run_with_memory_reporting(
                    || StorageManager::new(port, config_dir).start(),
                    String::from("storage-reporter"),
                    chan2,
                    StorageThreadMsg::CollectMemoryReport,
                );
            })
            .expect("Thread spawning failed");
        chan
    }
}

type OriginEntry = (usize, BTreeMap<String, String>);

struct StorageManager {
    port: IpcReceiver<StorageThreadMsg>,
    session_data: HashMap<WebViewId, HashMap<String, OriginEntry>>,
    local_data: HashMap<String, OriginEntry>,
    config_dir: Option<PathBuf>,
}

impl StorageManager {
    fn new(port: IpcReceiver<StorageThreadMsg>, config_dir: Option<PathBuf>) -> StorageManager {
        let mut local_data = HashMap::new();
        if let Some(ref config_dir) = config_dir {
            resource_thread::read_json_from_file(&mut local_data, config_dir, "local_data.json");
        }
        StorageManager {
            port,
            session_data: HashMap::new(),
            local_data,
            config_dir,
        }
    }
}

impl StorageManager {
    fn start(&mut self) {
        loop {
            match self.port.recv().unwrap() {
                StorageThreadMsg::Length(sender, storage_type, webview_id, url) => {
                    self.length(sender, storage_type, webview_id, url)
                },
                StorageThreadMsg::Key(sender, storage_type, webview_id, url, index) => {
                    self.key(sender, storage_type, webview_id, url, index)
                },
                StorageThreadMsg::Keys(sender, storage_type, webview_id, url) => {
                    self.keys(sender, storage_type, webview_id, url)
                },
                StorageThreadMsg::SetItem(sender, storage_type, webview_id, url, name, value) => {
                    self.set_item(sender, storage_type, webview_id, url, name, value);
                    self.save_state()
                },
                StorageThreadMsg::GetItem(sender, storage_type, webview_id, url, name) => {
                    self.request_item(sender, storage_type, webview_id, url, name)
                },
                StorageThreadMsg::RemoveItem(sender, storage_type, webview_id, url, name) => {
                    self.remove_item(sender, storage_type, webview_id, url, name);
                    self.save_state()
                },
                StorageThreadMsg::Clear(sender, storage_type, webview_id, url) => {
                    self.clear(sender, storage_type, webview_id, url);
                    self.save_state()
                },
                StorageThreadMsg::Clone {
                    sender,
                    src: src_webview_id,
                    dest: dest_webview_id,
                } => {
                    self.clone(src_webview_id, dest_webview_id);
                    let _ = sender.send(());
                },
                StorageThreadMsg::CollectMemoryReport(sender) => {
                    let reports = self.collect_memory_reports();
                    sender.send(ProcessReports::new(reports));
                },
                StorageThreadMsg::Exit(sender) => {
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
                size: self.local_data.size_of(ops),
            });

            reports.push(Report {
                path: path!["storage", "session"],
                kind: ReportKind::ExplicitJemallocHeapSize,
                size: self.session_data.size_of(ops),
            });
        });
        reports
    }

    fn save_state(&self) {
        if let Some(ref config_dir) = self.config_dir {
            resource_thread::write_json_to_file(&self.local_data, config_dir, "local_data.json");
        }
    }

    fn select_data(
        &self,
        storage_type: StorageType,
        webview_id: WebViewId,
        origin: &str,
    ) -> Option<&OriginEntry> {
        match storage_type {
            StorageType::Session => self
                .session_data
                .get(&webview_id)
                .and_then(|origin_map| origin_map.get(origin)),
            StorageType::Local => self.local_data.get(origin),
        }
    }

    fn select_data_mut(
        &mut self,
        storage_type: StorageType,
        webview_id: WebViewId,
        origin: &str,
    ) -> Option<&mut OriginEntry> {
        match storage_type {
            StorageType::Session => self
                .session_data
                .get_mut(&webview_id)
                .and_then(|origin_map| origin_map.get_mut(origin)),
            StorageType::Local => self.local_data.get_mut(origin),
        }
    }

    fn ensure_data_mut(
        &mut self,
        storage_type: StorageType,
        webview_id: WebViewId,
        origin: &str,
    ) -> &mut OriginEntry {
        match storage_type {
            StorageType::Session => self
                .session_data
                .entry(webview_id)
                .or_default()
                .entry(origin.to_string())
                .or_default(),
            StorageType::Local => self.local_data.entry(origin.to_string()).or_default(),
        }
    }

    fn length(
        &self,
        sender: IpcSender<usize>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
    ) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type, webview_id, &origin);
        sender
            .send(data.map_or(0, |(_, entry)| entry.len()))
            .unwrap();
    }

    fn key(
        &self,
        sender: IpcSender<Option<String>>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
        index: u32,
    ) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type, webview_id, &origin);
        let key = data
            .and_then(|(_, entry)| entry.keys().nth(index as usize))
            .cloned();
        sender.send(key).unwrap();
    }

    fn keys(
        &self,
        sender: IpcSender<Vec<String>>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
    ) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type, webview_id, &origin);
        let keys = data.map_or(vec![], |(_, entry)| entry.keys().cloned().collect());

        sender.send(keys).unwrap();
    }

    /// Sends Ok(changed, Some(old_value)) in case there was a previous
    /// value with the same key name but with different value name
    /// otherwise sends Err(()) to indicate that the operation would result in
    /// exceeding the quota limit
    fn set_item(
        &mut self,
        sender: IpcSender<Result<(bool, Option<String>), ()>>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
        name: String,
        value: String,
    ) {
        let origin = self.origin_as_string(url);

        let (this_storage_size, other_storage_size) = {
            let local_data = self.select_data(StorageType::Local, webview_id, &origin);
            let session_data = self.select_data(StorageType::Session, webview_id, &origin);
            let local_data_size = local_data.map_or(0, |&(total, _)| total);
            let session_data_size = session_data.map_or(0, |&(total, _)| total);
            match storage_type {
                StorageType::Local => (local_data_size, session_data_size),
                StorageType::Session => (session_data_size, local_data_size),
            }
        };

        let &mut (ref mut total, ref mut entry) =
            self.ensure_data_mut(storage_type, webview_id, &origin);

        let mut new_total_size = this_storage_size + value.len();
        if let Some(old_value) = entry.get(&name) {
            new_total_size -= old_value.len();
        } else {
            new_total_size += name.len();
        }

        let message = if (new_total_size + other_storage_size) > QUOTA_SIZE_LIMIT {
            Err(())
        } else {
            *total = new_total_size;
            entry
                .insert(name.clone(), value.clone())
                .map_or(Ok((true, None)), |old| {
                    if old == value {
                        Ok((false, None))
                    } else {
                        Ok((true, Some(old)))
                    }
                })
        };
        sender.send(message).unwrap();
    }

    fn request_item(
        &self,
        sender: IpcSender<Option<String>>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
        name: String,
    ) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type, webview_id, &origin);
        sender
            .send(data.and_then(|(_, entry)| entry.get(&name)).cloned())
            .unwrap();
    }

    /// Sends Some(old_value) in case there was a previous value with the key name, otherwise sends None
    fn remove_item(
        &mut self,
        sender: IpcSender<Option<String>>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
        name: String,
    ) {
        let origin = self.origin_as_string(url);
        let data = self.select_data_mut(storage_type, webview_id, &origin);
        let old_value = data.and_then(|&mut (ref mut total, ref mut entry)| {
            entry.remove(&name).inspect(|old| {
                *total -= name.len() + old.len();
            })
        });
        sender.send(old_value).unwrap();
    }

    fn clear(
        &mut self,
        sender: IpcSender<bool>,
        storage_type: StorageType,
        webview_id: WebViewId,
        url: ServoUrl,
    ) {
        let origin = self.origin_as_string(url);
        let data = self.select_data_mut(storage_type, webview_id, &origin);
        sender
            .send(data.is_some_and(|&mut (ref mut total, ref mut entry)| {
                if !entry.is_empty() {
                    entry.clear();
                    *total = 0;
                    true
                } else {
                    false
                }
            }))
            .unwrap();
    }

    fn clone(&mut self, src_webview_id: WebViewId, dest_webview_id: WebViewId) {
        let Some(src_origin_entries) = self.session_data.get(&src_webview_id) else {
            return;
        };

        let dest_origin_entries = src_origin_entries.clone();
        self.session_data
            .insert(dest_webview_id, dest_origin_entries);
    }

    fn origin_as_string(&self, url: ServoUrl) -> String {
        url.origin().ascii_serialization()
    }
}
