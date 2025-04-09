/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::PathBuf;
use std::thread;

use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use log::warn;
use net_traits::storage_thread::{StorageThreadMsg, StorageType};
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;

use crate::resource_thread;

const QUOTA_SIZE_LIMIT: usize = 5 * 1024 * 1024;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LocalStorageRecord {
    pub operation: String,
    pub origin: String,
    pub name: String,
    pub value: String,
}

pub trait StorageThreadFactory {
    fn new(config_dir: Option<PathBuf>) -> Self;
}

impl StorageThreadFactory for IpcSender<StorageThreadMsg> {
    /// Create a storage thread
    fn new(config_dir: Option<PathBuf>) -> IpcSender<StorageThreadMsg> {
        let (chan, port) = ipc::channel().unwrap();
        thread::Builder::new()
            .name("StorageManager".to_owned())
            .spawn(move || {
                StorageManager::new(port, config_dir).start();
            })
            .expect("Thread spawning failed");
        chan
    }
}

struct StorageManager {
    port: IpcReceiver<StorageThreadMsg>,
    session_data: HashMap<String, (usize, BTreeMap<String, String>)>,
    local_data: HashMap<String, (usize, BTreeMap<String, String>)>,
    config_dir: Option<PathBuf>,
}

impl StorageManager {
    /// When the local data is updated, we append the change as a single line in tracking file. At exit, we store the
    /// data to a json file and delete the tracking file. The tracking file is checked at the start of the program in
    /// case the changes from previous run is not properly stored in the json (e.g. caused by crash).
    /// TODO: Implement proper storage mechanism based on <https://storage.spec.whatwg.org/> and use proper key-value
    /// store mechanism.
    const LOCAL_DATA_FILENAME: &str = "local_data.json";
    const TRACK_CHANGES_FILENAME: &str = "track_changes.jsonl";

