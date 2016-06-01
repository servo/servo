/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(dead_code)]

use dom::bindings::js::JS;
use dom::blob::Blob;
use origin::Origin;
use std::collections::HashMap;
use uuid::Uuid;

#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
struct EntryPair(Origin, JS<Blob>);

// HACK: to work around the HeapSizeOf of Uuid
#[derive(PartialEq, HeapSizeOf, Eq, Hash, JSTraceable)]
struct BlobUrlId(#[ignore_heap_size_of = "defined in uuid"] Uuid);

#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub struct BlobURLStore {
    entries: HashMap<BlobUrlId, EntryPair>,
}

pub enum BlobURLStoreError {
    InvalidKey,
    InvalidOrigin,
}

impl BlobURLStore {
    pub fn new() -> BlobURLStore {
        BlobURLStore {
            entries: HashMap::new(),
        }
    }

    pub fn request(&self, id: Uuid, origin: &Origin) -> Result<&Blob, BlobURLStoreError> {
        match self.entries.get(&BlobUrlId(id)) {
            Some(ref pair) => {
                if pair.0.same_origin(origin) {
                    Ok(&pair.1)
                } else {
                    Err(BlobURLStoreError::InvalidOrigin)
                }
            }
            None => Err(BlobURLStoreError::InvalidKey)
        }
    }

    pub fn add_entry(&mut self, id: Uuid, origin: Origin, blob: &Blob) {
        self.entries.insert(BlobUrlId(id), EntryPair(origin, JS::from_ref(blob)));
    }

    pub fn delete_entry(&mut self, id: Uuid) {
        self.entries.remove(&BlobUrlId(id));
    }
}
