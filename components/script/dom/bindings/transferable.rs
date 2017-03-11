/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Trait representing the concept of [transferable objects]
//! (https://html.spec.whatwg.org/multipage/#transferable-objects).
use dom::bindings::error::Fallible;
use dom::bindings::js::Root;
use dom::bindings::reflector::DomObject;
use dom::globalscope::GlobalScope;

pub trait Transferable : DomObject {
    fn transfer(&self, target_global: &GlobalScope) -> Fallible<Root<Self>> where Self: Sized;
    fn detached(&self) -> Option<bool> { None }
    fn set_detached(&self, _value: bool) { }
    fn transferable(&self) -> bool { false }
}
