/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use url::Url;

use util::str::DOMString;
use util::task::spawn_named;

#[derive(Copy)]
pub enum StorageType {
    Session,
    Local
}

/// Request operations on the storage data associated with a particular url
pub enum StorageTaskMsg {
    /// gets the number of key/value pairs present in the associated storage data
    Length(Sender<u32>, Url, StorageType),

    /// gets the name of the key at the specified index in the associated storage data
    Key(Sender<Option<DOMString>>, Url, StorageType, u32),

    /// gets the value associated with the given key in the associated storage data
    GetItem(Sender<Option<DOMString>>, Url, StorageType, DOMString),

    /// sets the value of the given key in the associated storage data
    /// TODO throw QuotaExceededError in case of error
    SetItem(Sender<bool>, Url, StorageType, DOMString, DOMString),

    /// removes the key/value pair for the given key in the associated storage data
    RemoveItem(Sender<bool>, Url, StorageType, DOMString),

    /// clears the associated storage data by removing all the key/value pairs
    Clear(Sender<bool>, Url, StorageType),

    /// shut down this task
    Exit
}

/// Handle to a storage task
pub type StorageTask = Sender<StorageTaskMsg>;

pub trait StorageTaskFactory {
    fn new() -> Self;
}

impl StorageTaskFactory for StorageTask {
    /// Create a StorageTask
    fn new() -> StorageTask {
        let (chan, port) = channel();
        spawn_named("StorageManager".to_owned(), move || {
            StorageManager::new(port).start();
        });
        chan
    }
}

struct StorageManager {
    port: Receiver<StorageTaskMsg>,
    session_data: HashMap<String, BTreeMap<DOMString, DOMString>>,
    local_data: HashMap<String, BTreeMap<DOMString, DOMString>>,
}

impl StorageManager {
    fn new(port: Receiver<StorageTaskMsg>) -> StorageManager {
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
                StorageTaskMsg::SetItem(sender, url, storage_type, name, value) => {
                    self.set_item(sender, url, storage_type, name, value)
                }
                StorageTaskMsg::GetItem(sender, url, storage_type, name) => {
                    self.get_item(sender, url, storage_type, name)
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

    fn select_data(& self, storage_type: StorageType) -> &HashMap<String, BTreeMap<DOMString, DOMString>> {
        match storage_type {
            StorageType::Session => &self.session_data,
            StorageType::Local => &self.local_data
        }
    }

    fn select_data_mut(&mut self, storage_type: StorageType) -> &mut HashMap<String, BTreeMap<DOMString, DOMString>> {
        match storage_type {
            StorageType::Session => &mut self.session_data,
            StorageType::Local => &mut self.local_data
        }
    }

    fn length(&self, sender: Sender<u32>, url: Url, storage_type: StorageType) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data(storage_type);
        sender.send(data.get(&origin).map_or(0u, |entry| entry.len()) as u32).unwrap();
    }

    fn key(&self, sender: Sender<Option<DOMString>>, url: Url, storage_type: StorageType, index: u32) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data(storage_type);
        sender.send(data.get(&origin)
                    .and_then(|entry| entry.keys().nth(index as uint))
                    .map(|key| key.clone())).unwrap();
    }

    fn set_item(&mut self, sender: Sender<bool>, url: Url, storage_type: StorageType, name: DOMString, value: DOMString) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data_mut(storage_type);
        if !data.contains_key(&origin) {
            data.insert(origin.clone(), BTreeMap::new());
        }

        let updated = data.get_mut(&origin).map(|entry| {
            if entry.get(&origin).map_or(true, |item| *item != value) {
                entry.insert(name.clone(), value.clone());
                true
            } else {
                false
            }
        }).unwrap();

        sender.send(updated).unwrap();
    }

    fn get_item(&self, sender: Sender<Option<DOMString>>, url: Url, storage_type: StorageType, name: DOMString) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data(storage_type);
        sender.send(data.get(&origin)
                    .and_then(|entry| entry.get(&name))
                    .map(|value| value.to_string())).unwrap();
    }

    fn remove_item(&mut self, sender: Sender<bool>, url: Url, storage_type: StorageType, name: DOMString) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data_mut(storage_type);
        sender.send(data.get_mut(&origin)
                    .map_or(false, |entry| entry.remove(&name).is_some())).unwrap();
    }

    fn clear(&mut self, sender: Sender<bool>, url: Url, storage_type: StorageType) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data_mut(storage_type);
        sender.send(data.get_mut(&origin)
                    .map_or(false, |entry| {
                        if !entry.is_empty() {
                            entry.clear();
                            true
                        } else {
                            false
                        }})).unwrap();
    }

    fn get_origin_as_string(&self, url: Url) -> String {
        let mut origin = "".to_string();
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
