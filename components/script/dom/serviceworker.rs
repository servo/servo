/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::abstractworker::WorkerScriptMsg;
use dom::abstractworker::{SimpleWorkerErrorHandler, WorkerErrorHandler};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::ServiceWorkerBinding::{ServiceWorkerMethods, ServiceWorkerState, Wrap};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::errorevent::ErrorEvent;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use js::jsval::UndefinedValue;
use script_thread::Runnable;
use std::cell::Cell;
use std::sync::mpsc::Sender;
use url::Url;

pub type TrustedServiceWorkerAddress = Trusted<ServiceWorker>;

#[dom_struct]
pub struct ServiceWorker {
    eventtarget: EventTarget,
    script_url: DOMRefCell<String>,
    state: Cell<ServiceWorkerState>,
    #[ignore_heap_size_of = "Defined in std"]
    sender: Option<Sender<(TrustedServiceWorkerAddress, WorkerScriptMsg)>>,
    skip_waiting: Cell<bool>
}

impl ServiceWorker {
    fn new_inherited(script_url: &str,
                     skip_waiting: bool) -> ServiceWorker {
        ServiceWorker {
            eventtarget: EventTarget::new_inherited(),
            sender: None,
            script_url: DOMRefCell::new(String::from(script_url)),
            state: Cell::new(ServiceWorkerState::Installing),
            skip_waiting: Cell::new(skip_waiting)
        }
    }

    pub fn new(global: GlobalRef,
                script_url: &str,
                skip_waiting: bool) -> Root<ServiceWorker> {
        reflect_dom_object(box ServiceWorker::new_inherited(script_url, skip_waiting), global, Wrap)
    }

    pub fn dispatch_simple_error(address: TrustedServiceWorkerAddress) {
        let service_worker = address.root();
        service_worker.upcast().fire_simple_event("error");
    }

    pub fn set_transition_state(&self, state: ServiceWorkerState) {
        self.state.set(state);
        self.upcast::<EventTarget>().fire_simple_event("statechange");
    }

    pub fn get_script_url(&self) -> Url {
        Url::parse(&self.script_url.borrow().clone()).unwrap()
    }

    pub fn handle_error_message(address: TrustedServiceWorkerAddress, message: DOMString,
                                filename: DOMString, lineno: u32, colno: u32) {
        let worker = address.root();

        let global = worker.r().global();
        rooted!(in(global.r().get_cx()) let error = UndefinedValue());
        let errorevent = ErrorEvent::new(global.r(), atom!("error"),
                                         EventBubbles::Bubbles, EventCancelable::Cancelable,
                                         message, filename, lineno, colno, error.handle());
        errorevent.upcast::<Event>().fire(worker.upcast());
    }

    pub fn install_serviceworker(global: GlobalRef,
                                 script_url: Url,
                                 skip_waiting: bool) -> Root<ServiceWorker> {
        ServiceWorker::new(global,
                           script_url.as_str(),
                           skip_waiting)
    }
}

impl ServiceWorkerMethods for ServiceWorker {
    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-state-attribute
    fn State(&self) -> ServiceWorkerState {
        self.state.get()
    }

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-url-attribute
    fn ScriptURL(&self) -> USVString {
        USVString(self.script_url.borrow().clone())
    }

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-container-onerror-attribute
    event_handler!(error, GetOnerror, SetOnerror);

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#ref-for-service-worker-onstatechange-attribute-1
    event_handler!(statechange, GetOnstatechange, SetOnstatechange);
}

impl Runnable for SimpleWorkerErrorHandler<ServiceWorker> {
    #[allow(unrooted_must_root)]
    fn handler(self: Box<SimpleWorkerErrorHandler<ServiceWorker>>) {
        let this = *self;
        ServiceWorker::dispatch_simple_error(this.addr);
    }
}

impl Runnable for WorkerErrorHandler<ServiceWorker> {
    #[allow(unrooted_must_root)]
    fn handler(self: Box<WorkerErrorHandler<ServiceWorker>>) {
        let this = *self;
        ServiceWorker::handle_error_message(this.addr, this.msg, this.file_name, this.line_num, this.col_num);
    }
}
