/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [transferable objects]
//! (<https://html.spec.whatwg.org/multipage/#transferable-objects>).

use js::jsapi::MutableHandleObject;

use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::structuredclone::StructuredDataHolder;
use crate::dom::globalscope::GlobalScope;

pub trait Transferable: DomObject {
    fn transfer(&self, sc_holder: &mut StructuredDataHolder) -> Result<u64, ()>;
    fn transfer_receive(
        owner: &GlobalScope,
        sc_holder: &mut StructuredDataHolder,
        extra_data: u64,
        return_object: MutableHandleObject,
    ) -> Result<(), ()>;
}
