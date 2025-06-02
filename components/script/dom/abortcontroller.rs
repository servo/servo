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
    fn new_inherited() -> AbortController {
        // The new AbortController() constructor steps are:
        // Let signal be a new AbortSignal object.
        // Set this’s signal to signal.
        AbortController {
            reflector_: Reflector::new(),
            signal: Dom::from_ref(&AbortSignal::new_inherited()),
        }
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<AbortController> {
        reflect_dom_object_with_proto(
            Box::new(AbortController::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    /// <https://dom.spec.whatwg.org/#abortcontroller-signal-abort>
    fn signal_abort(&self, cx: JSContext, reason: HandleValue, can_gc: CanGc) {
        // signal abort on controller’s signal with reason if it is given.
        self.signal.signal_abort(cx, reason, can_gc);
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
    fn Abort(&self, cx: JSContext, reason: HandleValue, can_gc: CanGc) {
        // The abort(reason) method steps are
        // to signal abort on this with reason if it is given.
        self.signal_abort(cx, reason, can_gc);
    }

    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-signal>
    fn Signal(&self) -> DomRoot<AbortSignal> {
        // The signal getter steps are to return this’s signal.
        self.signal.as_rooted()
    }
}
