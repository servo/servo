/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [transferable objects]
//! (https://html.spec.whatwg.org/multipage/#transferable-objects).
use dom::bindings::reflector::DomObject;
use js::jsapi::{JSContext, JSStructuredCloneReader, MutableHandleObject};
use std::os::raw;

pub trait Transferable : DomObject {
    fn transfer(
        &self,
        closure: *mut raw::c_void,
        content: *mut *mut raw::c_void,
        extra_data: *mut u64
    ) -> bool;
    fn transfer_receive(
        cx: *mut JSContext,
        r: *mut JSStructuredCloneReader,
        closure: *mut raw::c_void,
        content: *mut raw::c_void,
        extra_data: u64,
        return_object: MutableHandleObject
    ) -> bool;
    fn detached(&self) -> Option<bool> { None }
    fn set_detached(&self, _value: bool) { }
    fn transferable(&self) -> bool { false }
}
