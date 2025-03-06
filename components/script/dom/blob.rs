/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::ptr;
use std::rc::Rc;

use base::id::{BlobId, BlobIndex, PipelineNamespaceId};
use dom_struct::dom_struct;
use encoding_rs::UTF_8;
use js::jsapi::JSObject;
use js::rust::HandleObject;
use js::typedarray::Uint8;
use net_traits::filemanager_thread::RelativePos;
use script_traits::serializable::BlobImpl;
use uuid::Uuid;

use crate::body::{FetchedData, run_array_buffer_data_algorithm};
use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::BlobBinding;
use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferOrArrayBufferViewOrBlobOrString;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::{Serializable, StorageKey};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone::{StructuredDataReader, StructuredDataWriter};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_runtime::CanGc;

// https://w3c.github.io/FileAPI/#blob
#[dom_struct]
pub(crate) struct Blob {
    reflector_: Reflector,
    #[no_trace]
    blob_id: BlobId,
}

impl Blob {
    pub(crate) fn new(global: &GlobalScope, blob_impl: BlobImpl, can_gc: CanGc) -> DomRoot<Blob> {
        Self::new_with_proto(global, None, blob_impl, can_gc)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        blob_impl: BlobImpl,
        can_gc: CanGc,
    ) -> DomRoot<Blob> {
        let dom_blob = reflect_dom_object_with_proto(
            Box::new(Blob::new_inherited(&blob_impl)),
            global,
            proto,
            can_gc,
        );
        global.track_blob(&dom_blob, blob_impl);
        dom_blob
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(blob_impl: &BlobImpl) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            blob_id: blob_impl.blob_id(),
        }
    }

    /// Get a slice to inner data, this might incur synchronous read and caching
    pub(crate) fn get_bytes(&self) -> Result<Vec<u8>, ()> {
        self.global().get_blob_bytes(&self.blob_id)
    }

    /// Get a copy of the type_string
    pub(crate) fn type_string(&self) -> String {
        self.global().get_blob_type_string(&self.blob_id)
    }

    /// Get a FileID representing the Blob content,
    /// used by URL.createObjectURL
    pub(crate) fn get_blob_url_id(&self) -> Uuid {
        self.global().get_blob_url_id(&self.blob_id)
    }

    /// <https://w3c.github.io/FileAPI/#blob-get-stream>
    pub(crate) fn get_stream(&self, can_gc: CanGc) -> Fallible<DomRoot<ReadableStream>> {
        self.global().get_blob_stream(&self.blob_id, can_gc)
    }
}

impl Serializable for Blob {
    /// <https://w3c.github.io/FileAPI/#ref-for-serialization-steps>
    fn serialize(&self, sc_writer: &mut StructuredDataWriter) -> Result<StorageKey, ()> {
        let blob_id = self.blob_id;

        // 1. Get a clone of the blob impl.
        let blob_impl = self.global().serialize_blob(&blob_id);

        // We clone the data, but the clone gets its own Id.
        let new_blob_id = blob_impl.blob_id();

        // 2. Store the object at a given key.
        let blobs = sc_writer.blobs.get_or_insert_with(HashMap::new);
        blobs.insert(new_blob_id, blob_impl);

        let PipelineNamespaceId(name_space) = new_blob_id.namespace_id;
        let BlobIndex(index) = new_blob_id.index;
        let index = index.get();

        let name_space = name_space.to_ne_bytes();
        let index = index.to_ne_bytes();

        let storage_key = StorageKey {
            index: u32::from_ne_bytes(index),
            name_space: u32::from_ne_bytes(name_space),
        };

        // 3. Return the storage key.
        Ok(storage_key)
    }

