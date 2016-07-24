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
use net_traits::IpcSend;
use net_traits::blob_url_store::{BlobBuf, get_blob_origin};
use net_traits::filemanager_thread::{FileManagerThreadMsg, SelectedFileId, RelativePos};
use std::cell::Cell;
use std::ops::Index;
use std::path::PathBuf;
use uuid::Uuid;

/// File-based blob
#[derive(JSTraceable)]
pub struct FileBlob {
    id: SelectedFileId,
    name: Option<PathBuf>,
    cache: DOMRefCell<Option<Vec<u8>>>,
    size: u64,
}


/// Blob backend implementation
#[must_root]
#[derive(JSTraceable)]
pub enum BlobImpl {
    /// File-based blob
    File(FileBlob),
    /// Memory-based blob
    Memory(Vec<u8>),
    /// Sliced blob, including parent blob and
    /// relative positions representing current slicing range,
    /// it is leaf of a two-layer fat tree
    Sliced(JS<Blob>, RelativePos),
}

impl BlobImpl {
    /// Construct memory-backed BlobImpl
    #[allow(unrooted_must_root)]
    pub fn new_from_bytes(bytes: Vec<u8>) -> BlobImpl {
        BlobImpl::Memory(bytes)
    }

    /// Construct file-backed BlobImpl from File ID
    pub fn new_from_file(file_id: SelectedFileId, name: PathBuf, size: u64) -> BlobImpl {
        BlobImpl::File(FileBlob {
            id: file_id,
            name: Some(name),
            cache: DOMRefCell::new(None),
            size: size,
        })
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
            // NOTE: Guarding the format correctness here,
            // https://w3c.github.io/FileAPI/#dfn-type
            typeString: normalize_type_string(&typeString),
            isClosed_: Cell::new(false),
        }
    }

    #[allow(unrooted_must_root)]
    fn new_sliced(parent: &Blob, rel_pos: RelativePos,
                  relativeContentType: DOMString) -> Root<Blob> {
        let global = parent.global();
        let blob_impl = match *parent.blob_impl.borrow() {
            BlobImpl::File(_) => {
                // Create new parent node
                BlobImpl::Sliced(JS::from_ref(parent), rel_pos)
            }
            BlobImpl::Memory(_) => {
                // Create new parent node
                BlobImpl::Sliced(JS::from_ref(parent), rel_pos)
            }
            BlobImpl::Sliced(ref grandparent, ref old_rel_pos) => {
                // Adjust the slicing position, using same parent
                BlobImpl::Sliced(grandparent.clone(), old_rel_pos.slice_inner(&rel_pos))
            }
        };

        Blob::new(global.r(), blob_impl, relativeContentType.into())
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

        Ok(Blob::new(global, BlobImpl::new_from_bytes(bytes), blobPropertyBag.type_.to_string()))
    }

    /// Get a slice to inner data, this might incur synchronous read and caching
    pub fn get_bytes(&self) -> Result<Vec<u8>, ()> {
        match *self.blob_impl.borrow() {
            BlobImpl::File(ref f) => {
                let (buffer, is_new_buffer) = match *f.cache.borrow() {
                    Some(ref bytes) => (bytes.clone(), false),
                    None => {
                        let global = self.global();
                        let bytes = read_file(global.r(), f.id.clone())?;
                        (bytes, true)
                    }
                };

                // Cache
                if is_new_buffer {
                    *f.cache.borrow_mut() = Some(buffer.clone());
                }

                Ok(buffer)
            }
            BlobImpl::Memory(ref s) => Ok(s.clone()),
            BlobImpl::Sliced(ref parent, ref rel_pos) => {
                parent.get_bytes().map(|v| {
                    let range = rel_pos.to_abs_range(v.len());
                    v.index(range).to_vec()
                })
            }
        }
    }

    /// Get a FileID representing the Blob content,
    /// used by URL.createObjectURL
    pub fn get_blob_url_id(&self) -> SelectedFileId {
        let opt_sliced_parent = match *self.blob_impl.borrow() {
            BlobImpl::Sliced(ref parent, ref rel_pos) => {
                Some((parent.promote(/* set_valid is */ false), rel_pos.clone(), parent.Size()))
            }
            _ => None
        };

        match opt_sliced_parent {
            Some((parent_id, rel_pos, size)) => self.create_sliced_url_id(&parent_id, &rel_pos, size),
            None => self.promote(/* set_valid is */ true),
        }
    }

    /// Promote non-Slice blob:
    /// 1. Memory-based: The bytes in data slice will be transferred to file manager thread.
    /// 2. File-based: Activation
    /// Depending on set_valid, the returned FileID can be part of
    /// valid or invalid Blob URL.
    fn promote(&self, set_valid: bool) -> SelectedFileId {
        let bytes = match *self.blob_impl.borrow() {
            BlobImpl::Sliced(_, _) => {
                debug!("Sliced can't have a sliced parent");
                // Return dummy id
                return SelectedFileId(Uuid::new_v4().simple().to_string());
            }
            BlobImpl::File(ref f) => {
                let global = self.global();
                let origin = get_blob_origin(&global.r().get_url());
                let filemanager = global.r().resource_threads().sender();
                let (tx, rx) = ipc::channel().unwrap();

                let _ = filemanager.send(FileManagerThreadMsg::ActivateBlobURL(f.id.clone(), tx, origin.clone()));

                match rx.recv().unwrap() {
                    Ok(_) => return f.id.clone(),
                    Err(_) => return SelectedFileId(Uuid::new_v4().simple().to_string()) // Return a dummy id on error
                }
            }
            BlobImpl::Memory(ref bytes) => bytes.clone(),
        };

        let global = self.global();
        let origin = get_blob_origin(&global.r().get_url());
        let filemanager = global.r().resource_threads().sender();

        let blob_buf = BlobBuf {
            filename: None,
            type_string: self.typeString.clone(),
            size: bytes.len() as u64,
            bytes: bytes.to_vec(),
        };

        let (tx, rx) = ipc::channel().unwrap();
        let _ = filemanager.send(FileManagerThreadMsg::PromoteMemory(blob_buf, set_valid, tx, origin.clone()));

        match rx.recv().unwrap() {
            Ok(id) => {
                let id = SelectedFileId(id.0);
                *self.blob_impl.borrow_mut() = BlobImpl::File(FileBlob {
                    id: id.clone(),
                    name: None,
                    cache: DOMRefCell::new(Some(bytes.to_vec())),
                    size: bytes.len() as u64,
                });
                id
            }
            // Dummy id
            Err(_) => SelectedFileId(Uuid::new_v4().simple().to_string()),
        }
    }

    /// Get a FileID representing sliced parent-blob content
    fn create_sliced_url_id(&self, parent_id: &SelectedFileId,
                            rel_pos: &RelativePos, parent_len: u64) -> SelectedFileId {
        let global = self.global();

        let origin = get_blob_origin(&global.r().get_url());

        let filemanager = global.r().resource_threads().sender();
        let (tx, rx) = ipc::channel().unwrap();
        let msg = FileManagerThreadMsg::AddSlicedURLEntry(parent_id.clone(),
                                                          rel_pos.clone(),
                                                          tx, origin.clone());
        let _ = filemanager.send(msg);
        let new_id = rx.recv().unwrap().unwrap(); // XX: unwrap
        let new_id = SelectedFileId(new_id.0);

        *self.blob_impl.borrow_mut() = BlobImpl::File(FileBlob {
            id: new_id.clone(),
            name: None,
            cache: DOMRefCell::new(None),
            size: rel_pos.to_abs_range(parent_len as usize).len() as u64,
        });

        // Return the indirect id reference
        new_id
    }

    /// Cleanups at the time of destruction/closing
    fn clean_up_file_resource(&self) {
        if let BlobImpl::File(ref f) = *self.blob_impl.borrow() {
            let global = self.global();
            let origin = get_blob_origin(&global.r().get_url());

            let filemanager = global.r().resource_threads().sender();
            let (tx, rx) = ipc::channel().unwrap();

            let msg = FileManagerThreadMsg::DecRef(f.id.clone(), origin, tx);
            let _ = filemanager.send(msg);
            let _ = rx.recv().unwrap();
        }
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        if !self.IsClosed() {
            self.clean_up_file_resource();
        }
    }
}

