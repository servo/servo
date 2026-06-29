/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_bindings::reflector::DomGlobalGeneric;
use script_bindings::root::DomRoot;

use crate::DomTypeHolder;
use crate::dom::types::GlobalScope;
use crate::realms::enter_auto_realm;

pub(crate) trait DomGlobal {
    /// Returns the [relevant global] in the same realm as the callee object.
    /// Will enter the realm of the global to ensure the global is only
    /// accessed from the correct realm.
    ///
    /// [relevant global]: https://html.spec.whatwg.org/multipage/#concept-relevant-global
    fn global(&self) -> DomRoot<GlobalScope>;
}

impl<T: DomGlobalGeneric<DomTypeHolder>> DomGlobal for T {
    #[expect(unsafe_code)]
    fn global(&self) -> DomRoot<GlobalScope> {
        // SAFETY: We only use this `cx` to enter a realm. That does not
        // incur a GC and hence is safe to perform. We do not want to
        // pass a `cx` as parameter to this function, as this used in
        // loads of places. At the same time, it also isn't necessary in
        // nearly all cases to enter realm, since we are already in the
        // correct realm.
        //
        // However, there are cases where it is difficult to ensure that
        // we are in the correct realm. Hence we always enter a realm here
        // even if that is unnecessary at times.
        let cx = unsafe { JSContext::get_from_thread() };
        let cx = &mut cx.expect("JS runtime has shut down");
        let _realm = enter_auto_realm(cx, self);
        <Self as DomGlobalGeneric<DomTypeHolder>>::global_from_reflector(self)
    }
}
