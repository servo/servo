/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [serializable objects]
//! (<https://html.spec.whatwg.org/multipage/#serializable-objects>).

use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::structuredclone::{StructuredDataReader, StructuredDataWriter};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

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
    fn serialize(&self, sc_writer: &mut StructuredDataWriter) -> Result<StorageKey, ()>;
    /// <https://html.spec.whatwg.org/multipage/#deserialization-steps>
    fn deserialize(
        owner: &GlobalScope,
        sc_reader: &mut StructuredDataReader,
        extra_data: StorageKey,
        can_gc: CanGc,
    ) -> Result<(), ()>;
}
