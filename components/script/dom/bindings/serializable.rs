/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [serializable objects]
//! (https://html.spec.whatwg.org/multipage/#serializable-objects).
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::StructuredCloneHolder;
use crate::dom::globalscope::GlobalScope;

pub trait Serializable: DomObject {
    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    fn serialize(&self, sc_holder: &mut StructuredCloneHolder) -> Result<(u32, u32), ()>;
    /// <https://html.spec.whatwg.org/multipage/#deserialization-steps>
    fn deserialize(
        owner: &DomRoot<GlobalScope>,
        sc_holder: &mut StructuredCloneHolder,
        extra_data: (u32, u32),
    ) -> Result<usize, ()>;
}
