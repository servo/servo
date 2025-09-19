/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

use dom_struct::dom_struct;
use indexmap::IndexSet;
use js::jsapi::{ExceptionStackBehavior, Heap, JS_SetPendingException};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use script_bindings::inheritance::Castable;
use script_bindings::trace::CustomTraceable;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AbortSignalBinding::AbortSignalMethods;
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::readablestream::PipeTo;
use crate::fetch::FetchContext;
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
    Fetch(
        #[no_trace]
        #[conditional_malloc_size_of]
        Arc<Mutex<FetchContext>>,
    ),
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

    /// <https://dom.spec.whatwg.org/#abortsignal-dependent>
    dependent: Cell<bool>,

    /// <https://dom.spec.whatwg.org/#abortsignal-source-signals>
    source_signals: DomRefCell<IndexSet<Dom<AbortSignal>>>,

    /// <https://dom.spec.whatwg.org/#abortsignal-dependent-signals>
    dependent_signals: DomRefCell<IndexSet<Dom<AbortSignal>>>,
}

impl AbortSignal {
    fn new_inherited() -> AbortSignal {
        AbortSignal {
            eventtarget: EventTarget::new_inherited(),
            abort_reason: Default::default(),
            abort_algorithms: Default::default(),
            dependent: Default::default(),
            source_signals: Default::default(),
            dependent_signals: Default::default(),
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
    #[cfg_attr(crown, allow(crown::unrooted_must_root))] // TODO(39333): Remove when all iterators are marked as safe
    pub(crate) fn signal_abort(
        &self,
        cx: SafeJSContext,
        reason: HandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        let global = self.global();

        // Step 1. If signal is aborted, then return.
        if self.Aborted() {
            return;
        }

        // Step 2. Set signal’s abort reason to reason if it is given;
        // otherwise to a new "AbortError" DOMException.
        let abort_reason = reason.get();
        if !abort_reason.is_undefined() {
            self.abort_reason.set(abort_reason);
        } else {
            rooted!(in(*cx) let mut rooted_error = UndefinedValue());
            Error::Abort.to_jsval(cx, &global, rooted_error.handle_mut(), can_gc);
            self.abort_reason.set(rooted_error.get())
        }

        // Step 3. Let dependentSignalsToAbort be a new list.
        let mut dependent_signals_to_abort = vec![];
        // Step 4. For each dependentSignal of signal’s dependent signals:
        for dependent_signal in self.dependent_signals.borrow().iter() {
            // Step 4.1. If dependentSignal is not aborted:
            if !dependent_signal.aborted() {
                // Step 4.1.1. Set dependentSignal’s abort reason to signal’s abort reason.
                dependent_signal.abort_reason.set(self.abort_reason.get());
                // Step 4.1.2. Append dependentSignal to dependentSignalsToAbort.
                dependent_signals_to_abort.push(dependent_signal.as_rooted());
            }
        }

        // Step 5. Run the abort steps for signal.
        self.run_the_abort_steps(cx, &global, realm, can_gc);

        // Step 6. For each dependentSignal of dependentSignalsToAbort, run the abort steps for dependentSignal.
        for dependent_signal in dependent_signals_to_abort.iter() {
            dependent_signal.run_the_abort_steps(cx, &global, realm, can_gc);
        }
    }

    /// <https://dom.spec.whatwg.org/#abortsignal-add>
    pub(crate) fn add(&self, algorithm: &AbortAlgorithm) {
        // Step 1. If signal is aborted, then return.
        if self.aborted() {
            return;
        }

        // Step 2. Append algorithm to signal’s abort algorithms.
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
            AbortAlgorithm::Fetch(fetch_context) => {
                rooted!(in(*cx) let mut reason = UndefinedValue());
                reason.set(self.abort_reason.get());
                fetch_context
                    .lock()
                    .unwrap()
                    .abort_fetch(reason.handle(), cx, can_gc);
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
        // Step 1. For each algorithm of signal’s abort algorithms: run algorithm.
        for algo in self.abort_algorithms.borrow().iter() {
            self.run_abort_algorithm(cx, global, algo, realm, can_gc);
        }
        // Step 2. Empty signal’s abort algorithms.
        self.abort_algorithms.borrow_mut().clear();

        // Step 3. Fire an event named abort at signal.
        self.upcast::<EventTarget>()
            .fire_event(atom!("abort"), can_gc);
    }

    /// <https://dom.spec.whatwg.org/#abortsignal-aborted>
    pub(crate) fn aborted(&self) -> bool {
        // An AbortSignal object is aborted when its abort reason is not undefined.
        !self.abort_reason.get().is_undefined()
    }

    /// <https://dom.spec.whatwg.org/#create-a-dependent-abort-signal>
    #[cfg_attr(crown, allow(crown::unrooted_must_root))] // TODO(39333): Remove when all iterators are marked as safe
    pub(crate) fn create_dependent_abort_signal(
        signals: Vec<DomRoot<AbortSignal>>,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> DomRoot<AbortSignal> {
        // Step 1. Let resultSignal be a new object implementing signalInterface using realm.
        let result_signal = Self::new_with_proto(global, None, can_gc);
        // Step 2. For each signal of signals: if signal is aborted,
        // then set resultSignal’s abort reason to signal’s abort reason and return resultSignal.
        for signal in signals.iter() {
            if signal.aborted() {
                result_signal.abort_reason.set(signal.abort_reason.get());
                return result_signal;
            }
        }
        // Step 3. Set resultSignal’s dependent to true.
        result_signal.dependent.set(true);
        // Step 4. For each signal of signals:
        for signal in signals.iter() {
            // Step 4.1. If signal’s dependent is false:
            if !signal.dependent.get() {
                // Step 4.1.1. Append signal to resultSignal’s source signals.
                result_signal
                    .source_signals
                    .borrow_mut()
                    .insert(Dom::from_ref(signal));
                // Step 4.1.2. Append resultSignal to signal’s dependent signals.
                signal
                    .dependent_signals
                    .borrow_mut()
                    .insert(Dom::from_ref(&result_signal));
            } else {
                // Step 4.2. Otherwise, for each sourceSignal of signal’s source signals:
                for source_signal in signal.source_signals.borrow().iter() {
                    // Step 4.2.1. Assert: sourceSignal is not aborted and not dependent.
                    assert!(!source_signal.aborted() && !source_signal.dependent.get());
                    // Step 4.2.2. Append sourceSignal to resultSignal’s source signals.
                    result_signal
                        .source_signals
                        .borrow_mut()
                        .insert(source_signal.clone());
                    // Step 4.2.3. Append resultSignal to sourceSignal’s dependent signals.
                    source_signal
                        .dependent_signals
                        .borrow_mut()
                        .insert(Dom::from_ref(&result_signal));
                }
            }
        }
        // Step 5. Return resultSignal.
        result_signal
    }
}

impl AbortSignalMethods<crate::DomTypeHolder> for AbortSignal {
    /// <https://dom.spec.whatwg.org/#dom-abortsignal-aborted>
    fn Aborted(&self) -> bool {
        // The aborted getter steps are to return true if this is aborted; otherwise false.
        self.aborted()
    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-abort>
    fn Abort(
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: HandleValue,
        can_gc: CanGc,
    ) -> DomRoot<AbortSignal> {
        // Step 1. Let signal be a new AbortSignal object.
        let signal = AbortSignal::new_with_proto(global, None, can_gc);

        // Step 2. Set signal’s abort reason to reason if it is given;
        // otherwise to a new "AbortError" DOMException.
        let abort_reason = reason.get();
        if !abort_reason.is_undefined() {
            signal.abort_reason.set(abort_reason);
        } else {
            rooted!(in(*cx) let mut rooted_error = UndefinedValue());
            Error::Abort.to_jsval(cx, global, rooted_error.handle_mut(), can_gc);
            signal.abort_reason.set(rooted_error.get())
        }

        // Step 3. Return signal.
        signal
    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-any>
    fn Any(
        global: &GlobalScope,
        signals: Vec<DomRoot<AbortSignal>>,
        can_gc: CanGc,
    ) -> DomRoot<AbortSignal> {
        // The static any(signals) method steps are to return the result
        // of creating a dependent abort signal from signals using AbortSignal and the current realm.
        Self::create_dependent_abort_signal(signals, global, can_gc)
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
