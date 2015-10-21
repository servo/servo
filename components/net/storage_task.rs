/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use net_traits::storage_task::{StorageTask, StorageTaskMsg, StorageType};
use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::mpsc::channel;
use url::Url;
use util::str::DOMString;
use util::task::spawn_named;

const QUOTA_SIZE_LIMIT: usize = 5 * 1024 * 1024;

pub trait StorageTaskFactory {
    fn new() -> Self;
}

impl StorageTaskFactory for StorageTask {
    /// Create a StorageTask
    fn new() -> StorageTask {
        let (chan, port) = ipc::channel().unwrap();
        spawn_named("StorageManager".to_owned(), move || {
            StorageManager::new(port).start();
        });
        chan
    }
}

struct StorageManager {
    port: IpcReceiver<StorageTaskMsg>,
    session_data: HashMap<String, (usize, BTreeMap<DOMString, DOMString>)>,
    local_data: HashMap<String, (usize, BTreeMap<DOMString, DOMString>)>,
}

impl StorageManager {
    fn new(port: IpcReceiver<StorageTaskMsg>) -> StorageManager {
        StorageManager {
            port: port,
            session_data: HashMap::new(),
            local_data: HashMap::new(),
        }
    }
}

impl StorageManager {
    fn start(&mut self) {
        loop {
            match self.port.recv().unwrap() {
                StorageTaskMsg::Length(sender, url, storage_type) => {
                    self.length(sender, url, storage_type)
                }
                StorageTaskMsg::Key(sender, url, storage_type, index) => {
                    self.key(sender, url, storage_type, index)
                }
                StorageTaskMsg::Keys(sender, url, storage_type) => {
                    self.keys(sender, url, storage_type)
                }
                StorageTaskMsg::SetItem(sender, url, storage_type, name, value) => {
                    self.set_item(sender, url, storage_type, name, value)
                }
                StorageTaskMsg::GetItem(sender, url, storage_type, name) => {
                    self.request_item(sender, url, storage_type, name)
                }
                StorageTaskMsg::RemoveItem(sender, url, storage_type, name) => {
                    self.remove_item(sender, url, storage_type, name)
                }
                StorageTaskMsg::Clear(sender, url, storage_type) => {
                    self.clear(sender, url, storage_type)
                }
                StorageTaskMsg::Exit => {
                    break
                }
            }
        }
    }

    fn select_data(&self, storage_type: StorageType)
                   -> &HashMap<String, (usize, BTreeMap<DOMString, DOMString>)> {
        match storage_type {
            StorageType::Session => &self.session_data,
            StorageType::Local => &self.local_data
        }
    }

    fn select_data_mut(&mut self, storage_type: StorageType)
                       -> &mut HashMap<String, (usize, BTreeMap<DOMString, DOMString>)> {
        match storage_type {
            StorageType::Session => &mut self.session_data,
            StorageType::Local => &mut self.local_data
        }
    }

    fn length(&self, sender: IpcSender<usize>, url: Url, storage_type: StorageType) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type);
        sender.send(data.get(&origin).map_or(0, |&(_, ref entry)| entry.len())).unwrap();
    }

    fn key(&self,
           sender: IpcSender<Option<DOMString>>,
           url: Url,
           storage_type: StorageType,
           index: u32) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type);
        let key = data.get(&origin)
                      .and_then(|&(_, ref entry)| entry.keys().nth(index as usize))
                      .cloned();
        sender.send(key).unwrap();
    }

    fn keys(&self,
            sender: IpcSender<Vec<DOMString>>,
            url: Url,
            storage_type: StorageType) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type);
        let keys = data.get(&origin)
                       .map_or(vec![], |&(_, ref entry)| entry.keys().cloned().collect());

        sender.send(keys).unwrap();
    }

    /// Sends Ok(changed, Some(old_value)) in case there was a previous
    /// value with the same key name but with different value name
    /// otherwise sends Err(()) to indicate that the operation would result in
    /// exceeding the quota limit
    fn set_item(&mut self,
                sender: IpcSender<Result<(bool, Option<DOMString>), ()>>,
                url: Url,
                storage_type: StorageType,
                name: DOMString,
                value: DOMString) {
        let origin = self.origin_as_string(url);

        let current_total_size = {
            let local_data = self.select_data(StorageType::Local);
            let session_data = self.select_data(StorageType::Session);
            let local_data_size = local_data.get(&origin).map_or(0, |&(total, _)| total);
            let session_data_size = session_data.get(&origin).map_or(0, |&(total, _)| total);
            local_data_size + session_data_size
        };

        let data = self.select_data_mut(storage_type);
        if !data.contains_key(&origin) {
            data.insert(origin.clone(), (0, BTreeMap::new()));
        }

        let message = data.get_mut(&origin).map(|&mut (ref mut total, ref mut entry)| {
            let mut new_total_size = current_total_size + value.as_bytes().len();
            if let Some(old_value) = entry.get(&name) {
                new_total_size -= old_value.as_bytes().len();
            } else {
                new_total_size += name.as_bytes().len();
            }

            if new_total_size > QUOTA_SIZE_LIMIT {
                return Err(());
            }

            let message = entry.insert(name.clone(), value.clone()).map_or(
                Ok((true, None)),
                |old| if old == value {
                    Ok((false, None))
                } else {
                    Ok((true, Some(old)))
                });
            *total = new_total_size;
            message
        }).unwrap();
        sender.send(message).unwrap();
    }

    fn request_item(&self,
                sender: IpcSender<Option<DOMString>>,
                url: Url,
                storage_type: StorageType,
                name: DOMString) {
        let origin = self.origin_as_string(url);
        let data = self.select_data(storage_type);
        sender.send(data.get(&origin)
                    .and_then(|&(_, ref entry)| entry.get(&name))
                    .map(|value| value.to_string())).unwrap();
    }

    /// Sends Some(old_value) in case there was a previous value with the key name, otherwise sends None
    fn remove_item(&mut self,
                   sender: IpcSender<Option<DOMString>>,
                   url: Url,
                   storage_type: StorageType,
                   name: DOMString) {
        let origin = self.origin_as_string(url);
        let data = self.select_data_mut(storage_type);
        let old_value = data.get_mut(&origin).and_then(|&mut (ref mut total, ref mut entry)| {
            entry.remove(&name).and_then(|old| {
                *total -= name.as_bytes().len() + old.as_bytes().len();
                Some(old)
            })
        });
        sender.send(old_value).unwrap();
    }

    fn clear(&mut self, sender: IpcSender<bool>, url: Url, storage_type: StorageType) {
        let origin = self.origin_as_string(url);
        let data = self.select_data_mut(storage_type);
        sender.send(data.get_mut(&origin)
                    .map_or(false, |&mut (ref mut total, ref mut entry)| {
                        if !entry.is_empty() {
                            entry.clear();
                            *total = 0;
                            true
                        } else {
                            false
                        }})).unwrap();
    }

    fn origin_as_string(&self, url: Url) -> String {
        let mut origin = "".to_owned();
        origin.push_str(&url.scheme);
        origin.push_str("://");
        url.domain().map(|domain| origin.push_str(&domain));
        url.port().map(|port| {
            origin.push_str(":");
            origin.push_str(&port.to_string());
        });
        origin.push_str("/");
        origin
    }
}