    /// <https://w3c.github.io/FileAPI/#ref-for-deserialization-steps>
    fn deserialize(
        owner: &GlobalScope,
        sc_reader: &mut StructuredDataReader,
        storage_key: StorageKey,
        can_gc: CanGc,
    ) -> Result<DomRoot<Self>, ()> {
        // 1. Re-build the key for the storage location
        // of the serialized object.
        let namespace_id = PipelineNamespaceId(storage_key.name_space);
        let index =
            BlobIndex(NonZeroU32::new(storage_key.index).expect("Deserialized blob index is zero"));

        let id = BlobId {
            namespace_id,
            index,
        };

        // 2. Get the transferred object from its storage, using the key.
        let blob_impls_map = sc_reader
            .blob_impls
            .as_mut()
            .expect("The SC holder does not have any blob impls");
        let blob_impl = blob_impls_map
            .remove(&id)
            .expect("No blob to be deserialized found.");
        if blob_impls_map.is_empty() {
            sc_reader.blob_impls = None;
        }

        let deserialized_blob = Blob::new(owner, blob_impl, can_gc);
        Ok(deserialized_blob)
    }

    fn destination(data: &mut StructuredDataReader) -> &mut Option<HashMap<StorageKey, DomRoot<Self>>> {
        &mut data.blobs
    }
}

/// Extract bytes from BlobParts, used by Blob and File constructor
/// <https://w3c.github.io/FileAPI/#constructorBlob>
#[allow(unsafe_code)]
pub(crate) fn blob_parts_to_bytes(
    mut blobparts: Vec<ArrayBufferOrArrayBufferViewOrBlobOrString>,
) -> Result<Vec<u8>, ()> {
    let mut ret = vec![];
    for blobpart in &mut blobparts {
        match blobpart {
            ArrayBufferOrArrayBufferViewOrBlobOrString::String(s) => {
                ret.extend(s.as_bytes());
            },
            ArrayBufferOrArrayBufferViewOrBlobOrString::Blob(b) => {
                let bytes = b.get_bytes().unwrap_or(vec![]);
                ret.extend(bytes);
            },
            ArrayBufferOrArrayBufferViewOrBlobOrString::ArrayBuffer(a) => unsafe {
                let bytes = a.as_slice();
                ret.extend(bytes);
            },
            ArrayBufferOrArrayBufferViewOrBlobOrString::ArrayBufferView(a) => unsafe {
                let bytes = a.as_slice();
                ret.extend(bytes);
            },
        }
    }

    Ok(ret)
}

