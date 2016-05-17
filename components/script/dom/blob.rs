/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BlobBinding;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::UnionTypes::BlobOrString;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use encoding::all::UTF_8;
use encoding::types::{EncoderTrap, Encoding};
use ipc_channel::ipc::{self, IpcSender};
use net_traits::filemanager_thread::FileManagerThreadMsg;
use num_traits::ToPrimitive;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::cmp::{max, min};
use std::sync::Arc;
use util::str::DOMString;
use uuid::Uuid;

#[derive(Clone, JSTraceable)]
pub struct DataSlice {
    bytes: Arc<Vec<u8>>,
    bytes_start: usize,
    bytes_end: usize
}

impl DataSlice {
    /// Construct DataSlice from reference counted bytes
    pub fn new(bytes: Arc<Vec<u8>>, start: Option<i64>, end: Option<i64>) -> DataSlice {
        let size = bytes.len() as i64;
        let relativeStart: i64 = match start {
            None => 0,
            Some(start) => {
                if start < 0 {
                    max(size + start, 0)
                } else {
                    min(start, size)
                }
            }
        };
        let relativeEnd: i64 = match end {
            None => size,
            Some(end) => {
                if end < 0 {
                    max(size + end, 0)
                } else {
                    min(end, size)
                }
            }
        };

        let span: i64 = max(relativeEnd - relativeStart, 0);
        let start = relativeStart.to_usize().unwrap();
        let end = (relativeStart + span).to_usize().unwrap();

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
}


#[derive(Clone, JSTraceable)]
pub enum BlobImpl {
    /// File-based, cached backend
    File(Uuid, DOMRefCell<Option<DataSlice>>),
    /// Memory-based backend
    Memory(DataSlice),
}

impl BlobImpl {
    /// Construct memory-backed BlobImpl from DataSlice
    pub fn new_from_slice(slice: DataSlice) -> BlobImpl {
        BlobImpl::Memory(slice)
    }

    /// Construct file-backed BlobImpl from File ID
    pub fn new_from_file(file_id: Uuid) -> BlobImpl {
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
    blob_impl: BlobImpl,
    typeString: String,
    isClosed_: Cell<bool>,
}

impl Blob {

    pub fn new(global: GlobalRef, blob_impl: BlobImpl, typeString: &str) -> Root<Blob> {
        let boxed_blob = box Blob::new_inherited(blob_impl, typeString);
        reflect_dom_object(boxed_blob, global, BlobBinding::Wrap)
    }

    pub fn new_inherited(blob_impl: BlobImpl, typeString: &str) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            blob_impl: blob_impl,
            typeString: typeString.to_owned(),
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
            Some(blobparts) => blob_parts_to_bytes(blobparts),
        };

        let slice = DataSlice::from_bytes(bytes);
        Ok(Blob::new(global, BlobImpl::new_from_slice(slice), &blobPropertyBag.get_typestring()))
    }

    /// Get a slice to inner data, this might incur synchronous read and caching
    pub fn get_slice(&self) -> DataSlice {
        match self.blob_impl {
            BlobImpl::File(ref id, ref slice) => {
                match *slice.borrow() {
                    Some(ref s) => s.clone(),
                    None => {
                        let global = self.global();
                        let s = read_file(global.r(), id.clone());
                        *slice.borrow_mut() = Some(s.clone()); // Cached
                        s
                    }
                }
            }
            BlobImpl::Memory(ref s) => s.clone()
        }
    }
}

fn read_file(global: GlobalRef, id: Uuid) -> DataSlice {
    // FIXME: After resource thread #11189 PR is landed
    // let global = global.filemanager_thread();

    let file_manager: IpcSender<FileManagerThreadMsg> = unimplemented!();
    let (chan, recv) = ipc::channel().unwrap();
    let _ = file_manager.send(FileManagerThreadMsg::ReadFile(chan, id));
    let result = recv.recv().unwrap();
    let bytes = result.unwrap();
    let slice = DataSlice::from_bytes(bytes);

    slice
}

/// Extract bytes from BlobParts, used by Blob and File constructor
/// https://w3c.github.io/FileAPI/#constructorBlob
pub fn blob_parts_to_bytes(blobparts: Vec<BlobOrString>) -> Vec<u8> {
    blobparts.iter().flat_map(|blobpart| {
            match blobpart {
                &BlobOrString::String(ref s) => {
                    UTF_8.encode(s, EncoderTrap::Replace).unwrap()
                },
                &BlobOrString::Blob(ref b) => {
                    b.get_slice().bytes.to_vec()
                },
            }
        }).collect::<Vec<u8>>()
}

impl BlobMethods for Blob {
    // https://w3c.github.io/FileAPI/#dfn-size
    fn Size(&self) -> u64 {
        self.get_slice().size()
    }

    // https://w3c.github.io/FileAPI/#dfn-type
    fn Type(&self) -> DOMString {
        DOMString::from(self.typeString.clone())
    }

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
        let bytes = self.get_slice().bytes.clone();
        let slice = DataSlice::new(bytes, start, end);
        Blob::new(global.r(), BlobImpl::new_from_slice(slice), &relativeContentType)
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
