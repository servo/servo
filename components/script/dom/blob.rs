/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::BlobBinding;
use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferOrArrayBufferViewOrBlobOrString;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone::StructuredCloneHolder;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use msg::constellation_msg::{BlobId, BlobIndex, PipelineNamespaceId};
use net_traits::filemanager_thread::RelativePos;
use script_traits::serializable::BlobImpl;
use std::num::NonZeroU32;
use uuid::Uuid;

// https://w3c.github.io/FileAPI/#blob
#[dom_struct]
pub struct Blob {
    reflector_: Reflector,
    blob_id: BlobId,
}

impl Blob {
    pub fn new(global: &GlobalScope, blob_impl: BlobImpl) -> DomRoot<Blob> {
        let dom_blob = reflect_dom_object(
            Box::new(Blob {
                reflector_: Reflector::new(),
                blob_id: blob_impl.blob_id(),
            }),
            global,
            BlobBinding::Wrap,
        );
        global.track_blob(&dom_blob, blob_impl);
        dom_blob
    }

    #[allow(unrooted_must_root)]
    pub fn new_inherited(blob_impl: &BlobImpl) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            blob_id: blob_impl.blob_id(),
        }
    }

    // https://w3c.github.io/FileAPI/#constructorBlob
    pub fn Constructor(
        global: &GlobalScope,
        blobParts: Option<Vec<ArrayBufferOrArrayBufferViewOrBlobOrString>>,
        blobPropertyBag: &BlobBinding::BlobPropertyBag,
    ) -> Fallible<DomRoot<Blob>> {
        // TODO: accept other blobParts types - ArrayBuffer or ArrayBufferView
        let bytes: Vec<u8> = match blobParts {
            None => Vec::new(),
            Some(blobparts) => match blob_parts_to_bytes(blobparts) {
                Ok(bytes) => bytes,
                Err(_) => return Err(Error::InvalidCharacter),
            },
        };

        let type_string = normalize_type_string(&blobPropertyBag.type_.to_string());
        let blob_impl = BlobImpl::new_from_bytes(bytes, type_string);

        Ok(Blob::new(global, blob_impl))
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
}

impl Serializable for Blob {
    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    fn serialize(&self, sc_holder: &mut StructuredCloneHolder) -> Result<(u32, u32), ()> {
        let blob_id = self.blob_id.clone();
        // 1. Get a clone of the blob impl.
        let blob_impl = self.global().clone_blob_impl(&blob_id);

        // We clone the data, but the clone gets its own Id.
        let new_blob_id = blob_impl.blob_id();

        // 2. Store the object at a given key.
        sc_holder.blob_impls.insert(new_blob_id.clone(), blob_impl);

        let PipelineNamespaceId(name_space) = new_blob_id.namespace_id;
        let BlobIndex(index) = new_blob_id.index;
        let index = index.get();

        let name_space = name_space.to_ne_bytes();
        let index = index.to_ne_bytes();

        // 3. Return a u32 pair representation of the key where the object is stored.
        Ok((u32::from_ne_bytes(name_space), u32::from_ne_bytes(index)))
    }

    /// <https://html.spec.whatwg.org/multipage/#deserialization-steps>
    fn deserialize(
        owner: &DomRoot<GlobalScope>,
        sc_holder: &mut StructuredCloneHolder,
        extra_data: (u32, u32),
    ) -> Result<usize, ()> {
        // 1. Re-build the key for the storage location
        // of the serialized object.
        let namespace_id = PipelineNamespaceId(extra_data.0);
        let index = BlobIndex(NonZeroU32::new(extra_data.1).expect("Index to be non-zero"));

        let id = BlobId {
            namespace_id,
            index,
        };

        // 2. Get the transferred object from it's storage, using the key.
        let blob_impl = sc_holder
            .blob_impls
            .remove(&id)
            .expect("Transferred port to be stored");

        sc_holder.blobs.push(Blob::new(&**owner, blob_impl));

        Ok(sc_holder.blobs.len() - 1)
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
            &mut ArrayBufferOrArrayBufferViewOrBlobOrString::String(ref s) => {
                ret.extend(s.as_bytes());
            },
            &mut ArrayBufferOrArrayBufferViewOrBlobOrString::Blob(ref b) => {
                let bytes = b.get_bytes().unwrap_or(vec![]);
                ret.extend(bytes);
            },
            &mut ArrayBufferOrArrayBufferViewOrBlobOrString::ArrayBuffer(ref mut a) => unsafe {
                let bytes = a.as_slice();
                ret.extend(bytes);
            },
            &mut ArrayBufferOrArrayBufferViewOrBlobOrString::ArrayBufferView(ref mut a) => unsafe {
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

    // https://w3c.github.io/FileAPI/#slice-method-algo
    fn Slice(
        &self,
        start: Option<i64>,
        end: Option<i64>,
        content_type: Option<DOMString>,
    ) -> DomRoot<Blob> {
        let rel_pos = RelativePos::from_opts(start, end);
        let blob_impl = BlobImpl::new_sliced(
            rel_pos,
            self.blob_id.clone(),
            content_type
                .unwrap_or(DOMString::from(""))
                .to_owned()
                .to_string(),
        );
        Blob::new(&*self.global(), blob_impl)
    }
}

/// Get the normalized, MIME-parsable type string
/// <https://w3c.github.io/FileAPI/#dfn-type>
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
