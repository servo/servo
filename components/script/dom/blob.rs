/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BlobBinding;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::UnionTypes::BlobOrString;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use encoding::all::UTF_8;
use encoding::types::{EncoderTrap, Encoding};
use ipc_channel::ipc;
use net_traits::filemanager_thread::{FileManagerThreadMsg, SelectedFileId};
use net_traits::filemanager_thread::{RelativePos, compute_slice_pos};
use std::ascii::AsciiExt;
use std::cell::Cell;
use std::sync::Arc;

#[derive(Clone, JSTraceable)]
pub struct DataSlice {
    bytes: Arc<Vec<u8>>,
    bytes_start: usize,
    bytes_end: usize
}

impl DataSlice {
    /// Construct DataSlice from reference counted bytes
    pub fn new(bytes: Arc<Vec<u8>>, start: Option<i64>, end: Option<i64>) -> DataSlice {
        let (start, end) = compute_slice_pos(start, end, bytes.len());

        DataSlice {
            bytes: bytes,
            bytes_start: start,
            bytes_end: end
        }
    }

    /// Construct data slice from a vector of bytes
    pub fn from_bytes(bytes: Vec<u8>) -> DataSlice {
        DataSlice::new(Arc::new(bytes), None, None)
    }

    /// Construct an empty data slice
    pub fn empty() -> DataSlice {
        DataSlice {
            bytes: Arc::new(Vec::new()),
            bytes_start: 0,
            bytes_end: 0,
        }
    }

    /// Get sliced bytes
    pub fn get_bytes(&self) -> &[u8] {
        &self.bytes[self.bytes_start..self.bytes_end]
    }

    /// Get length of sliced bytes
    pub fn size(&self) -> u64 {
        (self.bytes_end as u64) - (self.bytes_start as u64)
    }

    /// Further adjust the slice range based on passed-in relative positions
    pub fn slice(&self, pos: &RelativePos) -> DataSlice {
        let old_size = self.size();
        let (start, end) = compute_slice_pos(Some(pos.start), pos.end, old_size as usize);
        DataSlice {
            bytes: self.bytes.clone(),
            bytes_start: self.bytes_start + start,
            bytes_end: self.bytes_start + end,
        }
    }

    /// Get the slicing position info as a relative positions pair
    pub fn get_rel_pos(&self) -> RelativePos {
        RelativePos {
            start: self.bytes_start as i64,
            end: Some(self.bytes_end as i64),
        }
    }
}

#[must_root]
#[derive(JSTraceable)]
pub enum BlobImpl {
    /// File-based blob, including id and possibly cached content
    File(SelectedFileId, DOMRefCell<Option<DataSlice>>),
    /// Memory-based blob
    Memory(DataSlice),
    /// Sliced file-based blob, including parent blob's uuid, and
    /// relative positions representing current slicing range
    SlicedFile(SelectedFileId, RelativePos),
    /// Sliced memory-backed blob, including pointer to parent blob and
    /// a data slice representing bytes in currently sliced range
    SlicedMemory(JS<Blob>, DataSlice),
}

impl BlobImpl {
    /// Construct memory-backed BlobImpl from DataSlice
    pub fn new_from_slice(slice: DataSlice) -> BlobImpl {
        BlobImpl::Memory(slice)
    }

    /// Construct file-backed BlobImpl from File ID
    pub fn new_from_file(file_id: SelectedFileId) -> BlobImpl {
        BlobImpl::File(file_id, DOMRefCell::new(None))
    }

    /// Construct empty, memory-backed BlobImpl
    pub fn new_from_empty_slice() -> BlobImpl {
        BlobImpl::new_from_slice(DataSlice::empty())
    }
}

// https://w3c.github.io/FileAPI/#blob
#[dom_struct]
pub struct Blob {
    reflector_: Reflector,
    #[ignore_heap_size_of = "No clear owner"]
    blob_impl: DOMRefCell<BlobImpl>,
    typeString: String,
    isClosed_: Cell<bool>,
}

