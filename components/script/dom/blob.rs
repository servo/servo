/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::BlobBinding;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::UnionTypes::BlobOrString;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use net_traits::{CoreResourceMsg, IpcSend};
use net_traits::blob_url_store::{BlobBuf, get_blob_origin};
use net_traits::filemanager_thread::{FileManagerThreadMsg, ReadFileProgress, RelativePos};
use std::ascii::AsciiExt;
use std::mem;
use std::ops::Index;
use std::path::PathBuf;
use uuid::Uuid;

/// File-based blob
#[derive(JSTraceable)]
pub struct FileBlob {
    id: Uuid,
    name: Option<PathBuf>,
    cache: DomRefCell<Option<Vec<u8>>>,
    size: u64,
}


/// Different backends of Blob
#[must_root]
#[derive(JSTraceable)]
pub enum BlobImpl {
    /// File-based blob, whose content lives in the net process
    File(FileBlob),
    /// Memory-based blob, whose content lives in the script process
    Memory(Vec<u8>),
    /// Sliced blob, including parent blob reference and
    /// relative positions of current slicing range,
    /// IMPORTANT: The depth of tree is only two, i.e. the parent Blob must be
    /// either File-based or Memory-based
    Sliced(Dom<Blob>, RelativePos),
}

impl BlobImpl {
    /// Construct memory-backed BlobImpl
    #[allow(unrooted_must_root)]
    pub fn new_from_bytes(bytes: Vec<u8>) -> BlobImpl {
        BlobImpl::Memory(bytes)
    }

    /// Construct file-backed BlobImpl from File ID
    pub fn new_from_file(file_id: Uuid, name: PathBuf, size: u64) -> BlobImpl {
        BlobImpl::File(FileBlob {
            id: file_id,
            name: Some(name),
            cache: DomRefCell::new(None),
            size: size,
        })
    }
}

// https://w3c.github.io/FileAPI/#blob
#[dom_struct]
pub struct Blob {
    reflector_: Reflector,
    #[ignore_heap_size_of = "No clear owner"]
    blob_impl: DomRefCell<BlobImpl>,
    /// content-type string
    type_string: String,
}

impl Blob {
    #[allow(unrooted_must_root)]
    pub fn new(
            global: &GlobalScope, blob_impl: BlobImpl, typeString: String)
            -> DomRoot<Blob> {
        let boxed_blob = Box::new(Blob::new_inherited(blob_impl, typeString));
        reflect_dom_object(boxed_blob, global, BlobBinding::Wrap)
    }

