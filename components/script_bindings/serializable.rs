/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::{JSContext, NoGC};
use rustc_hash::FxHashMap;
use servo_base::id::NamespaceIndex;

use crate::DomTypes;
use crate::reflector::DomObject;
use crate::root::DomRoot;
use crate::structuredclone::{MarkedAsSerializableInIdl, StructuredData};

/// Interface for serializable platform objects.
/// <https://html.spec.whatwg.org/multipage/#serializable>
#[allow(clippy::result_unit_err)]
pub trait Serializable<D: DomTypes>: DomObject + MarkedAsSerializableInIdl
where
    Self: Sized,
{
    type Index: Copy + Eq + std::hash::Hash;
    type Data;

    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    fn serialize(&self, no_gc: &NoGC) -> Result<(NamespaceIndex<Self::Index>, Self::Data), ()>;

    /// <https://html.spec.whatwg.org/multipage/#deserialization-steps>
    fn deserialize(
        cx: &mut JSContext,
        owner: &D::GlobalScope,
        serialized: Self::Data,
    ) -> Result<DomRoot<Self>, ()>
    where
        Self: Sized;

    /// Returns the field of [StructuredDataReader]/[StructuredDataWriter] that
    /// should be used to read/store serialized instances of this type.
    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<FxHashMap<NamespaceIndex<Self::Index>, Self::Data>>;
}
