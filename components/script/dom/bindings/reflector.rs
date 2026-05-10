/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::realms::InRealm;
use script_bindings::reflector::DomGlobalGeneric;
use script_bindings::root::DomRoot;

use crate::DomTypeHolder;
use crate::dom::types::GlobalScope;
use crate::realms::enter_realm;

pub(crate) trait DomGlobal {
    /// Returns the [relevant global] in whatever realm is currently active.
    ///
    /// [relevant global]: https://html.spec.whatwg.org/multipage/#concept-relevant-global
    fn global_(&self, realm: InRealm) -> DomRoot<GlobalScope>;

    /// Returns the [relevant global] in the same realm as the callee object.
    /// If you know the callee's realm is already the current realm, it is
    /// more efficient to call [DomGlobal::global_] instead.
    ///
    /// [relevant global]: https://html.spec.whatwg.org/multipage/#concept-relevant-global
    fn global(&self) -> DomRoot<GlobalScope>;
}

impl<T: DomGlobalGeneric<DomTypeHolder>> DomGlobal for T {
    fn global_(&self, realm: InRealm) -> DomRoot<GlobalScope> {
        <Self as DomGlobalGeneric<DomTypeHolder>>::global_(self, realm)
    }

    fn global(&self) -> DomRoot<GlobalScope> {
        let realm = enter_realm(self);
        <Self as DomGlobalGeneric<DomTypeHolder>>::global_(self, InRealm::entered(&realm))
    }
}
