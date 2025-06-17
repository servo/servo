/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::{HandleObject, HandleValue};

use crate::dom::abortsignal::AbortSignal;
use crate::dom::bindings::codegen::Bindings::AbortControllerBinding::AbortControllerMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::realms::InRealm;
use crate::script_runtime::{CanGc, JSContext};

/// <https://dom.spec.whatwg.org/#abortcontroller>
#[dom_struct]
pub(crate) struct AbortController {
    reflector_: Reflector,

    /// An AbortController object has an associated signal (an AbortSignal object).
    signal: Dom<AbortSignal>,
}

impl AbortController {
    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-abortcontroller>
    fn new_inherited(signal: &AbortSignal) -> AbortController {
        // Note: continuation of the constructor steps.

        // Set this’s signal to signal.
        AbortController {
            reflector_: Reflector::new(),
            signal: Dom::from_ref(signal),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-abortcontroller>
    pub(crate) fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<AbortController> {
        // The new AbortController() constructor steps are:
        // Let signal be a new AbortSignal object.
        let signal = AbortSignal::new_with_proto(global, None, can_gc);
        reflect_dom_object_with_proto(
            Box::new(AbortController::new_inherited(&signal)),
            global,
            proto,
            can_gc,
        )
    }

    /// <https://dom.spec.whatwg.org/#abortcontroller-signal-abort>
    pub(crate) fn signal_abort(
        &self,
        cx: JSContext,
        reason: HandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        // signal abort on controller’s signal with reason if it is given.
        self.signal.signal_abort(cx, reason, realm, can_gc);
    }

    /// <https://dom.spec.whatwg.org/#abortcontroller-signal>
    pub(crate) fn signal(&self) -> DomRoot<AbortSignal> {
        // The signal getter steps are to return this’s signal.
        self.signal.as_rooted()
    }
}

impl AbortControllerMethods<crate::DomTypeHolder> for AbortController {
    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-abortcontroller>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<AbortController> {
        AbortController::new_with_proto(global, proto, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-abort>
    fn Abort(&self, cx: JSContext, reason: HandleValue, realm: InRealm, can_gc: CanGc) {
        // The abort(reason) method steps are
        // to signal abort on this with reason if it is given.
        self.signal_abort(cx, reason, realm, can_gc);
    }

    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-signal>
    fn Signal(&self) -> DomRoot<AbortSignal> {
        // The signal getter steps are to return this’s signal.
        self.signal.as_rooted()
    }
}
