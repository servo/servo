/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use js::jsapi::{ExceptionStackBehavior, Heap, JS_SetPendingException};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use script_bindings::inheritance::Castable;

use crate::dom::bindings::codegen::Bindings::AbortSignalBinding::AbortSignalMethods;
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::readablestream::PipeTo;
use crate::realms::InRealm;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

impl js::gc::Rootable for AbortAlgorithm {}

/// <https://dom.spec.whatwg.org/#abortcontroller-api-integration>
/// TODO: implement algorithms at call point,
/// in order to integrate the abort signal with its various use cases.
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[allow(dead_code)]
pub(crate) enum AbortAlgorithm {
    /// <https://dom.spec.whatwg.org/#add-an-event-listener>
    DomEventLister,
    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    StreamPiping(PipeTo),
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
    fn new_inherited() -> AbortSignal {
        AbortSignal {
            eventtarget: EventTarget::new_inherited(),
            abort_reason: Default::default(),
            abort_algorithms: Default::default(),
        }
    }

    pub(crate) fn new_with_proto(
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
    pub(crate) fn signal_abort(
        &self,
        cx: SafeJSContext,
        reason: HandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        let global = self.global();

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
            Error::Abort.to_jsval(cx, &global, rooted_error.handle_mut(), can_gc);
            self.abort_reason.set(rooted_error.get())
        }

        // Let dependentSignalsToAbort be a new list.
        // For each dependentSignal of signal’s dependent signals:
        // TODO: #36936

        // Run the abort steps for signal.
        self.run_the_abort_steps(cx, &global, realm, can_gc);

        // For each dependentSignal of dependentSignalsToAbort, run the abort steps for dependentSignal.
        // TODO: #36936
    }

    /// <https://dom.spec.whatwg.org/#abortsignal-add>
    pub(crate) fn add(&self, algorithm: &AbortAlgorithm) {
        // If signal is aborted, then return.
        if self.aborted() {
            return;
        }

        // Append algorithm to signal’s abort algorithms.
        self.abort_algorithms.borrow_mut().push(algorithm.clone());
    }

    /// Run a specific abort algorithm.
    pub(crate) fn run_abort_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        algorithm: &AbortAlgorithm,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        match algorithm {
            AbortAlgorithm::StreamPiping(pipe) => {
                rooted!(in(*cx) let mut reason = UndefinedValue());
                reason.set(self.abort_reason.get());
                pipe.abort_with_reason(cx, global, reason.handle(), realm, can_gc);
            },
            _ => {
                // TODO: match on variant and implement algo steps.
                // See the various items of #34866
            },
        }
    }

    /// <https://dom.spec.whatwg.org/#run-the-abort-steps>
    fn run_the_abort_steps(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        // For each algorithm of signal’s abort algorithms: run algorithm.
        for algo in self.abort_algorithms.borrow().iter() {
            self.run_abort_algorithm(cx, global, algo, realm, can_gc);
        }

        // Empty signal’s abort algorithms.
        self.abort_algorithms.borrow_mut().clear();

        // Fire an event named abort at signal.
        self.upcast::<EventTarget>()
            .fire_event(atom!("abort"), can_gc);
    }

    /// <https://dom.spec.whatwg.org/#abortsignal-aborted>
    pub(crate) fn aborted(&self) -> bool {
        // An AbortSignal object is aborted when its abort reason is not undefined.
        !self.abort_reason.get().is_undefined()
    }
}

impl AbortSignalMethods<crate::DomTypeHolder> for AbortSignal {
    /// <https://dom.spec.whatwg.org/#dom-abortsignal-aborted>
    fn Aborted(&self) -> bool {
        // The aborted getter steps are to return true if this is aborted; otherwise false.
        self.aborted()
    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-reason>
    fn Reason(&self, _cx: SafeJSContext, mut rval: MutableHandleValue) {
        // The reason getter steps are to return this’s abort reason.
        rval.set(self.abort_reason.get());
    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-throwifaborted>
    #[allow(unsafe_code)]
    fn ThrowIfAborted(&self) {
        // The throwIfAborted() method steps are to throw this’s abort reason, if this is aborted.
        if self.aborted() {
            let cx = GlobalScope::get_cx();
            unsafe {
                JS_SetPendingException(
                    *cx,
                    self.abort_reason.handle(),
                    ExceptionStackBehavior::Capture,
                )
            };
        }
    }

    // <https://dom.spec.whatwg.org/#dom-abortsignal-onabort>
    event_handler!(abort, GetOnabort, SetOnabort);
}
