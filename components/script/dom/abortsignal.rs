/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{ExceptionStackBehavior, Heap, JS_IsExceptionPending};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::wrappers::JS_SetPendingException;
use js::rust::HandleValue;

use super::bindings::weakref::WeakRef;
use super::promise::Promise;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AbortSignalBinding::AbortSignalMethods;
use crate::dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use crate::dom::bindings::codegen::Bindings::EventTargetBinding::EventListenerOptions;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::weakref::WeakReferenceable;
use crate::dom::domexception::DOMErrorName;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::DOMException;
use crate::script_runtime::JSContext;

#[derive(JSTraceable, MallocSizeOf)]
pub enum AbortAlgorithm {
    RemoveEventListener(
        DomRoot<EventTarget>,
        DOMString,
        #[ignore_malloc_size_of = "Rc"] Rc<EventListener>,
        #[ignore_malloc_size_of = "generated"] EventListenerOptions,
    ),
    FetchCancel {
        #[ignore_malloc_size_of = "Rc"]
        locally_aborted: Rc<AtomicBool>,
        promise: Rc<Promise>,
        request: DomRoot<Request>,
        response_object: Option<Response>,
    }
}
impl AbortAlgorithm {
    fn exec(self) {
        match self {
            Self::RemoveEventListener(target, ty, listener, options) => {
                target.remove_event_listener(ty, Some(listener), options)
            },
            // https://fetch.spec.whatwg.org/#fetch-method step 11
            Self::FetchCancelLocal { locally_aborted } => {
                locally_aborted.store(true, Ordering::SeqCst);
                
            }
        }
    }
}

#[dom_struct]
pub struct AbortSignal {
    event_target: EventTarget,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    reason: Heap<JSVal>,
    abort_algorithms: DomRefCell<Vec<AbortAlgorithm>>,

    source_signals: DomRefCell<Vec<WeakRef<AbortSignal>>>,
    dependent_signals: DomRefCell<Vec<WeakRef<AbortSignal>>>,
    dependent: bool,
}

impl AbortSignal {
    pub fn new_inherited(dependent: bool) -> Self {
        Self {
            event_target: EventTarget::new_inherited(),
            reason: Heap::default(),
            abort_algorithms: DomRefCell::default(),
            source_signals: DomRefCell::default(),
            dependent_signals: DomRefCell::default(),
            dependent,
        }
    }
    pub fn new(global: &GlobalScope, dependent: bool) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(dependent)), global)
    }
    // https://dom.spec.whatwg.org/#create-a-dependent-abort-signal
    pub fn create_dependent_signal(
        global: &GlobalScope,
        signals: Vec<DomRoot<Self>>,
    ) -> DomRoot<Self> {
        for signal in &signals {
            // 2
            if signal.Aborted() {
                let result_signal = Self::new(global, false); // 1
                result_signal.reason.set(signal.reason.get().clone());
                return result_signal;
            }
        }
        let result_signal = Self::new(global, true); // 1, 3
        for signal in signals {
            // 4
            if signal.dependent {
                // 4.2
                for source_signal in signal.source_signals.borrow_mut().iter_mut() {
                    // ignore dropped source signals
                    if let Some(source_signal) = source_signal.root() {
                        // 4.2.1
                        assert!(!source_signal.Aborted());
                        assert!(!source_signal.dependent);
                        // 4.2.2
                        result_signal
                            .source_signals
                            .borrow_mut()
                            .push(WeakRef::new(&source_signal));
                        // 4.2.3
                        (*source_signal)
                            .dependent_signals
                            .borrow_mut()
                            .push(WeakRef::new(&result_signal));
                    }
                }
            } else {
                // 4.1.1
                result_signal
                    .source_signals
                    .borrow_mut()
                    .push(signal.downgrade());
                // 4.1.2
                signal
                    .dependent_signals
                    .borrow_mut()
                    .push(WeakRef::new(&result_signal));
            }
        }
        result_signal
    }

    /// <https://dom.spec.whatwg.org/#abortsignal-add>
    pub fn add_abort_algorithm(&self, alg: AbortAlgorithm) {
        if !self.Aborted() {
            self.abort_algorithms.borrow_mut().push(alg);
        }
    }
    /// <https://dom.spec.whatwg.org/#abortsignal-signal-abort>
    #[allow(unsafe_code)]
    pub fn signal_abort(&self, reason: HandleValue) {
        // 1. If signal is aborted, then return.
        if self.Aborted() {
            return;
        }
        // 2. Set signal’s abort reason to reason if it is given; otherwise to a new "AbortError" DOMException.
        let cx = *GlobalScope::get_cx();
        rooted!(in(cx) let mut new_reason = UndefinedValue());
        let reason = if reason.is_undefined() {
            let exception = DOMException::new(&self.global(), DOMErrorName::AbortError);
            unsafe {
                exception.to_jsval(cx, new_reason.handle_mut());
            };
            new_reason.handle()
        } else {
            reason
        };
        self.reason.set(reason.get());

        // 3. For each algorithm of signal’s abort algorithms: run algorithm.
        // 4. Empty signal’s abort algorithms.
        for algorithm in self.abort_algorithms.borrow_mut().drain(..) {
            algorithm.exec();
        }

        // 5. Fire an event named abort at signal.
        let event = Event::new(
            &self.global(),
            atom!("abort"),
            EventBubbles::DoesNotBubble,
            EventCancelable::Cancelable,
        );
        event.fire(self.upcast());
        // 6. For each dependentSignal of signal’s dependent signals,
        // signal abort on dependentSignal with signal’s abort reason.
        // TODO
    }
}

impl AbortSignalMethods for AbortSignal {
    // https://dom.spec.whatwg.org/#dom-abortsignal-onabort
    event_handler!(Abort, GetOnabort, SetOnabort);
    /// <https://dom.spec.whatwg.org/#dom-abortsignal-aborted>
    fn Aborted(&self) -> bool {
        !self.reason.get().is_undefined()
    }
    /// <https://dom.spec.whatwg.org/#dom-abortsignal-reason>
    fn Reason(&self, _cx: JSContext) -> JSVal {
        self.reason.get()
    }
    #[allow(unsafe_code)]
    /// <https://dom.spec.whatwg.org/#dom-abortsignal-throwifaborted>
    fn ThrowIfAborted(&self) {
        let reason = self.reason.get();
        if !reason.is_undefined() {
            let cx = *GlobalScope::get_cx();
            unsafe {
                assert!(!JS_IsExceptionPending(cx));
                rooted!(in(cx) let mut thrown = UndefinedValue());
                reason.to_jsval(cx, thrown.handle_mut());
                JS_SetPendingException(cx, thrown.handle(), ExceptionStackBehavior::Capture);
            }
        }
    }
}
