/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::reflector::DomGlobalGeneric;
use script_bindings::root::DomRoot;

use crate::DomTypeHolder;
use crate::dom::types::GlobalScope;

pub(crate) trait DomGlobal {
    /// Returns the [relevant global] in the same realm as the callee object.
    /// Will enter the realm of the global to ensure the global is only
    /// accessed from the correct realm.
    ///
    /// [relevant global]: https://html.spec.whatwg.org/multipage/#concept-relevant-global
    fn global(&self) -> DomRoot<GlobalScope>;
}

impl<T: DomGlobalGeneric<DomTypeHolder>> DomGlobal for T {
    fn global(&self) -> DomRoot<GlobalScope> {
        <Self as DomGlobalGeneric<DomTypeHolder>>::global_from_reflector(self)
    }
}
