/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [transferable objects]
//! (<https://html.spec.whatwg.org/multipage/#transferable-objects>).

use std::collections::HashMap;
use std::num::NonZeroU32;

use base::id::PipelineNamespaceId;

use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::{StructuredDataReader, StructuredDataWriter};
use crate::dom::globalscope::GlobalScope;

pub(crate) trait IdFromComponents
where
    Self: Sized,
{
    fn from(namespace_id: PipelineNamespaceId, index: NonZeroU32) -> Self;
}

pub(crate) trait Transferable: DomObject
where
    Self: Sized,
{
    type Id: Eq + std::hash::Hash + Copy + IdFromComponents;
    type Data;

    fn transfer(&self, sc_writer: &mut StructuredDataWriter) -> Result<u64, ()>;
    fn transfer_receive(
        owner: &GlobalScope,
        id: Self::Id,
        serialized: Self::Data,
    ) -> Result<DomRoot<Self>, ()>;

    fn serialized_storage(
        reader: &mut StructuredDataReader,
    ) -> &mut Option<HashMap<Self::Id, Self::Data>>;
    fn deserialized_storage(reader: &mut StructuredDataReader) -> &mut Option<Vec<DomRoot<Self>>>;
}
