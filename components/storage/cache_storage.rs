/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::thread;

use log::error;
use net_traits::request::Request;
use net_traits::response::Response;
use servo_base::generic_channel::{self, GenericReceiver, GenericSender};
use servo_url::ImmutableOrigin;
use storage_traits::cache_storage::{
    CacheStorageError, CacheStorageThreadHandle, CacheStorageThreadMessage,
    CacheStorageThreadResponse,
};
use storage_traits::client_storage::StorageProxyMap;

trait CacheStorageEngine {
    type Error: Debug;

    /// <https://w3c.github.io/ServiceWorker/#cache-storage-has>
    fn has_cache(
        &mut self,
        origin: ImmutableOrigin,
        cache_name: &str,
    ) -> Result<bool, CacheStorageError<Self::Error>>;

    /// <https://w3c.github.io/ServiceWorker/#cache-storage-open>
    fn open_cache(
        &mut self,
        origin: ImmutableOrigin,
        cache_name: String,
        proxy_map: &StorageProxyMap,
    ) -> Result<(), CacheStorageError<Self::Error>>;

    /// <https://w3c.github.io/ServiceWorker/#cache-keys>
    fn keys(
        &mut self,
        origin: ImmutableOrigin,
        cache_name: &str,
    ) -> Result<Vec<String>, CacheStorageError<Self::Error>>;

    /// <https://w3c.github.io/ServiceWorker/#dom-cachestorage-delete>
    fn delete_cache(
        &mut self,
        origin: ImmutableOrigin,
        cache_name: &str,
        proxy_map: &StorageProxyMap,
    ) -> Result<bool, CacheStorageError<Self::Error>>;
}

/// <https://w3c.github.io/ServiceWorker/#dfn-request-response-list>
pub struct RequestResponseList {
    /// A list of tuples consisting of a request (a request) and a response (a response)
    pub list: Vec<(Request, Response)>,
}

pub struct MemCacheStorageEngine {
    /// <https://w3c.github.io/ServiceWorker/#dfn-name-to-cache-map>
    name_to_cache_map: HashMap<(ImmutableOrigin, String), RequestResponseList>,
}

impl CacheStorageEngine for MemCacheStorageEngine {
    type Error = ();

    /// <https://w3c.github.io/ServiceWorker/#cache-storage-has>
    /// The parallel steps.
    fn has_cache(
        &mut self,
        origin: ImmutableOrigin,
        cache_name: &str,
    ) -> Result<bool, CacheStorageError<Self::Error>> {
        // Step 2.1:For each key → value of the relevant name to cache map:
        // Step 2.1.1: If cacheName matches key, resolve promise with true and abort these steps.
        // Step 2.2: Resolve promise with false.
        // Note: promise resolved in the callback in CacheStorage.
        Ok(self
            .name_to_cache_map
            .contains_key(&(origin, cache_name.to_string())))
    }

    /// <https://w3c.github.io/ServiceWorker/#cache-storage-open>
    /// The parallel steps.
    fn open_cache(
        &mut self,
        origin: ImmutableOrigin,
        cache_name: String,
        proxy_map: &StorageProxyMap,
    ) -> Result<(), CacheStorageError<Self::Error>> {
        // Step 2.1: For each key → value of the relevant name to cache map:
        // Step 2.1.1: If cacheName matches key, then:
        // Resolve promise with a new Cache object that represents value.
        // Note: promise resolved in script.
        if self
            .name_to_cache_map
            .contains_key(&(origin.clone(), cache_name.clone()))
        {
            // Step 2.1.2: Abort these steps.
            return Ok(());
        }

        // Step 2.2: Let cache be a new request response list.
        let cache = RequestResponseList { list: Vec::new() };

        // Step 2.3: Set the relevant name to cache map[cacheName] to cache.
        // If this cache write operation failed due to exceeding the granted quota limit,
        // reject promise with a QuotaExceededError and abort these steps.
        // Note: there are no quota checks storage side at this point.
        let Ok(response) = proxy_map
            .handle
            .create_database(proxy_map.bottle_id, cache_name.clone())
            .recv()
        else {
            return Err(CacheStorageError::Internal(()));
        };
        if response.is_err() {
            return Err(CacheStorageError::Internal(()));
        }
        self.name_to_cache_map.insert((origin, cache_name), cache);

        // Step 2.4: Resolve promise with a new Cache object that represents cache.
        // Note: promise resolved in script.
        Ok(())
    }

    /// <https://w3c.github.io/ServiceWorker/#cache-keys>
    /// The parallel steps.
    fn keys(
        &mut self,
        origin: ImmutableOrigin,
        cache_name: &str,
    ) -> Result<Vec<String>, CacheStorageError<Self::Error>> {
        // Step 5.1: Let requests be an empty list.
        let mut requests: Vec<String> = Vec::new();

        // Step 5.2: If the optional argument request is omitted, then:
        // Step 5.2.1: For each requestResponse of the relevant request response list:
        let Some(relevant_cache) = self
            .name_to_cache_map
            .get(&(origin, cache_name.to_string()))
        else {
            return Err(CacheStorageError::Internal(()));
        };
        // Step 5.2.2: Add requestResponse’s request to requests.
        for (request, _response) in &relevant_cache.list {
            requests.push(request.url().to_string());
        }

        // Step 5.3: Else:
        // Note: implementing this steps depends on Query Cache; todo.

        Ok(requests)
    }

