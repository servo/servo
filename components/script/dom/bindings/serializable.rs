/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [serializable objects]
//! (<https://html.spec.whatwg.org/multipage/#serializable-objects>).

use js::jsapi::{JSStructuredCloneReader, JS_ReadUint32Pair};

use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::{CloneableObject, StructuredWriteDataHolder, StructuredReadDataHolder};
use crate::dom::globalscope::GlobalScope;

/// The key corresponding to the storage location
/// of a serialized platform object stored in a StructuredDataHolder.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StorageKey {
    pub index: u32,
    pub name_space: u32,
}

impl ToSerializeOperations for StorageKey {
    fn to_serialize_operations(&self) -> Vec<SerializeOperation> {
        vec![SerializeOperation::Uint32Pair(self.name_space, self.index)]
    }
}

impl FromStructuredClone for StorageKey {
    unsafe fn from_structured_clone(r: *mut JSStructuredCloneReader) -> StorageKey {
        let mut key = StorageKey {
            name_space: 0,
            index: 0,
        };
        assert!(JS_ReadUint32Pair(
            r,
            &mut key.name_space,
            &mut key.index,
        ));
        key
     }
}

/// An interface to construct a stream of serialization operations that will be
/// evaluated as part of serializing a DOM object.
pub trait ToSerializeOperations {
    fn to_serialize_operations(&self) -> Vec<SerializeOperation>;
}

/// An interface to reconstruct a known type using previously-serialized values
/// that can be obtained from the provided JSStructuredCloneReader.
///
/// Safety:
/// The provided pointer must point to a live JSStructuredCloneReader object.
pub trait FromStructuredClone: Sized {
    unsafe fn from_structured_clone(r: *mut JSStructuredCloneReader) -> Self;
}

/// Operations permitted as part of structured clones to serialize arbitrary values
/// during serialization of DOM objects.
pub enum SerializeOperation {
    Uint32Pair(u32, u32),
}

/// Interface for serializable platform objects.
/// <https://html.spec.whatwg.org/multipage/#serializable>
pub trait Serializable: DomObject + Sized {
    /// Arbitrary additional data that must be serialized in order to support
    /// deserializing this platform object correctly.
    type Data: ToSerializeOperations + FromStructuredClone;
    /// A unique identifying value for this serializable platform object.
    const TAG: CloneableObject;

    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    fn serialize(&self, sc_holder: &mut StructuredWriteDataHolder) -> Result<Self::Data, ()>;
    /// <https://html.spec.whatwg.org/multipage/#deserialization-steps>
    fn deserialize(
        owner: &GlobalScope,
        sc_holder: &mut StructuredReadDataHolder,
        extra_data: Self::Data,
    ) -> Result<DomRoot<Self>, ()>;
}
