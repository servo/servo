/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [serializable objects]
//! (<https://html.spec.whatwg.org/multipage/#serializable-objects>).

use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::structuredclone::{CloneableObject, StructuredDataHolder};
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

pub trait ToSerializeOperations {
    fn to_serialize_operations(&self) -> Vec<SerializeOperation>;
}

pub enum SerializeOperation {
    Uint32Pair(u32, u32),
}

/// Interface for serializable platform objects.
/// <https://html.spec.whatwg.org/multipage/#serializable>
pub trait Serializable: DomObject {
    type Data: ToSerializeOperations;
    const TAG: CloneableObject;

    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    fn serialize(&self, sc_holder: &mut StructuredDataHolder) -> Result<Self::Data, ()>;
    /// <https://html.spec.whatwg.org/multipage/#deserialization-steps>
    fn deserialize(
        owner: &GlobalScope,
        sc_holder: &mut StructuredDataHolder,
        extra_data: Self::Data,
    ) -> Result<(), ()>;
}