fn read_file(global: GlobalRef, id: SelectedFileId) -> Result<Vec<u8>, ()> {
    let file_manager = global.filemanager_thread();
    let (chan, recv) = ipc::channel().map_err(|_|())?;
    let origin = get_blob_origin(&global.get_url());
    let check_url_validity = false;
    let msg = FileManagerThreadMsg::ReadFile(chan, id, check_url_validity, origin);
    let _ = file_manager.send(msg);

    match recv.recv().unwrap() {
        Ok(blob_buf) => Ok(blob_buf.bytes),
        Err(_) => Err(()),
    }
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
                let mut bytes = b.get_bytes().unwrap_or(vec![]);
                ret.append(&mut bytes);
            },
        }
    }

    Ok(ret)
}

impl BlobMethods for Blob {
    // https://w3c.github.io/FileAPI/#dfn-size
    fn Size(&self) -> u64 {
        // XXX: This will incur reading if file-based
        match self.get_bytes() {
            Ok(s) => s.len() as u64,
            _ => 0,
        }
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
        let rel_pos = RelativePos::from_opts(start, end);
        Blob::new_sliced(self, rel_pos, contentType.unwrap_or(DOMString::from("")))
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

        // Step 3
        self.clean_up_file_resource();
    }
}

/// Get the normalized, MIME-parsable type string
/// https://w3c.github.io/FileAPI/#dfn-type
/// XXX: We will relax the restriction here,
/// since the spec has some problem over this part.
/// see https://github.com/w3c/FileAPI/issues/43
fn normalize_type_string(s: &str) -> String {
    if is_ascii_printable(s) {
        let s_lower = s.to_lowercase();
        // match s_lower.parse() as Result<Mime, ()> {
            // Ok(_) => s_lower,
            // Err(_) => "".to_string()
        s_lower
    } else {
        "".to_string()
    }
}

fn is_ascii_printable(string: &str) -> bool {
    // Step 5.1 in Sec 5.1 of File API spec
    // https://w3c.github.io/FileAPI/#constructorBlob
    string.chars().all(|c| c >= '\x20' && c <= '\x7E')
}
