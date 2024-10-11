/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [serializable objects]
//! (<https://html.spec.whatwg.org/multipage/#serializable-objects>).

use js::jsapi::JSStructuredCloneReader;

use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::{CloneableObject, StructuredWriteDataHolder, StructuredReadDataHolder};
use crate::dom::globalscope::GlobalScope;


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
    Double(f64),
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