    /// <https://w3c.github.io/ServiceWorker/#dom-cachestorage-delete>
    /// The "running the algorithm specified in has" part, and the parallel steps.
    fn delete_cache(
        &mut self,
        origin: ImmutableOrigin,
        cache_name: &str,
        proxy: &StorageProxyMap,
    ) -> Result<bool, CacheStorageError<Self::Error>> {
        // Step 1: Let promise be the result of running the algorithm specified in has(cacheName) method with cacheName.
        // Step 2: Return the result of reacting to promise with a fulfillment handler that,
        // when called with argument cacheExists,
        // performs the following substeps:
        // Note: skipping th promise here.

        // Step 2.1: If cacheExists is false, then:
        // Step 2.1.1: Resolve promise with false.
        // Note: done below using return value from `is_some`.

        // Step 2.3: Run the following substeps in parallel:
        // Step 2.3.1: Remove the relevant name to cache map[cacheName].
        let has = self
            .name_to_cache_map
            .remove(&(origin, cache_name.to_string()))
            .is_some();

        if has {
            let Ok(response) = proxy
                .handle
                .delete_database(proxy.bottle_id, cache_name.to_string())
                .recv()
            else {
                return Err(CacheStorageError::Internal(()));
            };
            if response.is_err() {
                return Err(CacheStorageError::Internal(()));
            }
        }

        // Step 2.3.2: Resolve promise with true.
        // Note: promise resolved in script.
        Ok(has)
    }
}

pub trait CacheStorageThreadFactory {
    fn new(config_dir: Option<PathBuf>, temporary_storage: bool) -> Self;
}

impl CacheStorageThreadFactory for CacheStorageThreadHandle {
    fn new(config_dir: Option<PathBuf>, temporary_storage: bool) -> CacheStorageThreadHandle {
        let (generic_sender, generic_receiver) = generic_channel::channel().unwrap();
        let mut temp_dir: Option<tempfile::TempDir> = None;
        let base_dir = config_dir
            .unwrap_or_else(|| {
                let tmp_dir = tempfile::tempdir().unwrap();
                let path = tmp_dir.path().to_path_buf();
                temp_dir = Some(tmp_dir);
                path
            })
            .join("cachestorage");
        let storage_dir = if temporary_storage {
            let unique_id = uuid::Uuid::new_v4().to_string();
            base_dir.join("temporary").join(unique_id)
        } else {
            base_dir.join("default_v1")
        };
        std::fs::create_dir_all(&storage_dir)
            .expect("Failed to create CacheStorage storage directory");
        let sender_clone = generic_sender.clone();
        thread::Builder::new()
            .name("CacheStorageThread".to_owned())
            .spawn(move || {
                // Keep temp_dir alive while the thread runs.
                let _ = temp_dir;
                let engine = MemCacheStorageEngine {
                    name_to_cache_map: Default::default(),
                };
                let mut cache_storage_thread =
                    CacheStorageThread::new(sender_clone, generic_receiver, engine);
                cache_storage_thread.start();
            })
            .expect("Thread spawning failed");

        CacheStorageThreadHandle::new(generic_sender)
    }
}

struct CacheStorageThread<E: CacheStorageEngine> {
    receiver: GenericReceiver<CacheStorageThreadMessage>,
    // Note: a sender to self might be required later for the storage engine.
    _sender: GenericSender<CacheStorageThreadMessage>,
    engine: E,
}

impl<E> CacheStorageThread<E>
where
    E: CacheStorageEngine,
{
    pub fn new(
        _sender: GenericSender<CacheStorageThreadMessage>,
        receiver: GenericReceiver<CacheStorageThreadMessage>,
        engine: E,
    ) -> CacheStorageThread<E> {
        CacheStorageThread {
            _sender,
            receiver,
            engine,
        }
    }

    pub fn start(&mut self) {
        while let Ok(message) = self.receiver.recv() {
            match message {
                CacheStorageThreadMessage::HasCache {
                    cache_name,
                    callback,
                    proxy: _,
                    origin,
                } => {
                    let result = self.engine.has_cache(origin.clone(), &cache_name);
                    if callback
                        .send(CacheStorageThreadResponse::HasCacheResult(
                            result.map_err(|e| format!("{:?}", e)),
                        ))
                        .is_err()
                    {
                        error!("Failed to send response to script for HasCache message.");
                    }
                },
                CacheStorageThreadMessage::OpenCache {
                    cache_name,
                    callback,
                    proxy,
                    origin,
                } => {
                    let result = self
                        .engine
                        .open_cache(origin.clone(), cache_name.clone(), &proxy);
                    if callback
                        .send(CacheStorageThreadResponse::OpenCacheResult {
                            result: result.map_err(|e| format!("{:?}", e)),
                            cache_name,
                        })
                        .is_err()
                    {
                        error!("Failed to send response to script for OpenCache message.");
                    }
                },
                CacheStorageThreadMessage::Keys {
                    cache_name,
                    callback,
                    origin,
                } => {
                    let result = self.engine.keys(origin.clone(), &cache_name);
                    if callback
                        .send(CacheStorageThreadResponse::KeysResult(
                            result.map_err(|e| format!("{:?}", e)),
                        ))
                        .is_err()
                    {
                        error!("Failed to send response to script for Keys message.");
                    }
                },
                CacheStorageThreadMessage::DeleteCache {
                    cache_name,
                    callback,
                    proxy,
                    origin,
                } => {
                    let result = self
                        .engine
                        .delete_cache(origin.clone(), &cache_name, &proxy);
                    if callback
                        .send(CacheStorageThreadResponse::DeleteCacheResult(
                            result.map_err(|e| format!("{:?}", e)),
                        ))
                        .is_err()
                    {
                        error!("Failed to send response to script for DeleteCache message.");
                    }
                },
                CacheStorageThreadMessage::Exit(sender) => {
                    let _ = sender.send(());
                    break;
                },
            }
        }
    }
}