    #[allow(unrooted_must_root)]
    pub fn new_inherited(blob_impl: BlobImpl, type_string: String) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            blob_impl: DomRefCell::new(blob_impl),
            // NOTE: Guarding the format correctness here,
            // https://w3c.github.io/FileAPI/#dfn-type
            type_string: normalize_type_string(&type_string),
        }
    }

    #[allow(unrooted_must_root)]
    fn new_sliced(parent: &Blob, rel_pos: RelativePos,
                  relative_content_type: DOMString) -> DomRoot<Blob> {
        let blob_impl = match *parent.blob_impl.borrow() {
            BlobImpl::File(_) => {
                // Create new parent node
                BlobImpl::Sliced(Dom::from_ref(parent), rel_pos)
            }
            BlobImpl::Memory(_) => {
                // Create new parent node
                BlobImpl::Sliced(Dom::from_ref(parent), rel_pos)
            }
            BlobImpl::Sliced(ref grandparent, ref old_rel_pos) => {
                // Adjust the slicing position, using same parent
                BlobImpl::Sliced(grandparent.clone(), old_rel_pos.slice_inner(&rel_pos))
            }
        };

        Blob::new(&parent.global(), blob_impl, relative_content_type.into())
    }

    // https://w3c.github.io/FileAPI/#constructorBlob
    pub fn Constructor(global: &GlobalScope,
                       blobParts: Option<Vec<BlobOrString>>,
                       blobPropertyBag: &BlobBinding::BlobPropertyBag)
                       -> Fallible<DomRoot<Blob>> {
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
                        let bytes = read_file(&self.global(), f.id.clone())?;
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

    /// Get a copy of the type_string
    pub fn type_string(&self) -> String {
        self.type_string.clone()
    }

    /// Get a FileID representing the Blob content,
    /// used by URL.createObjectURL
    pub fn get_blob_url_id(&self) -> Uuid {
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
    /// 2. File-based: If set_valid, then activate the FileID so it can serve as URL
    /// Depending on set_valid, the returned FileID can be part of
    /// valid or invalid Blob URL.
    fn promote(&self, set_valid: bool) -> Uuid {
        let mut bytes = vec![];
        let global_url = self.global().get_url();

        match *self.blob_impl.borrow_mut() {
            BlobImpl::Sliced(_, _) => {
                debug!("Sliced can't have a sliced parent");
                // Return dummy id
                return Uuid::new_v4();
            }
            BlobImpl::File(ref f) => {
                if set_valid {
                    let origin = get_blob_origin(&global_url);
                    let (tx, rx) = ipc::channel().unwrap();

                    let msg = FileManagerThreadMsg::ActivateBlobURL(f.id.clone(), tx, origin.clone());
                    self.send_to_file_manager(msg);

                    match rx.recv().unwrap() {
                        Ok(_) => return f.id.clone(),
                        // Return a dummy id on error
                        Err(_) => return Uuid::new_v4(),
                    }
                } else {
                    // no need to activate
                    return f.id.clone();
                }
            }
            BlobImpl::Memory(ref mut bytes_in) => mem::swap(bytes_in, &mut bytes),
        };

        let origin = get_blob_origin(&global_url);

        let blob_buf = BlobBuf {
            filename: None,
            type_string: self.type_string.clone(),
            size: bytes.len() as u64,
            bytes: bytes.to_vec(),
        };

        let (tx, rx) = ipc::channel().unwrap();
        let msg = FileManagerThreadMsg::PromoteMemory(blob_buf, set_valid, tx, origin.clone());
        self.send_to_file_manager(msg);

        match rx.recv().unwrap() {
            Ok(id) => {
                *self.blob_impl.borrow_mut() = BlobImpl::File(FileBlob {
                    id: id.clone(),
                    name: None,
                    cache: DomRefCell::new(Some(bytes.to_vec())),
                    size: bytes.len() as u64,
                });
                id
            }
            // Dummy id
            Err(_) => Uuid::new_v4(),
        }
    }

    /// Get a FileID representing sliced parent-blob content
    fn create_sliced_url_id(&self, parent_id: &Uuid,
                            rel_pos: &RelativePos, parent_len: u64) -> Uuid {
        let origin = get_blob_origin(&self.global().get_url());

        let (tx, rx) = ipc::channel().unwrap();
        let msg = FileManagerThreadMsg::AddSlicedURLEntry(parent_id.clone(),
                                                          rel_pos.clone(),
                                                          tx, origin.clone());
        self.send_to_file_manager(msg);
        match rx.recv().expect("File manager thread is down") {
            Ok(new_id) => {
                *self.blob_impl.borrow_mut() = BlobImpl::File(FileBlob {
                    id: new_id.clone(),
                    name: None,
                    cache: DomRefCell::new(None),
                    size: rel_pos.to_abs_range(parent_len as usize).len() as u64,
                });

                // Return the indirect id reference
                new_id
            }
            Err(_) => {
                // Return dummy id
                Uuid::new_v4()
            }
        }
    }

    /// Cleanups at the time of destruction/closing
    fn clean_up_file_resource(&self) {
        if let BlobImpl::File(ref f) = *self.blob_impl.borrow() {
            let origin = get_blob_origin(&self.global().get_url());

            let (tx, rx) = ipc::channel().unwrap();

            let msg = FileManagerThreadMsg::DecRef(f.id.clone(), origin, tx);
            self.send_to_file_manager(msg);
            let _ = rx.recv().unwrap();
        }
    }

    fn send_to_file_manager(&self, msg: FileManagerThreadMsg) {
        let global = self.global();
        let resource_threads = global.resource_threads();
        let _ = resource_threads.send(CoreResourceMsg::ToFileManager(msg));
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        self.clean_up_file_resource();
    }
}

fn read_file(global: &GlobalScope, id: Uuid) -> Result<Vec<u8>, ()> {
    let resource_threads = global.resource_threads();
    let (chan, recv) = ipc::channel().map_err(|_|())?;
    let origin = get_blob_origin(&global.get_url());
    let check_url_validity = false;
    let msg = FileManagerThreadMsg::ReadFile(chan, id, check_url_validity, origin);
    let _ = resource_threads.send(CoreResourceMsg::ToFileManager(msg));

    let mut bytes = vec![];

    loop {
        match recv.recv().unwrap() {
            Ok(ReadFileProgress::Meta(mut blob_buf)) => {
                bytes.append(&mut blob_buf.bytes);
            }
            Ok(ReadFileProgress::Partial(mut bytes_in)) => {
                bytes.append(&mut bytes_in);
            }
            Ok(ReadFileProgress::EOF) => {
                return Ok(bytes);
            }
            Err(_) => return Err(()),
        }
    }
}

/// Extract bytes from BlobParts, used by Blob and File constructor
/// https://w3c.github.io/FileAPI/#constructorBlob
pub fn blob_parts_to_bytes(blobparts: Vec<BlobOrString>) -> Result<Vec<u8>, ()> {
    let mut ret = vec![];

    for blobpart in &blobparts {
        match blobpart {
            &BlobOrString::String(ref s) => {
                ret.extend(s.as_bytes());
            },
            &BlobOrString::Blob(ref b) => {
                let bytes = b.get_bytes().unwrap_or(vec![]);
                ret.extend(bytes);
            },
        }
    }

    Ok(ret)
}

impl BlobMethods for Blob {
    // https://w3c.github.io/FileAPI/#dfn-size
    fn Size(&self) -> u64 {
        match *self.blob_impl.borrow() {
           BlobImpl::File(ref f) => f.size,
           BlobImpl::Memory(ref v) => v.len() as u64,
           BlobImpl::Sliced(ref parent, ref rel_pos) =>
               rel_pos.to_abs_range(parent.Size() as usize).len() as u64,
        }
    }

    // https://w3c.github.io/FileAPI/#dfn-type
    fn Type(&self) -> DOMString {
        DOMString::from(self.type_string.clone())
    }

    // https://w3c.github.io/FileAPI/#slice-method-algo
    fn Slice(&self,
             start: Option<i64>,
             end: Option<i64>,
             content_type: Option<DOMString>)
             -> DomRoot<Blob> {
        let rel_pos = RelativePos::from_opts(start, end);
        Blob::new_sliced(self, rel_pos, content_type.unwrap_or(DOMString::from("")))
    }
}

/// Get the normalized, MIME-parsable type string
/// https://w3c.github.io/FileAPI/#dfn-type
/// XXX: We will relax the restriction here,
/// since the spec has some problem over this part.
/// see https://github.com/w3c/FileAPI/issues/43
fn normalize_type_string(s: &str) -> String {
    if is_ascii_printable(s) {
        let s_lower = s.to_ascii_lowercase();
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
