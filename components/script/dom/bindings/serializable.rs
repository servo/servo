/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [serializable objects]
//! (<https://html.spec.whatwg.org/multipage/#serializable-objects>).

use std::collections::HashMap;

use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::{StructuredDataReader, StructuredDataWriter};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

/// The key corresponding to the storage location
/// of a serialized platform object stored in a StructuredDataHolder.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct StorageKey {
    pub(crate) index: u32,
    pub(crate) name_space: u32,
}

/// Interface for serializable platform objects.
/// <https://html.spec.whatwg.org/multipage/#serializable>
pub(crate) trait Serializable: DomObject where Self: Sized {
    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    fn serialize(&self, sc_writer: &mut StructuredDataWriter) -> Result<StorageKey, ()>;
    /// <https://html.spec.whatwg.org/multipage/#deserialization-steps>
    fn deserialize(
        owner: &GlobalScope,
        sc_reader: &mut StructuredDataReader,
        extra_data: StorageKey,
        can_gc: CanGc,
    ) -> Result<DomRoot<Self>, ()> where Self: Sized;
    /// Returns the field of [StructuredDataReader] that should be used to store
    /// deserialized instances of this type.
    fn destination(reader: &mut StructuredDataReader) -> &mut Option<HashMap<StorageKey, DomRoot<Self>>>;
}
