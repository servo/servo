/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{Deref, DerefMut};

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::{GenericCallback, GenericSender};
use servo_url::ImmutableOrigin;

use crate::client_storage::StorageProxyMap;

#[derive(Debug, Deserialize, Serialize)]
pub enum CacheStorageError<T> {
    Internal(T),
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct CacheStorageThreadHandle {
    sender: GenericSender<CacheStorageThreadMessage>,
}

impl CacheStorageThreadHandle {
    pub fn new(sender: GenericSender<CacheStorageThreadMessage>) -> Self {
        CacheStorageThreadHandle { sender }
    }
}

impl From<CacheStorageThreadHandle> for GenericSender<CacheStorageThreadMessage> {
    fn from(handle: CacheStorageThreadHandle) -> Self {
        handle.sender
    }
}

impl From<GenericSender<CacheStorageThreadMessage>> for CacheStorageThreadHandle {
    fn from(sender: GenericSender<CacheStorageThreadMessage>) -> Self {
        CacheStorageThreadHandle::new(sender)
    }
}

impl Deref for CacheStorageThreadHandle {
    type Target = GenericSender<CacheStorageThreadMessage>;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

impl DerefMut for CacheStorageThreadHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sender
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CacheStorageThreadMessage {
    /// <https://w3c.github.io/ServiceWorker/#cache-storage-has>
    HasCache {
        cache_name: String,
        callback: GenericCallback<CacheStorageThreadResponse>,
        proxy: StorageProxyMap,
        origin: ImmutableOrigin,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CacheStorageThreadResponse {
    HasCacheResult(Result<bool, String>),
}