impl Blob {
    #[allow(unrooted_must_root)]
    pub fn new(global: GlobalRef, blob_impl: BlobImpl, typeString: String) -> Root<Blob> {
        let boxed_blob = box Blob::new_inherited(blob_impl, typeString);
        reflect_dom_object(boxed_blob, global, BlobBinding::Wrap)
    }

    #[allow(unrooted_must_root)]
    pub fn new_inherited(blob_impl: BlobImpl, typeString: String) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            blob_impl: DOMRefCell::new(blob_impl),
            typeString: typeString,
            isClosed_: Cell::new(false),
        }
    }

    // https://w3c.github.io/FileAPI/#constructorBlob
    pub fn Constructor(global: GlobalRef,
                       blobParts: Option<Vec<BlobOrString>>,
                       blobPropertyBag: &BlobBinding::BlobPropertyBag)
                       -> Fallible<Root<Blob>> {
        // TODO: accept other blobParts types - ArrayBuffer or ArrayBufferView
        let bytes: Vec<u8> = match blobParts {
            None => Vec::new(),
            Some(blobparts) => match blob_parts_to_bytes(blobparts) {
                Ok(bytes) => bytes,
                Err(_) => return Err(Error::InvalidCharacter),
            }
        };

        let slice = DataSlice::from_bytes(bytes);
        Ok(Blob::new(global, BlobImpl::new_from_slice(slice), blobPropertyBag.get_typestring()))
    }

    /// Get a slice to inner data, this might incur synchronous read and caching
    pub fn get_slice(&self) -> Result<DataSlice, ()> {
        match *self.blob_impl.borrow() {
            BlobImpl::File(ref id, ref cached) => {
                let buffer = match *cached.borrow() {
                    Some(ref s) => Ok(s.clone()),
                    None => {
                        let global = self.global();
                        let s = read_file(global.r(), id.clone())?;
                        Ok(s)
                    }
                };

                // Cache
                if let Ok(buf) = buffer.clone() {
                    *cached.borrow_mut() = Some(buf);
                }

                buffer
            }
            BlobImpl::Memory(ref s) => Ok(s.clone()),
            BlobImpl::SlicedMemory(_, ref dataslice) => Ok(dataslice.clone()),
            BlobImpl::SlicedFile(ref parent_id, ref rel_pos) => {
                let global = self.global();
                let s = read_file(global.r(), parent_id.clone())?;
                // XXX: should we reslice here? Should we cache here?
                Ok(s.slice(rel_pos))
            }
        }
    }

    /// Try to get a slice, and if any exception happens, return the empty slice
    pub fn get_slice_or_empty(&self) -> DataSlice {
        self.get_slice().unwrap_or(DataSlice::empty())
    }

    pub fn get_blob_impl(&self) -> &DOMRefCell<BlobImpl> {
        &self.blob_impl
    }
}

fn read_file(global: GlobalRef, id: SelectedFileId) -> Result<DataSlice, ()> {
    let file_manager = global.filemanager_thread();
    let (chan, recv) = ipc::channel().map_err(|_|())?;
    let origin = global.get_url().origin().unicode_serialization();
    let msg = FileManagerThreadMsg::ReadFile(chan, id, origin);
    let _ = file_manager.send(msg);

    let result = match recv.recv() {
        Ok(ret) => ret,
        Err(e) => {
            debug!("File manager thread has problem {:?}", e);
            return Err(())
        }
    };

    let bytes = result.map_err(|_|())?;
    Ok(DataSlice::from_bytes(bytes))
}

