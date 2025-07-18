/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains implementations in script that are serializable,
//! as per <https://html.spec.whatwg.org/multipage/#serializable-objects>.
//! The implementations are here instead of in script as they need to
//! be passed through the Constellation.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use base::id::{BlobId, DomExceptionId, DomPointId, ImageBitmapId};
use malloc_size_of_derive::MallocSizeOf;
use net_traits::filemanager_thread::RelativePos;
use pixels::Snapshot;
use serde::{Deserialize, Serialize};
use servo_url::ImmutableOrigin;
use strum::EnumIter;
use uuid::Uuid;

use super::StructuredSerializedData;

pub(crate) trait BroadcastClone
where
    Self: Sized,
{
    /// The ID type that uniquely identify each value.
    type Id: Eq + std::hash::Hash + Copy;
    /// Clone this value so that it can be reused with a broadcast channel.
    /// Only return None if cloning is impossible.
    fn clone_for_broadcast(&self) -> Option<Self>;
    /// The field from which to clone values.
    fn source(data: &StructuredSerializedData) -> &Option<HashMap<Self::Id, Self>>;
    /// The field into which to place cloned values.
    fn destination(data: &mut StructuredSerializedData) -> &mut Option<HashMap<Self::Id, Self>>;
}

/// All the DOM interfaces that can be serialized.
#[derive(Clone, Copy, Debug, EnumIter)]
pub enum Serializable {
    /// The `Blob` interface.
    Blob,
    /// The `DOMPoint` interface.
    DomPoint,
    /// The `DOMPointReadOnly` interface.
    DomPointReadOnly,
    /// The `DOMException` interface.
    DomException,
    /// The `ImageBitmap` interface.
    ImageBitmap,
}

impl Serializable {
    pub(super) fn clone_values(
        &self,
    ) -> fn(&StructuredSerializedData, &mut StructuredSerializedData) {
        match self {
            Serializable::Blob => StructuredSerializedData::clone_all_of_type::<BlobImpl>,
            Serializable::DomPointReadOnly => {
                StructuredSerializedData::clone_all_of_type::<DomPoint>
            },
            Serializable::DomPoint => StructuredSerializedData::clone_all_of_type::<DomPoint>,
            Serializable::DomException => {
                StructuredSerializedData::clone_all_of_type::<DomException>
            },
            Serializable::ImageBitmap => {
                StructuredSerializedData::clone_all_of_type::<SerializableImageBitmap>
            },
        }
    }
}

/// Message for communication between the constellation and a global managing broadcast channels.
#[derive(Debug, Deserialize, Serialize)]
pub struct BroadcastChannelMsg {
    /// The origin of this message.
    pub origin: ImmutableOrigin,
    /// The name of the channel.
    pub channel_name: String,
    /// A data-holder for serialized data.
    pub data: StructuredSerializedData,
}

impl Clone for BroadcastChannelMsg {
    fn clone(&self) -> BroadcastChannelMsg {
        BroadcastChannelMsg {
            data: self.data.clone_for_broadcast(),
            origin: self.origin.clone(),
            channel_name: self.channel_name.clone(),
        }
    }
}

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

impl BroadcastClone for BlobImpl {
    type Id = BlobId;

    fn source(
        data: &StructuredSerializedData,
    ) -> &Option<std::collections::HashMap<Self::Id, Self>> {
        &data.blobs
    }

    fn destination(
        data: &mut StructuredSerializedData,
    ) -> &mut Option<std::collections::HashMap<Self::Id, Self>> {
        &mut data.blobs
    }

    fn clone_for_broadcast(&self) -> Option<Self> {
        let type_string = self.type_string();

        if let BlobData::Memory(bytes) = self.blob_data() {
            let blob_clone = BlobImpl::new_from_bytes(bytes.clone(), type_string);

            // Note: we insert the blob at the original id,
            // otherwise this will not match the storage key as serialized by SM in `serialized`.
            // The clone has it's own new Id however.
            return Some(blob_clone);
        } else {
            // Not panicking only because this is called from the constellation.
            log::warn!("Serialized blob not in memory format(should never happen).");
        }
        None
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
    pub fn new_sliced(range: RelativePos, parent: BlobId, type_string: String) -> BlobImpl {
        let blob_id = BlobId::new();
        let blob_data = BlobData::Sliced(parent, range);
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

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
/// A serializable version of the DOMPoint/DOMPointReadOnly interface.
pub struct DomPoint {
    /// The x coordinate.
    pub x: f64,
    /// The y coordinate.
    pub y: f64,
    /// The z coordinate.
    pub z: f64,
    /// The w coordinate.
    pub w: f64,
}

impl BroadcastClone for DomPoint {
    type Id = DomPointId;

    fn source(
        data: &StructuredSerializedData,
    ) -> &Option<std::collections::HashMap<Self::Id, Self>> {
        &data.points
    }

    fn destination(
        data: &mut StructuredSerializedData,
    ) -> &mut Option<std::collections::HashMap<Self::Id, Self>> {
        &mut data.points
    }

    fn clone_for_broadcast(&self) -> Option<Self> {
        Some(self.clone())
    }
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
/// A serializable version of the DOMException interface.
pub struct DomException {
    pub message: String,
    pub name: String,
}

impl BroadcastClone for DomException {
    type Id = DomExceptionId;

    fn source(
        data: &StructuredSerializedData,
    ) -> &Option<std::collections::HashMap<Self::Id, Self>> {
        &data.exceptions
    }

    fn destination(
        data: &mut StructuredSerializedData,
    ) -> &mut Option<std::collections::HashMap<Self::Id, Self>> {
        &mut data.exceptions
    }

    fn clone_for_broadcast(&self) -> Option<Self> {
        Some(self.clone())
    }
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
/// A serializable version of the ImageBitmap interface.
pub struct SerializableImageBitmap {
    pub bitmap_data: Snapshot,
}

impl BroadcastClone for SerializableImageBitmap {
    type Id = ImageBitmapId;

    fn source(
        data: &StructuredSerializedData,
    ) -> &Option<std::collections::HashMap<Self::Id, Self>> {
        &data.image_bitmaps
    }

    fn destination(
        data: &mut StructuredSerializedData,
    ) -> &mut Option<std::collections::HashMap<Self::Id, Self>> {
        &mut data.image_bitmaps
    }

    fn clone_for_broadcast(&self) -> Option<Self> {
        Some(self.clone())
    }
}
