/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains implementations in script that are serializable,
//! as per <https://html.spec.whatwg.org/multipage/#serializable-objects>.
//! The implementations are here instead of in script
//! so that the other modules involved in the serialization don't have
//! to depend on script.

use std::cell::RefCell;
use std::path::PathBuf;

use base::id::BlobId;
use malloc_size_of_derive::MallocSizeOf;
use net_traits::filemanager_thread::RelativePos;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// File-based blob
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct FileBlob {
    #[ignore_malloc_size_of = "Uuid are hard(not really)"]
    id: Uuid,
    #[ignore_malloc_size_of = "PathBuf are hard"]
    name: Option<PathBuf>,
    cache: RefCell<Option<Vec<u8>>>,
    size: u64,
}

impl FileBlob {
    /// Create a new file blob.
    pub fn new(id: Uuid, name: Option<PathBuf>, cache: Option<Vec<u8>>, size: u64) -> FileBlob {
        FileBlob {
            id,
            name,
            cache: RefCell::new(cache),
            size,
        }
    }

    /// Get the size of the file.
    pub fn get_size(&self) -> u64 {
        self.size
    }

    /// Get the cached file data, if any.
    pub fn get_cache(&self) -> Option<Vec<u8>> {
        self.cache.borrow().clone()
    }

    /// Cache data.
    pub fn cache_bytes(&self, bytes: Vec<u8>) {
        *self.cache.borrow_mut() = Some(bytes);
    }

    /// Get the file id.
    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

impl crate::BroadcastClone for BlobImpl {
    type Id = BlobId;

    fn source(data: &crate::StructuredSerializedData) -> &Option<std::collections::HashMap<Self::Id, Self>> {
        &data.blobs
    }

    fn destination(data: &mut crate::StructuredSerializedData) -> &mut Option<std::collections::HashMap<Self::Id, Self>> {
        &mut data.blobs
    }

    fn clone_for_broadcast(&self) -> Option<Self> {
        let type_string = self.type_string();

        if let BlobData::Memory(ref bytes) = self.blob_data() {
            let blob_clone = BlobImpl::new_from_bytes(bytes.clone(), type_string);

            // Note: we insert the blob at the original id,
            // otherwise this will not match the storage key as serialized by SM in `serialized`.
            // The clone has it's own new Id however.
            return Some(blob_clone);
        } else {
            // Not panicking only because this is called from the constellation.
            log::warn!("Serialized blob not in memory format(should never happen).");
        }
        return None;
    }
}

/// The data backing a DOM Blob.
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct BlobImpl {
    /// UUID of the blob.
    blob_id: BlobId,
    /// Content-type string
    type_string: String,
    /// Blob data-type.
    blob_data: BlobData,
    /// Sliced blobs referring to this one.
    slices: Vec<BlobId>,
}

/// Different backends of Blob
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum BlobData {
    /// File-based blob, whose content lives in the net process
    File(FileBlob),
    /// Memory-based blob, whose content lives in the script process
    Memory(Vec<u8>),
    /// Sliced blob, including parent blob-id and
    /// relative positions of current slicing range,
    /// IMPORTANT: The depth of tree is only two, i.e. the parent Blob must be
    /// either File-based or Memory-based
    Sliced(BlobId, RelativePos),
}

impl BlobImpl {
    /// Construct memory-backed BlobImpl
    pub fn new_from_bytes(bytes: Vec<u8>, type_string: String) -> BlobImpl {
        let blob_id = BlobId::new();
        let blob_data = BlobData::Memory(bytes);
        BlobImpl {
            blob_id,
            type_string,
            blob_data,
            slices: vec![],
        }
    }

    /// Construct file-backed BlobImpl from File ID
    pub fn new_from_file(file_id: Uuid, name: PathBuf, size: u64, type_string: String) -> BlobImpl {
        let blob_id = BlobId::new();
        let blob_data = BlobData::File(FileBlob {
            id: file_id,
            name: Some(name),
            cache: RefCell::new(None),
            size,
        });
        BlobImpl {
            blob_id,
            type_string,
            blob_data,
            slices: vec![],
        }
    }

    /// Construct a BlobImpl from a slice of a parent.
    pub fn new_sliced(rel_pos: RelativePos, parent: BlobId, type_string: String) -> BlobImpl {
        let blob_id = BlobId::new();
        let blob_data = BlobData::Sliced(parent, rel_pos);
        BlobImpl {
            blob_id,
            type_string,
            blob_data,
            slices: vec![],
        }
    }

    /// Get a clone of the blob-id
    pub fn blob_id(&self) -> BlobId {
        self.blob_id
    }

    /// Get a clone of the type-string
    pub fn type_string(&self) -> String {
        self.type_string.clone()
    }

    /// Get a mutable ref to the data
    pub fn blob_data(&self) -> &BlobData {
        &self.blob_data
    }

    /// Get a mutable ref to the data
    pub fn blob_data_mut(&mut self) -> &mut BlobData {
        &mut self.blob_data
    }
}
