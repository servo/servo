/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use server::{ClientId, Server, SharedServerProxy};
use std::collections::{HashMap, TreeMap};
use std::sync::{Arc, Mutex};
use url::Url;

use servo_util::str::DOMString;
use servo_util::task::spawn_named;

/// Request operations on the storage data associated with a particular url
#[deriving(Decodable, Encodable)]
pub enum StorageTaskMsg {
    /// gets the number of key/value pairs present in the associated storage data
    Length(Url),

    /// gets the name of the key at the specified index in the associated storage data
    Key(Url, u32),

    /// gets the value associated with the given key in the associated storage data
    GetItem(Url, DOMString),

    /// sets the value of the given key in the associated storage data
    /// TODO throw QuotaExceededError in case of error
    SetItem(Url, DOMString, DOMString),

    /// removes the key/value pair for the given key in the associated storage data
    RemoveItem(Url, DOMString),

    /// clears the associated storage data by removing all the key/value pairs
    Clear(Url),
}

/// Response operations on the storage data associated with a particular URL.
#[deriving(Decodable, Encodable)]
pub enum StorageTaskResponse {
    /// The number of key/value pairs present in the associated storage data.
    Length(u32),
    /// The name of the key at the specified index in the associated storage data.
    Key(Option<DOMString>),
    /// The value associated with the given key in the associated storage data.
    GetItem(Option<DOMString>),
    /// A simple acknowledgement of success/failure, used for `SetItem`, `RemoveItem`, and `Clear`.
    Complete(bool),
}

/// Handle to a storage task
#[deriving(Clone)]
pub struct StorageTask {
    pub client: SharedServerProxy<StorageTaskMsg,StorageTaskResponse>,
}

impl StorageTask {
    #[inline]
    pub fn from_client(client: SharedServerProxy<StorageTaskMsg,StorageTaskResponse>)
                       -> StorageTask {
        StorageTask {
            client: client,
        }
    }

    pub fn send(&self, msg: StorageTaskMsg) -> StorageTaskResponse {
        self.client.lock().send_sync(msg)
    }

    pub fn create_new_client(&self) -> StorageTask {
        StorageTask {
            client: Arc::new(Mutex::new(self.client.lock().create_new_client())),
        }
    }
}

pub trait StorageTaskFactory {
    fn new() -> StorageTask;
}

impl StorageTaskFactory for StorageTask {
    /// Create a StorageTask
    fn new() -> StorageTask {
        let mut server = Server::new("StorageTask");
        let client = Arc::new(Mutex::new(server.create_new_client()));
        spawn_named("StorageManager".to_owned(), proc() {
            StorageManager::new(server).start();
        });
        StorageTask {
            client: client,
        }
    }
}

struct StorageManager {
    server: Server<StorageTaskMsg,StorageTaskResponse>,
    data: HashMap<String, TreeMap<DOMString, DOMString>>,
}

impl StorageManager {
    fn new(server: Server<StorageTaskMsg,StorageTaskResponse>) -> StorageManager {
        StorageManager {
            server: server,
            data: HashMap::new(),
        }
    }
}

impl StorageManager {
    fn start(&mut self) {
        while let Some(msgs) = self.server.recv() {
            for (client_id, msg) in msgs.into_iter() {
                match msg {
                    StorageTaskMsg::Length(url) => self.length(client_id, url),
                    StorageTaskMsg::Key(url, index) => self.key(client_id, url, index),
                    StorageTaskMsg::SetItem(url, name, value) => {
                        self.set_item(client_id, url, name, value)
                    }
                    StorageTaskMsg::GetItem(url, name) => self.get_item(client_id, url, name),
                    StorageTaskMsg::RemoveItem(url, name) => {
                        self.remove_item(client_id, url, name)
                    }
                    StorageTaskMsg::Clear(url) => self.clear(client_id, url),
                }
            }
        }
    }

    fn length(&self, sender: ClientId, url: Url) {
        let origin = self.get_origin_as_string(url);
        self.server.send(sender, StorageTaskResponse::Length(
                self.data.get(&origin).map_or(0u, |entry| entry.len()) as u32));
    }

    fn key(&self, sender: ClientId, url: Url, index: u32) {
        let origin = self.get_origin_as_string(url);
        self.server.send(sender,
                         StorageTaskResponse::Key(
                self.data.get(&origin)
                         .and_then(|entry| entry.keys().nth(index as uint))
                         .map(|key| key.clone())));
    }

    fn set_item(&mut self, sender: ClientId, url: Url, name: DOMString, value: DOMString) {
        let origin = self.get_origin_as_string(url);
        if !self.data.contains_key(&origin) {
            self.data.insert(origin.clone(), TreeMap::new());
        }

        let updated = self.data.get_mut(&origin).map(|entry| {
            if entry.get(&origin).map_or(true, |item| item.as_slice() != value.as_slice()) {
                entry.insert(name.clone(), value.clone());
                true
            } else {
                false
            }
        }).unwrap();

        self.server.send(sender, StorageTaskResponse::Complete(updated));
    }

    fn get_item(&self, sender: ClientId, url: Url, name: DOMString) {
        let origin = self.get_origin_as_string(url);
        self.server.send(sender,
                         StorageTaskResponse::GetItem(
                             self.data.get(&origin)
                                      .and_then(|entry| entry.get(&name))
                                      .map(|value| value.to_string())));
    }

    fn remove_item(&mut self, sender: ClientId, url: Url, name: DOMString) {
        let origin = self.get_origin_as_string(url);
        self.server.send(sender,
                         StorageTaskResponse::Complete(
                             self.data.get_mut(&origin)
                                      .map_or(false, |entry| entry.remove(&name).is_some())));
    }

    fn clear(&mut self, sender: ClientId, url: Url) {
        let origin = self.get_origin_as_string(url);
        self.server.send(sender,
                         StorageTaskResponse::Complete(self.data.get_mut(&origin)
                                                                .map_or(false, |entry| {
                                                                    if !entry.is_empty() {
                                                                        entry.clear();
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }})));
    }

    fn get_origin_as_string(&self, url: Url) -> String {
        let mut origin = "".to_string();
        origin.push_str(url.scheme.as_slice());
        origin.push_str("://");
        url.domain().map(|domain| origin.push_str(domain.as_slice()));
        url.port().map(|port| {
            origin.push_str(":");
            origin.push_str(port.to_string().as_slice());
        });
        origin.push_str("/");
        origin
    }
}
