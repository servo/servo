/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{ExceptionStackBehavior, Heap, JS_IsExceptionPending};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::wrappers::JS_SetPendingException;
use js::rust::HandleValue;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AbortSignalBinding::AbortSignalMethods;
use crate::dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use crate::dom::bindings::codegen::Bindings::EventTargetBinding::EventListenerOptions;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
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
}
impl AbortAlgorithm {
    fn exec(self) {
        match self {
            Self::RemoveEventListener(target, ty, listener, options) => {
                target.remove_event_listener(ty, Some(listener), options)
            },
        }
    }
}

#[dom_struct]
pub struct AbortSignal {
    event_target: EventTarget,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    reason: Heap<JSVal>,
    abort_algorithms: DomRefCell<Vec<AbortAlgorithm>>,
}

impl AbortSignal {
    pub fn new_inherited() -> Self {
        Self {
            event_target: EventTarget::new_inherited(),
            reason: Heap::default(),
            abort_algorithms: DomRefCell::default(),
        }
    }
    pub fn new(global: &GlobalScope) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited()), global)
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
