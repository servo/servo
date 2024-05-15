/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::ptr::NonNull;
use std::rc::Rc;

use base::id::{BlobId, BlobIndex, PipelineNamespaceId};
use dom_struct::dom_struct;
use encoding_rs::UTF_8;
use js::jsapi::JSObject;
use js::rust::HandleObject;
use net_traits::filemanager_thread::RelativePos;
use script_traits::serializable::BlobImpl;
use uuid::Uuid;

use crate::body::{run_array_buffer_data_algorithm, FetchedData};
use crate::dom::bindings::codegen::Bindings::BlobBinding;
use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferOrArrayBufferViewOrBlobOrString;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::{Serializable, StorageKey};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone::StructuredDataHolder;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_runtime::JSContext;

// https://w3c.github.io/FileAPI/#blob
#[dom_struct]
pub struct Blob {
    reflector_: Reflector,
    #[no_trace]
    blob_id: BlobId,
}

impl Blob {
    pub fn new(global: &GlobalScope, blob_impl: BlobImpl) -> DomRoot<Blob> {
        Self::new_with_proto(global, None, blob_impl)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        blob_impl: BlobImpl,
    ) -> DomRoot<Blob> {
        let dom_blob =
            reflect_dom_object_with_proto(Box::new(Blob::new_inherited(&blob_impl)), global, proto);
        global.track_blob(&dom_blob, blob_impl);
        dom_blob
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new_inherited(blob_impl: &BlobImpl) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            blob_id: blob_impl.blob_id(),
        }
    }

    // https://w3c.github.io/FileAPI/#constructorBlob
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
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

        Ok(Blob::new_with_proto(global, proto, blob_impl))
    }

    /// Get a slice to inner data, this might incur synchronous read and caching
    pub fn get_bytes(&self) -> Result<Vec<u8>, ()> {
        self.global().get_blob_bytes(&self.blob_id)
    }

    /// Get a copy of the type_string
    pub fn type_string(&self) -> String {
        self.global().get_blob_type_string(&self.blob_id)
    }

    /// Get a FileID representing the Blob content,
    /// used by URL.createObjectURL
    pub fn get_blob_url_id(&self) -> Uuid {
        self.global().get_blob_url_id(&self.blob_id)
    }

    /// <https://w3c.github.io/FileAPI/#blob-get-stream>
    pub fn get_stream(&self) -> DomRoot<ReadableStream> {
        self.global().get_blob_stream(&self.blob_id)
    }
}

impl Serializable for Blob {
    /// <https://w3c.github.io/FileAPI/#ref-for-serialization-steps>
    fn serialize(&self, sc_holder: &mut StructuredDataHolder) -> Result<StorageKey, ()> {
        let blob_impls = match sc_holder {
            StructuredDataHolder::Write { blobs, .. } => blobs,
            _ => panic!("Unexpected variant of StructuredDataHolder"),
        };

        let blob_id = self.blob_id;

        // 1. Get a clone of the blob impl.
        let blob_impl = self.global().serialize_blob(&blob_id);

        // We clone the data, but the clone gets its own Id.
        let new_blob_id = blob_impl.blob_id();

        // 2. Store the object at a given key.
        let blobs = blob_impls.get_or_insert_with(HashMap::new);
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
        sc_holder: &mut StructuredDataHolder,
        storage_key: StorageKey,
    ) -> Result<(), ()> {
        // 1. Re-build the key for the storage location
        // of the serialized object.
        let namespace_id = PipelineNamespaceId(storage_key.name_space);
        let index =
            BlobIndex(NonZeroU32::new(storage_key.index).expect("Deserialized blob index is zero"));

        let id = BlobId {
            namespace_id,
            index,
        };

        let (blobs, blob_impls) = match sc_holder {
            StructuredDataHolder::Read {
                blobs, blob_impls, ..
            } => (blobs, blob_impls),
            _ => panic!("Unexpected variant of StructuredDataHolder"),
        };

        // 2. Get the transferred object from its storage, using the key.
        let blob_impls_map = blob_impls
            .as_mut()
            .expect("The SC holder does not have any blob impls");
        let blob_impl = blob_impls_map
            .remove(&id)
            .expect("No blob to be deserialized found.");
        if blob_impls_map.is_empty() {
            *blob_impls = None;
        }

        let deserialized_blob = Blob::new(owner, blob_impl);

        let blobs = blobs.get_or_insert_with(HashMap::new);
        blobs.insert(storage_key, deserialized_blob);

        Ok(())
    }
}

/// Extract bytes from BlobParts, used by Blob and File constructor
/// <https://w3c.github.io/FileAPI/#constructorBlob>
#[allow(unsafe_code)]
pub fn blob_parts_to_bytes(
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

impl BlobMethods for Blob {
    // https://w3c.github.io/FileAPI/#dfn-size
    fn Size(&self) -> u64 {
        self.global().get_blob_size(&self.blob_id)
    }

    // https://w3c.github.io/FileAPI/#dfn-type
    fn Type(&self) -> DOMString {
        DOMString::from(self.type_string())
    }

    // <https://w3c.github.io/FileAPI/#blob-get-stream>
    fn Stream(&self, _cx: JSContext) -> NonNull<JSObject> {
        self.get_stream().get_js_stream()
    }

    // https://w3c.github.io/FileAPI/#slice-method-algo
    fn Slice(
        &self,
        start: Option<i64>,
        end: Option<i64>,
        content_type: Option<DOMString>,
    ) -> DomRoot<Blob> {
        let type_string =
            normalize_type_string(content_type.unwrap_or(DOMString::from("")).as_ref());
        let rel_pos = RelativePos::from_opts(start, end);
        let blob_impl = BlobImpl::new_sliced(rel_pos, self.blob_id, type_string);
        Blob::new(&self.global(), blob_impl)
    }

    // https://w3c.github.io/FileAPI/#text-method-algo
    fn Text(&self) -> Rc<Promise> {
        let global = self.global();
        let in_realm_proof = AlreadyInRealm::assert();
        let p = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof));
        let id = self.get_blob_url_id();
        global.read_file_async(
            id,
            p.clone(),
            Box::new(|promise, bytes| match bytes {
                Ok(b) => {
                    let (text, _, _) = UTF_8.decode(&b);
                    let text = DOMString::from(text);
                    promise.resolve_native(&text);
                },
                Err(e) => {
                    promise.reject_error(e);
                },
            }),
        );
        p
    }

    // https://w3c.github.io/FileAPI/#arraybuffer-method-algo
    fn ArrayBuffer(&self) -> Rc<Promise> {
        let global = self.global();
        let in_realm_proof = AlreadyInRealm::assert();
        let p = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof));

        let id = self.get_blob_url_id();

        global.read_file_async(
            id,
            p.clone(),
            Box::new(|promise, bytes| {
                match bytes {
                    Ok(b) => {
                        let cx = GlobalScope::get_cx();
                        let result = run_array_buffer_data_algorithm(cx, b);

                        match result {
                            Ok(FetchedData::ArrayBuffer(a)) => promise.resolve_native(&a),
                            Err(e) => promise.reject_error(e),
                            _ => panic!("Unexpected result from run_array_buffer_data_algorithm"),
                        }
                    },
                    Err(e) => promise.reject_error(e),
                };
            }),
        );
        p
    }
}

/// Get the normalized, MIME-parsable type string
/// <https://w3c.github.io/FileAPI/#dfn-type>
/// XXX: We will relax the restriction here,
/// since the spec has some problem over this part.
/// see <https://github.com/w3c/FileAPI/issues/43>
pub fn normalize_type_string(s: &str) -> String {
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