impl BlobMethods<crate::DomTypeHolder> for Blob {
    // https://w3c.github.io/FileAPI/#constructorBlob
    #[allow(non_snake_case)]
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        blobParts: Option<Vec<ArrayBufferOrArrayBufferViewOrBlobOrString>>,
        blobPropertyBag: &BlobBinding::BlobPropertyBag,
    ) -> Fallible<DomRoot<Blob>> {
        let bytes: Vec<u8> = match blobParts {
            None => Vec::new(),
            Some(blobparts) => match blob_parts_to_bytes(blobparts) {
                Ok(bytes) => bytes,
                Err(_) => return Err(Error::InvalidCharacter),
            },
        };

        let type_string = normalize_type_string(blobPropertyBag.type_.as_ref());
        let blob_impl = BlobImpl::new_from_bytes(bytes, type_string);

        Ok(Blob::new_with_proto(global, proto, blob_impl, can_gc))
    }

    // https://w3c.github.io/FileAPI/#dfn-size
    fn Size(&self) -> u64 {
        self.global().get_blob_size(&self.blob_id)
    }

    // https://w3c.github.io/FileAPI/#dfn-type
    fn Type(&self) -> DOMString {
        DOMString::from(self.type_string())
    }

    // <https://w3c.github.io/FileAPI/#blob-get-stream>
    fn Stream(&self, can_gc: CanGc) -> Fallible<DomRoot<ReadableStream>> {
        self.get_stream(can_gc)
    }

    // https://w3c.github.io/FileAPI/#slice-method-algo
    fn Slice(
        &self,
        start: Option<i64>,
        end: Option<i64>,
        content_type: Option<DOMString>,
        can_gc: CanGc,
    ) -> DomRoot<Blob> {
        let type_string =
            normalize_type_string(content_type.unwrap_or(DOMString::from("")).as_ref());
        let rel_pos = RelativePos::from_opts(start, end);
        let blob_impl = BlobImpl::new_sliced(rel_pos, self.blob_id, type_string);
        Blob::new(&self.global(), blob_impl, can_gc)
    }

    // https://w3c.github.io/FileAPI/#text-method-algo
    fn Text(&self, can_gc: CanGc) -> Rc<Promise> {
        let global = self.global();
        let in_realm_proof = AlreadyInRealm::assert();
        let p = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);
        let id = self.get_blob_url_id();
        global.read_file_async(
            id,
            p.clone(),
            Box::new(|promise, bytes| match bytes {
                Ok(b) => {
                    let (text, _, _) = UTF_8.decode(&b);
                    let text = DOMString::from(text);
                    promise.resolve_native(&text, CanGc::note());
                },
                Err(e) => {
                    promise.reject_error(e, CanGc::note());
                },
            }),
        );
        p
    }

    // https://w3c.github.io/FileAPI/#arraybuffer-method-algo
    fn ArrayBuffer(&self, can_gc: CanGc) -> Rc<Promise> {
        let global = self.global();
        let in_realm_proof = AlreadyInRealm::assert();
        let p = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);

        let id = self.get_blob_url_id();

        global.read_file_async(
            id,
            p.clone(),
            Box::new(|promise, bytes| {
                match bytes {
                    Ok(b) => {
                        let cx = GlobalScope::get_cx();
                        let result = run_array_buffer_data_algorithm(cx, b, CanGc::note());

                        match result {
                            Ok(FetchedData::ArrayBuffer(a)) => {
                                promise.resolve_native(&a, CanGc::note())
                            },
                            Err(e) => promise.reject_error(e, CanGc::note()),
                            _ => panic!("Unexpected result from run_array_buffer_data_algorithm"),
                        }
                    },
                    Err(e) => promise.reject_error(e, CanGc::note()),
                };
            }),
        );
        p
    }

    /// <https://w3c.github.io/FileAPI/#dom-blob-bytes>
    fn Bytes(&self, in_realm: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let cx = GlobalScope::get_cx();
        let global = GlobalScope::from_safe_context(cx, in_realm);
        let p = Promise::new_in_current_realm(in_realm, can_gc);

        // 1. Let stream be the result of calling get stream on this.
        let stream = self.get_stream(can_gc);

        // 2. Let reader be the result of getting a reader from stream.
        //    If that threw an exception, return a new promise rejected with that exception.
        let reader = match stream.and_then(|s| s.acquire_default_reader(can_gc)) {
            Ok(r) => r,
            Err(e) => {
                p.reject_error(e, can_gc);
                return p;
            },
        };

        // 3. Let promise be the result of reading all bytes from stream with reader.
        let p_success = p.clone();
        let p_failure = p.clone();
        reader.read_all_bytes(
            cx,
            &global,
            Rc::new(move |bytes| {
                rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
                let arr = create_buffer_source::<Uint8>(cx, bytes, js_object.handle_mut(), can_gc)
                    .expect("Converting input to uint8 array should never fail");
                p_success.resolve_native(&arr, can_gc);
            }),
            Rc::new(move |cx, v| {
                p_failure.reject(cx, v, can_gc);
            }),
            in_realm,
            can_gc,
        );
        p
    }
}

/// Get the normalized, MIME-parsable type string
/// <https://w3c.github.io/FileAPI/#dfn-type>
/// XXX: We will relax the restriction here,
/// since the spec has some problem over this part.
/// see <https://github.com/w3c/FileAPI/issues/43>
pub(crate) fn normalize_type_string(s: &str) -> String {
    if is_ascii_printable(s) {
        s.to_ascii_lowercase()
        // match s_lower.parse() as Result<Mime, ()> {
        // Ok(_) => s_lower,
        // Err(_) => "".to_string()
    } else {
        "".to_string()
    }
}

fn is_ascii_printable(string: &str) -> bool {
    // Step 5.1 in Sec 5.1 of File API spec
    // <https://w3c.github.io/FileAPI/#constructorBlob>
    string.chars().all(|c| ('\x20'..='\x7E').contains(&c))
}
