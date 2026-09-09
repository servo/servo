/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::Atom;

use crate::DomTypes;
use crate::str::DOMString;

/// Trait for subcrates to use DOM events.
pub trait DomEventTrait<D: DomTypes> {
    fn new_inherited() -> Self;
    fn init_event(&self, type_: Atom, bubbles: bool, cancelable: bool);
    #[expect(non_snake_case)]
    fn IsTrusted(&self) -> bool;
    fn fire(&self, cx: &mut js::context::JSContext, target: &D::EventTarget);
}

/// Trait for subcrates to use DomExceptions.
pub trait DomExceptionTrait {
    fn new_inherited(message: DOMString, name: DOMString) -> Self;
}
