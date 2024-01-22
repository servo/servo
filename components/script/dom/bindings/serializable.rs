/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [serializable objects]
//! (<https://html.spec.whatwg.org/multipage/#serializable-objects>).

use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::structuredclone::StructuredDataHolder;
use crate::dom::globalscope::GlobalScope;

/// The key corresponding to the storage location
/// of a serialized platform object stored in a StructuredDataHolder.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StorageKey {
    pub index: u32,
    pub name_space: u32,
}

/// Interface for serializable platform objects.
/// <https://html.spec.whatwg.org/multipage/#serializable>
pub trait Serializable: DomObject {
    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    fn serialize(&self, sc_holder: &mut StructuredDataHolder) -> Result<StorageKey, ()>;
    /// <https://html.spec.whatwg.org/multipage/#deserialization-steps>
    fn deserialize(
        owner: &GlobalScope,
        sc_holder: &mut StructuredDataHolder,
        extra_data: StorageKey,
    ) -> Result<(), ()>;
}
