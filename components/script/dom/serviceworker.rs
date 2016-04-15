/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::abstractworker::{SimpleWorkerErrorHandler, WorkerErrorHandler};
use dom::abstractworker::{WorkerScriptMsg, WorkerScriptLoadOrigin, SharedRt};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::ServiceWorkerBinding::{ServiceWorkerMethods, ServiceWorkerState, Wrap};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::client::Client;
use dom::errorevent::ErrorEvent;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use dom::workerglobalscope::prepare_workerscope_init;
use ipc_channel::ipc;
use js::jsapi::RootedValue;
use js::jsval::UndefinedValue;
use script_thread::Runnable;
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use url::Url;

pub type TrustedServiceWorkerAddress = Trusted<ServiceWorker>;

#[dom_struct]
pub struct ServiceWorker {
    eventtarget: EventTarget,
    script_url: DOMRefCell<String>,
    state: Cell<ServiceWorkerState>,
    closing: Arc<AtomicBool>,
    #[ignore_heap_size_of = "Defined in std"]
    sender: Sender<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
    #[ignore_heap_size_of = "Defined in rust-mozjs"]
    runtime: Arc<Mutex<Option<SharedRt>>>,
    skip_waiting: Cell<bool>
}

impl ServiceWorker {
    fn new_inherited(sender: Sender<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
                     closing: Arc<AtomicBool>,
                     script_url: &str,
                     skip_waiting: bool) -> ServiceWorker {
        ServiceWorker {
            eventtarget: EventTarget::new_inherited(),
            closing: closing,
            sender: sender,
            script_url: DOMRefCell::new(String::from(script_url)),
            state: Cell::new(ServiceWorkerState::Installing),
            runtime: Arc::new(Mutex::new(None)),
            skip_waiting: Cell::new(skip_waiting)
        }
    }

    pub fn new(global: GlobalRef,
                closing: Arc<AtomicBool>,
                sender: Sender<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
                script_url: &str,
                skip_waiting: bool) -> Root<ServiceWorker> {
        reflect_dom_object(box ServiceWorker::new_inherited(sender, closing, script_url, skip_waiting), global, Wrap)
    }

    pub fn dispatch_simple_error(address: TrustedServiceWorkerAddress) {
        let service_worker = address.root();
        service_worker.upcast().fire_simple_event("error");
    }

    pub fn is_closing(&self) -> bool {
        self.closing.load(Ordering::SeqCst)
    }

    pub fn set_transition_state(&self, state: ServiceWorkerState) {
        self.state.set(state);
        self.upcast::<EventTarget>().fire_simple_event("statechange");
    }

    pub fn handle_error_message(address: TrustedServiceWorkerAddress, message: DOMString,
                                filename: DOMString, lineno: u32, colno: u32) {
        let worker = address.root();

        if worker.is_closing() {
            return;
        }

        let global = worker.r().global();
        let error = RootedValue::new(global.r().get_cx(), UndefinedValue());
        let errorevent = ErrorEvent::new(global.r(), atom!("error"),
                                         EventBubbles::Bubbles, EventCancelable::Cancelable,
                                         message, filename, lineno, colno, error.handle());
        errorevent.upcast::<Event>().fire(worker.upcast());
    }

    #[allow(unsafe_code)]
    pub fn init_service_worker(global: GlobalRef,
                               script_url: Url,
                               skip_waiting: bool) -> Root<ServiceWorker> {
        let (sender, receiver) = channel();
        let closing = Arc::new(AtomicBool::new(false));
        let worker = ServiceWorker::new(global,
                                        closing.clone(),
                                        sender.clone(),
                                        script_url.as_str(),
                                        skip_waiting);
        let worker_ref = Trusted::new(worker.r());

        let worker_load_origin = WorkerScriptLoadOrigin {
            referrer_url: None,
            referrer_policy: None,
            request_source: global.request_source(),
            pipeline_id: Some(global.pipeline())
        };

        let (devtools_sender, devtools_receiver) = ipc::channel().unwrap();
        let init = prepare_workerscope_init(global,
            "Service Worker".to_owned(),
            script_url.clone(),
            devtools_sender.clone(),
            closing);

        // represents a service worker client
        let sw_client = Client::new(global.as_window());
        let trusted_client = Trusted::new(&*sw_client);

        ServiceWorkerGlobalScope::run_serviceworker_scope(
            init, script_url, global.pipeline(), devtools_receiver, worker.runtime.clone(), worker_ref,
            global.script_chan(), sender, receiver, trusted_client, worker_load_origin);

        worker
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