    fn new(port: IpcReceiver<StorageThreadMsg>, config_dir: Option<PathBuf>) -> StorageManager {
        let mut local_data = HashMap::new();
        if let Some(ref config_dir) = config_dir {
            resource_thread::read_json_from_file(
                &mut local_data,
                config_dir,
                Self::LOCAL_DATA_FILENAME,
            );
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
                StorageThreadMsg::Initialize(sender) => self.handle_tracking_file(sender),
                StorageThreadMsg::Length(sender, url, storage_type) => {
                    self.length(sender, url, storage_type)
                },
                StorageThreadMsg::Key(sender, url, storage_type, index) => {
                    self.key(sender, url, storage_type, index)
                },
                StorageThreadMsg::Keys(sender, url, storage_type) => {
                    self.keys(sender, url, storage_type)
                },
                StorageThreadMsg::SetItem(sender, url, storage_type, name, value) => {
                    self.set_item(
                        sender,
                        url.clone(),
                        storage_type,
                        name.clone(),
                        value.clone(),
                    );
                    if matches!(storage_type, StorageType::Local) {
                        self.append_change(url, "SET", &name, &value);
                    }
                },
                StorageThreadMsg::GetItem(sender, url, storage_type, name) => {
                    self.request_item(sender, url, storage_type, name)
                },
                StorageThreadMsg::RemoveItem(sender, url, storage_type, name) => {
                    self.remove_item(sender, url.clone(), storage_type, name.clone());
                    if matches!(storage_type, StorageType::Local) {
                        self.append_change(url, "REMOVE", &name, "");
                    }
                },
                StorageThreadMsg::Clear(sender, url, storage_type) => {
                    self.clear(sender, url.clone(), storage_type);
                    if matches!(storage_type, StorageType::Local) {
                        self.append_change(url, "CLEAR", "", "");
                    }
                },
                StorageThreadMsg::Exit(sender) => {
                    let _ = self.save_state();
                    let _ = sender.send(());
                    break;
                },
            }
        }
    }

    fn update_from_record(&mut self, track_item: LocalStorageRecord) {
        match track_item.operation.as_str() {
            "SET" => {
                let storage_size = self
                    .local_data
                    .get(&track_item.origin)
                    .map_or(0, |&(total, _)| total);

                if !self.local_data.contains_key(&track_item.origin) {
                    self.local_data
                        .insert(track_item.origin.clone(), (0, BTreeMap::new()));
                }

                if let Some((total, entry)) = self.local_data.get_mut(&track_item.origin) {
                    let mut new_total_size = storage_size + track_item.value.len();
                    if let Some(old_value) = entry.get(&track_item.name) {
                        new_total_size -= old_value.len();
                    } else {
                        new_total_size += track_item.name.len();
                    }

                    entry.insert(track_item.name.clone(), track_item.value.clone());

                    *total = new_total_size;
                }
            },
            "REMOVE" => {
                self.local_data.get_mut(&track_item.origin).and_then(
                    |&mut (ref mut total, ref mut entry)| {
                        entry.remove(&track_item.name).inspect(|old| {
                            *total -= track_item.name.len() + old.len();
                        })
                    },
                );
            },
            "CLEAR" => {
                if let Some((total, entry)) = self.local_data.get_mut(&track_item.origin) {
                    *total = 0;
                    entry.clear();
                }
            },
            _ => warn!("Invalid operation at tracking file"),
        }
    }

    // Merge local data with the change in TRACK_CHANGES_FILENAME,
    // then clear the file (or create the file if it did not exist).
    fn handle_tracking_file(&mut self, sender: IpcSender<Result<(), ()>>) {
        let track_list = if let Some(ref config_dir) = self.config_dir {
            resource_thread::read_jsonl_file(config_dir, Self::TRACK_CHANGES_FILENAME)
        } else {
            return;
        };
        for track_elem in track_list {
            self.update_from_record(track_elem);
        }

        // Before clearing TRACK_CHANGES_FILENAME, make sure we write the data to LOCAL_DATA_FILENAME first.
        // This ensures the data from previous session is saved in the JSON file even if the current session crashes.
        sender.send(self.save_state()).unwrap();
    }

    // Append every change to TRACK_CHANGES_FILENAME instead of overwriting LOCAL_DATA_FILENAME.
    fn append_change(&self, url: ServoUrl, operation: &str, name: &str, value: &str) {
        if let Some(ref config_dir) = self.config_dir {
            let origin = self.origin_as_string(url);
            let change = LocalStorageRecord {
                operation: operation.to_string(),
                origin,
                name: name.to_string(),
                value: value.to_string(),
            };
            resource_thread::append_to_jsonl_file(change, config_dir, Self::TRACK_CHANGES_FILENAME);
        }
    }

    fn save_state(&self) -> Result<(), ()> {
        if let Some(ref config_dir) = self.config_dir {
            let path = config_dir.join(Self::TRACK_CHANGES_FILENAME);
            if File::create(path).is_err() {
                return Err(());
            }
            return resource_thread::write_json_to_file(
                &self.local_data,
                config_dir,
                Self::LOCAL_DATA_FILENAME,
            );
        }
        Err(())
    }

    fn select_data(
        &self,
        storage_type: StorageType,
    ) -> &HashMap<String, (usize, BTreeMap<String, String>)> {
        match storage_type {
            StorageType::Session => &self.session_data,
            StorageType::Local => &self.local_data,
        }
    }

    fn select_data_mut(
        &mut self,
        storage_type: StorageType,
    ) -> &mut HashMap<String, (usize, BTreeMap<String, String>)> {
        match storage_type {
            StorageType::Session => &mut self.session_data,
            StorageType::Local => &mut self.local_data,
        }
    }

    fn length(&self, sender: IpcSender<usize>, url: ServoUrl, storage_type: StorageType) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type);
        sender
            .send(data.get(&origin).map_or(0, |(_, entry)| entry.len()))
            .unwrap();
    }

    fn key(
        &self,
        sender: IpcSender<Option<String>>,
        url: ServoUrl,
        storage_type: StorageType,
        index: u32,
    ) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type);
        let key = data
            .get(&origin)
            .and_then(|(_, entry)| entry.keys().nth(index as usize))
            .cloned();
        sender.send(key).unwrap();
    }

    fn keys(&self, sender: IpcSender<Vec<String>>, url: ServoUrl, storage_type: StorageType) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type);
        let keys = data
            .get(&origin)
            .map_or(vec![], |(_, entry)| entry.keys().cloned().collect());

        sender.send(keys).unwrap();
    }

    /// Sends Ok(changed, Some(old_value)) in case there was a previous
    /// value with the same key name but with different value name
    /// otherwise sends Err(()) to indicate that the operation would result in
    /// exceeding the quota limit
    fn set_item(
        &mut self,
        sender: IpcSender<Result<(bool, Option<String>), ()>>,
        url: ServoUrl,
        storage_type: StorageType,
        name: String,
        value: String,
    ) {
        let origin = self.origin_as_string(url);

        let (this_storage_size, other_storage_size) = {
            let local_data = self.select_data(StorageType::Local);
            let session_data = self.select_data(StorageType::Session);
            let local_data_size = local_data.get(&origin).map_or(0, |&(total, _)| total);
            let session_data_size = session_data.get(&origin).map_or(0, |&(total, _)| total);
            match storage_type {
                StorageType::Local => (local_data_size, session_data_size),
                StorageType::Session => (session_data_size, local_data_size),
            }
        };

        let data = self.select_data_mut(storage_type);
        if !data.contains_key(&origin) {
            data.insert(origin.clone(), (0, BTreeMap::new()));
        }

        let message = data
            .get_mut(&origin)
            .map(|&mut (ref mut total, ref mut entry)| {
                let mut new_total_size = this_storage_size + value.len();
                if let Some(old_value) = entry.get(&name) {
                    new_total_size -= old_value.len();
                } else {
                    new_total_size += name.len();
                }

                if (new_total_size + other_storage_size) > QUOTA_SIZE_LIMIT {
                    return Err(());
                }

                let message =
                    entry
                        .insert(name.clone(), value.clone())
                        .map_or(Ok((true, None)), |old| {
                            if old == value {
                                Ok((false, None))
                            } else {
                                Ok((true, Some(old)))
                            }
                        });
                *total = new_total_size;
                message
            })
            .unwrap();
        sender.send(message).unwrap();
    }

    fn request_item(
        &self,
        sender: IpcSender<Option<String>>,
        url: ServoUrl,
        storage_type: StorageType,
        name: String,
    ) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type);
        sender
            .send(
                data.get(&origin)
                    .and_then(|(_, entry)| entry.get(&name))
                    .cloned(),
            )
            .unwrap();
    }

    /// Sends Some(old_value) in case there was a previous value with the key name, otherwise sends None
    fn remove_item(
        &mut self,
        sender: IpcSender<Option<String>>,
        url: ServoUrl,
        storage_type: StorageType,
        name: String,
    ) {
        let origin = self.origin_as_string(url);
        let data = self.select_data_mut(storage_type);
        let old_value = data
            .get_mut(&origin)
            .and_then(|&mut (ref mut total, ref mut entry)| {
                entry.remove(&name).inspect(|old| {
                    *total -= name.len() + old.len();
                })
            });
        sender.send(old_value).unwrap();
    }

    fn clear(&mut self, sender: IpcSender<bool>, url: ServoUrl, storage_type: StorageType) {
        let origin = self.origin_as_string(url);
        let data = self.select_data_mut(storage_type);
        sender
            .send(
                data.get_mut(&origin)
                    .is_some_and(|&mut (ref mut total, ref mut entry)| {
                        if !entry.is_empty() {
                            entry.clear();
                            *total = 0;
                            true
                        } else {
                            false
                        }
                    }),
            )
            .unwrap();
    }

    fn origin_as_string(&self, url: ServoUrl) -> String {
        url.origin().ascii_serialization()
    }
}
