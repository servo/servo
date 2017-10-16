/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{DevtoolsPageInfo, ScriptToDevtoolsControlMsg};
use dom::abstractworker::{SharedRt, SimpleWorkerErrorHandler};
use dom::abstractworker::WorkerScriptMsg;
use dom::bindings::codegen::Bindings::WorkerBinding;
use dom::bindings::codegen::Bindings::WorkerBinding::WorkerMethods;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::messageevent::MessageEvent;
use dom::workerglobalscope::prepare_workerscope_init;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use js::jsapi::{HandleValue, JSAutoCompartment, JSContext};
use js::jsval::UndefinedValue;
use script_traits::WorkerScriptLoadOrigin;
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use task::TaskOnce;

pub type TrustedWorkerAddress = Trusted<Worker>;

// https://html.spec.whatwg.org/multipage/#worker
#[dom_struct]
pub struct Worker {
    eventtarget: EventTarget,
    #[ignore_heap_size_of = "Defined in std"]
    /// Sender to the Receiver associated with the DedicatedWorkerGlobalScope
    /// this Worker created.
    sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
    closing: Arc<AtomicBool>,
    #[ignore_heap_size_of = "Defined in rust-mozjs"]
    runtime: Arc<Mutex<Option<SharedRt>>>,
    terminated: Cell<bool>,
}

impl Worker {
    fn new_inherited(sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
                     closing: Arc<AtomicBool>) -> Worker {
        Worker {
            eventtarget: EventTarget::new_inherited(),
            sender: sender,
            closing: closing,
            runtime: Arc::new(Mutex::new(None)),
            terminated: Cell::new(false),
        }
    }

    pub fn new(global: &GlobalScope,
               sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
               closing: Arc<AtomicBool>) -> DomRoot<Worker> {
        reflect_dom_object(Box::new(Worker::new_inherited(sender, closing)),
                           global,
                           WorkerBinding::Wrap)
    }

    // https://html.spec.whatwg.org/multipage/#dom-worker
    #[allow(unsafe_code)]
    pub fn Constructor(global: &GlobalScope, script_url: DOMString) -> Fallible<DomRoot<Worker>> {
        // Step 2-4.
        let worker_url = match global.api_base_url().join(&script_url) {
            Ok(url) => url,
            Err(_) => return Err(Error::Syntax),
        };

        let (sender, receiver) = channel();
        let closing = Arc::new(AtomicBool::new(false));
        let worker = Worker::new(global, sender.clone(), closing.clone());
        let worker_ref = Trusted::new(&*worker);

        let worker_load_origin = WorkerScriptLoadOrigin {
            referrer_url: None,
            referrer_policy: None,
            pipeline_id: Some(global.pipeline_id()),
        };

        let (devtools_sender, devtools_receiver) = ipc::channel().unwrap();
        let worker_id = global.get_next_worker_id();
        if let Some(ref chan) = global.devtools_chan() {
            let pipeline_id = global.pipeline_id();
                let title = format!("Worker for {}", worker_url);
                let page_info = DevtoolsPageInfo {
                    title: title,
                    url: worker_url.clone(),
                };
                let _ = chan.send(ScriptToDevtoolsControlMsg::NewGlobal((pipeline_id, Some(worker_id)),
                                                                devtools_sender.clone(),
                                                                page_info));
        }

        let init = prepare_workerscope_init(global, Some(devtools_sender));

        DedicatedWorkerGlobalScope::run_worker_scope(
            init, worker_url, devtools_receiver, worker.runtime.clone(), worker_ref,
            global.script_chan(), sender, receiver, worker_load_origin, closing);

        Ok(worker)
    }

    pub fn is_closing(&self) -> bool {
        self.closing.load(Ordering::SeqCst)
    }

    pub fn is_terminated(&self) -> bool {
        self.terminated.get()
    }

    pub fn handle_message(address: TrustedWorkerAddress,
                          data: StructuredCloneData) {
        let worker = address.root();

        if worker.is_terminated() {
            return;
        }

        let global = worker.global();
        let target = worker.upcast();
        let _ac = JSAutoCompartment::new(global.get_cx(), target.reflector().get_jsobject().get());
        rooted!(in(global.get_cx()) let mut message = UndefinedValue());
        data.read(&global, message.handle_mut());
        MessageEvent::dispatch_jsval(target, &global, message.handle());
    }

    pub fn dispatch_simple_error(address: TrustedWorkerAddress) {
        let worker = address.root();
        worker.upcast().fire_event(atom!("error"));
    }
}

impl WorkerMethods for Worker {
    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-worker-postmessage
    unsafe fn PostMessage(&self, cx: *mut JSContext, message: HandleValue) -> ErrorResult {
        let data = StructuredCloneData::write(cx, message)?;
        let address = Trusted::new(self);

        // NOTE: step 9 of https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage
        // indicates that a nonexistent communication channel should result in a silent error.
        let _ = self.sender.send((address, WorkerScriptMsg::DOMMessage(data)));
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#terminate-a-worker
    fn Terminate(&self) {
        // Step 1
        if self.closing.swap(true, Ordering::SeqCst) {
            return;
        }

        // Step 2
        self.terminated.set(true);

        // Step 3
        if let Some(runtime) = *self.runtime.lock().unwrap() {
            runtime.request_interrupt();
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-worker-onmessage
    event_handler!(message, GetOnmessage, SetOnmessage);

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-onerror
    event_handler!(error, GetOnerror, SetOnerror);
}

impl TaskOnce for SimpleWorkerErrorHandler<Worker> {
    #[allow(unrooted_must_root)]
    fn run_once(self) {
        Worker::dispatch_simple_error(self.addr);
    }
}
