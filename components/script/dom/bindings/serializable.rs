/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [serializable objects]
//! (<https://html.spec.whatwg.org/multipage/#serializable-objects>).

pub(crate) use script_bindings::serializable::Serializable;
use servo_base::id::{Index, NamespaceIndex, PipelineNamespaceId};

/// The key corresponding to the storage location
/// of a serialized platform object stored in a StructuredDataHolder.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct StorageKey {
    pub(crate) index: u32,
    pub(crate) name_space: u32,
}

impl StorageKey {
    pub(crate) fn new<T>(index: NamespaceIndex<T>) -> StorageKey {
        let name_space = index.namespace_id.0.to_ne_bytes();
        let index = index.index.0.get().to_ne_bytes();
        StorageKey {
            index: u32::from_ne_bytes(index),
            name_space: u32::from_ne_bytes(name_space),
        }
    }
}

impl<T> From<StorageKey> for NamespaceIndex<T> {
    fn from(key: StorageKey) -> NamespaceIndex<T> {
        NamespaceIndex {
            namespace_id: PipelineNamespaceId(key.name_space),
            index: Index::new(key.index).expect("Index must not be zero"),
        }
    }
}

pub(crate) fn assert_serializable<T: Serializable<crate::DomTypeHolder>>() {}