/// Extract bytes from BlobParts, used by Blob and File constructor
/// https://w3c.github.io/FileAPI/#constructorBlob
pub fn blob_parts_to_bytes(blobparts: Vec<BlobOrString>) -> Result<Vec<u8>, ()> {
    let mut ret = vec![];

    for blobpart in &blobparts {
        match blobpart {
            &BlobOrString::String(ref s) => {
                let mut bytes = UTF_8.encode(s, EncoderTrap::Replace).map_err(|_|())?;
                ret.append(&mut bytes);
            },
            &BlobOrString::Blob(ref b) => {
                ret.append(&mut b.get_slice_or_empty().bytes.to_vec());
            },
        }
    }

    Ok(ret)
}

impl BlobMethods for Blob {
    // https://w3c.github.io/FileAPI/#dfn-size
    fn Size(&self) -> u64 {
        self.get_slice_or_empty().size()
    }

    // https://w3c.github.io/FileAPI/#dfn-type
    fn Type(&self) -> DOMString {
        DOMString::from(self.typeString.clone())
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/FileAPI/#slice-method-algo
    fn Slice(&self,
             start: Option<i64>,
             end: Option<i64>,
             contentType: Option<DOMString>)
             -> Root<Blob> {
        let relativeContentType = match contentType {
            None => DOMString::new(),
            Some(mut str) => {
                if is_ascii_printable(&str) {
                    str.make_ascii_lowercase();
                    str
                } else {
                    DOMString::new()
                }
            }
        };

        let global = self.global();

        let rel_pos = RelativePos::from(start, end);

        let blob_impl = match *self.blob_impl.borrow() {
            BlobImpl::File(ref id, _) => {
                // Bump the reference counter in file manager thread
                let file_manager = global.r().filemanager_thread();
                let origin = global.r().get_url().origin().unicode_serialization();
                let msg = FileManagerThreadMsg::IncRef(id.clone(), origin);
                let _ = file_manager.send(msg);

                BlobImpl::SlicedFile(id.clone(), rel_pos)
            }
            BlobImpl::Memory(ref old_slice) => {
                BlobImpl::SlicedMemory(JS::from_ref(self), old_slice.slice(&rel_pos))
            }
            BlobImpl::SlicedFile(ref parent_id, ref old_rel_pos) => {
                // Bump the reference counter in file manager thread
                let file_manager = global.r().filemanager_thread();
                let origin = global.r().get_url().origin().unicode_serialization();
                let msg = FileManagerThreadMsg::IncRef(parent_id.clone(), origin);
                let _ = file_manager.send(msg);

                // Adjust the slicing position
                let new_rel_pos = old_rel_pos.recalculate(&rel_pos);
                BlobImpl::SlicedFile(parent_id.clone(), new_rel_pos)
            }
            BlobImpl::SlicedMemory(ref parent, ref old_slice) => {
                // Adjust the slicing position in DataSlice
                // no bytes are copied
                BlobImpl::SlicedMemory(parent.clone(), old_slice.slice(&rel_pos))
            }
        };

        Blob::new(global.r(), blob_impl, relativeContentType.into())
    }

    // https://w3c.github.io/FileAPI/#dfn-isClosed
    fn IsClosed(&self) -> bool {
        self.isClosed_.get()
    }

    // https://w3c.github.io/FileAPI/#dfn-close
    fn Close(&self) {
        // Step 1
        if self.isClosed_.get() {
            return;
        }

        // Step 2
        self.isClosed_.set(true);

        // TODO Step 3 if Blob URL Store is implemented

    }
}


impl BlobBinding::BlobPropertyBag {
    /// Get the normalized inner type string
    /// https://w3c.github.io/FileAPI/#dfn-type
    pub fn get_typestring(&self) -> String {
        if is_ascii_printable(&self.type_) {
            self.type_.to_lowercase()
        } else {
            "".to_string()
        }
    }
}

fn is_ascii_printable(string: &str) -> bool {
    // Step 5.1 in Sec 5.1 of File API spec
    // https://w3c.github.io/FileAPI/#constructorBlob
    string.chars().all(|c| c >= '\x20' && c <= '\x7E')
}
