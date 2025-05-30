/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::mem;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use script_bindings::inheritance::Castable;

use crate::dom::bindings::codegen::Bindings::AbortSignalBinding::AbortSignalMethods;
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext};

/// <https://dom.spec.whatwg.org/#abortcontroller-api-integration>
/// TODO: implement algorithms at call point,
/// in order to integrate the abort signal with its various use cases.
#[derive(JSTraceable, MallocSizeOf)]
#[allow(dead_code)]
enum AbortAlgorithm {
    /// <https://dom.spec.whatwg.org/#add-an-event-listener>
    DomEventLister,
    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    StreamPiping,
    /// <https://fetch.spec.whatwg.org/#dom-global-fetch>
    Fetch,
}

/// <https://dom.spec.whatwg.org/#abortsignal>
#[dom_struct]
pub(crate) struct AbortSignal {
    eventtarget: EventTarget,

    /// <https://dom.spec.whatwg.org/#abortsignal-abort-reason>
    #[ignore_malloc_size_of = "mozjs"]
    abort_reason: Heap<JSVal>,

    /// <https://dom.spec.whatwg.org/#abortsignal-abort-algorithms>
    abort_algorithms: RefCell<Vec<AbortAlgorithm>>,
}

impl AbortSignal {
    pub(crate) fn new_inherited() -> AbortSignal {
        AbortSignal {
            eventtarget: EventTarget::new_inherited(),
            abort_reason: Default::default(),
            abort_algorithms: Default::default(),
        }
    }

    #[allow(dead_code)]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<AbortSignal> {
        reflect_dom_object_with_proto(
            Box::new(AbortSignal::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    /// <https://dom.spec.whatwg.org/#abortsignal-signal-abort>
    pub(crate) fn signal_abort(&self, cx: JSContext, reason: HandleValue, can_gc: CanGc) {
        // If signal is aborted, then return.
        if self.Aborted() {
            return;
        }

        let abort_reason = reason.get();

        // Set signal’s abort reason to reason if it is given;
        if !abort_reason.is_undefined() {
            self.abort_reason.set(abort_reason);
        } else {
            // otherwise to a new "AbortError" DOMException.
            rooted!(in(*cx) let mut rooted_error = UndefinedValue());
            Error::Abort.to_jsval(cx, &self.global(), rooted_error.handle_mut(), can_gc);
            self.abort_reason.set(rooted_error.get())
        }

        // Let dependentSignalsToAbort be a new list.
        // For each dependentSignal of signal’s dependent signals:
        // TODO: #36936

        // Run the abort steps for signal.
        self.run_the_abort_steps(can_gc);

        // For each dependentSignal of dependentSignalsToAbort, run the abort steps for dependentSignal.
        // TODO: #36936
    }

    /// <https://dom.spec.whatwg.org/#run-the-abort-steps>
    fn run_the_abort_steps(&self, can_gc: CanGc) {
        // For each algorithm of signal’s abort algorithms: run algorithm.
        let algos = mem::take(&mut *self.abort_algorithms.borrow_mut());
        for _algo in algos {
            // TODO: match on variant and implement algo steps.
            // See the various items of #34866
        }

        // Empty signal’s abort algorithms.
        // Done above with `take`.

        // Fire an event named abort at signal.
        self.upcast::<EventTarget>()
            .fire_event(atom!("abort"), can_gc);
    }
}

impl AbortSignalMethods<crate::DomTypeHolder> for AbortSignal {
    /// <https://dom.spec.whatwg.org/#dom-abortsignal-aborted>
    fn Aborted(&self) -> bool {
        // TODO
        false
    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-reason>
    fn Reason(&self, _: JSContext, _rval: MutableHandleValue) {
        // TODO
    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-throwifaborted>
    #[allow(unsafe_code)]
    fn ThrowIfAborted(&self) {
        // TODO
    }

    // <https://dom.spec.whatwg.org/#dom-abortsignal-onabort>
    event_handler!(abort, GetOnabort, SetOnabort);
}
