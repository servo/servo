/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;
use std::path::PathBuf;
use std::thread;

use log::error;
use servo_base::generic_channel::{self, GenericReceiver, GenericSender};
use storage_traits::cache_storage::{
    CacheStorageError, CacheStorageThreadHandle, CacheStorageThreadMessage,
    CacheStorageThreadResponse,
};

trait CacheStorageEngine {
    type Error: Debug;

    /// <https://w3c.github.io/ServiceWorker/#cache-storage-has>
    fn has_cache(&mut self, cache_name: &str) -> Result<bool, CacheStorageError<Self::Error>>;
}

pub struct DummyCacheStorageEngine;

impl CacheStorageEngine for DummyCacheStorageEngine {
    type Error = ();

    fn has_cache(&mut self, _cache_name: &str) -> Result<bool, CacheStorageError<Self::Error>> {
        // TODO: implement.
        Ok(false)
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
                let engine = DummyCacheStorageEngine;
                CacheStorageThread::new(sender_clone, generic_receiver, engine).start();
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
                    origin: _,
                } => {
                    let result = self.engine.has_cache(&cache_name);
                    if callback
                        .send(CacheStorageThreadResponse::HasCacheResult(
                            result.map(|_| false).map_err(|e| format!("{:?}", e)),
                        ))
                        .is_err()
                    {
                        error!("Failed to send response to script for HasCache message.");
                    }
                },
            }
        }
    }
}
