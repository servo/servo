/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(dead_code)]

use dom::bindings::js::Root;
use dom::blob::Blob;
use origin::Origin;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(JSTraceable)]
struct EntryPair(Origin, Root<Blob>);

#[derive(JSTraceable)]
pub struct BlobURLStore {
    entries: HashMap<Uuid, EntryPair>,
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
        match self.entries.get(&id) {
            Some(ref pair) => {
                if pair.0.same_origin(origin) {
                    Ok(pair.1.r())
                } else {
                    Err(BlobURLStoreError::InvalidOrigin)
                }
            }
            None => Err(BlobURLStoreError::InvalidKey)
        }
    }

    pub fn add_entry(&mut self, id: Uuid, origin: Origin, blob: &Blob) {
        self.entries.insert(id, EntryPair(origin, Root::from_ref(blob)));
    }

    pub fn delete_entry(&mut self, id: Uuid) {
        self.entries.remove(&id);
    }
}
