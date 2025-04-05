/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [serializable objects]
//! (<https://html.spec.whatwg.org/multipage/#serializable-objects>).

use std::collections::HashMap;
use std::num::NonZeroU32;

use base::id::PipelineNamespaceId;

use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::{StructuredData, StructuredDataReader};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

/// The key corresponding to the storage location
/// of a serialized platform object stored in a StructuredDataHolder.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct StorageKey {
    pub(crate) index: u32,
    pub(crate) name_space: u32,
}

impl StorageKey {
    pub(crate) fn new(name_space: PipelineNamespaceId, index: NonZeroU32) -> StorageKey {
        let name_space = name_space.0.to_ne_bytes();
        let index = index.get().to_ne_bytes();
        StorageKey {
            index: u32::from_ne_bytes(index),
            name_space: u32::from_ne_bytes(name_space),
        }
    }
}

pub(crate) trait IntoStorageKey
where
    Self: Sized,
{
    fn into_storage_key(self) -> StorageKey;
}

/// Interface for serializable platform objects.
/// <https://html.spec.whatwg.org/multipage/#serializable>
pub(crate) trait Serializable: DomObject
where
    Self: Sized,
{
    type Id: Copy + Eq + std::hash::Hash + IntoStorageKey + From<StorageKey>;
    type Data;

    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    fn serialize(&self) -> Result<(Self::Id, Self::Data), ()>;
    /// <https://html.spec.whatwg.org/multipage/#deserialization-steps>
    fn deserialize(
        owner: &GlobalScope,
        serialized: Self::Data,
        can_gc: CanGc,
    ) -> Result<DomRoot<Self>, ()>
    where
        Self: Sized;

    /// Returns the field of [StructuredDataReader]/[StructuredDataWriter] that
    /// should be used to read/store serialized instances of this type.
    fn serialized_storage(data: StructuredData<'_>) -> &mut Option<HashMap<Self::Id, Self::Data>>;

    /// Returns the field of [StructuredDataReader] that should be used to store
    /// deserialized instances of this type.
    fn deserialized_storage(
        reader: &mut StructuredDataReader,
    ) -> &mut Option<HashMap<StorageKey, DomRoot<Self>>>;
}
