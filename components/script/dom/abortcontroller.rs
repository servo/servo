/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use js::rust::{HandleObject, HandleValue};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};

use crate::dom::abortsignal::AbortSignal;
use crate::dom::bindings::codegen::Bindings::AbortControllerBinding::AbortControllerMethods;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;

/// <https://dom.spec.whatwg.org/#abortcontroller>
#[dom_struct]
pub(crate) struct AbortController {
    reflector_: Reflector,

    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-signal>
    signal: Dom<AbortSignal>,
}

impl AbortController {
    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-abortcontroller>
    fn new_inherited(signal: &AbortSignal) -> AbortController {
        // Note: continuation of the constructor steps.

        // Step 2. Set this’s signal to signal.
        AbortController {
            reflector_: Reflector::new(),
            signal: Dom::from_ref(signal),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-abortcontroller>
    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<AbortController> {
        // Step 1. Let signal be a new AbortSignal object.
        let signal = AbortSignal::new_with_proto(cx, global, None);
        // Step 2. Set this’s signal to signal.
        reflect_dom_object_with_proto_and_cx(
            Box::new(AbortController::new_inherited(&signal)),
            global,
            proto,
            cx,
        )
    }

    /// <https://dom.spec.whatwg.org/#abortcontroller-signal-abort>
    pub(crate) fn signal_abort(&self, cx: &mut CurrentRealm, reason: HandleValue) {
        // To signal abort on an AbortController controller with an optional reason,
        // signal abort on controller’s signal with reason if it is given.
        self.signal.signal_abort(cx, reason);
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
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<AbortController> {
        AbortController::new_with_proto(cx, global, proto)
    }

    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-abort>
    fn Abort(&self, cx: &mut CurrentRealm, reason: HandleValue) {
        // The abort(reason) method steps are
        // to signal abort on this with reason if it is given.
        self.signal_abort(cx, reason);
    }

    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-signal>
    fn Signal(&self) -> DomRoot<AbortSignal> {
        // The signal getter steps are to return this’s signal.
        self.signal.as_rooted()
    }
}
