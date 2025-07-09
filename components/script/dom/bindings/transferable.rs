/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [transferable objects]
//! (<https://html.spec.whatwg.org/multipage/#transferable-objects>).

use std::collections::HashMap;
use std::hash::Hash;

use base::id::NamespaceIndex;

use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::globalscope::GlobalScope;

pub(crate) trait Transferable: DomObject
where
    Self: Sized,
{
    type Index: Copy + Eq + Hash;
    type Data;

    fn can_transfer(&self) -> bool {
        true
    }

    /// <https://html.spec.whatwg.org/multipage/#transfer-steps>
    fn transfer(&self) -> Fallible<(NamespaceIndex<Self::Index>, Self::Data)>;

    /// <https://html.spec.whatwg.org/multipage/#transfer-receiving-steps>
    fn transfer_receive(
        owner: &GlobalScope,
        id: NamespaceIndex<Self::Index>,
        serialized: Self::Data,
    ) -> Result<DomRoot<Self>, ()>;

    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<HashMap<NamespaceIndex<Self::Index>, Self::Data>>;
}
