/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crossbeam_channel::{unbounded, Sender};
use devtools_traits::{DevtoolsPageInfo, ScriptToDevtoolsControlMsg, WorkerId};
use dom_struct::dom_struct;
use ipc_channel::ipc;
use js::jsapi::{Heap, JSObject};
use js::jsval::UndefinedValue;
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleObject, HandleValue};
use script_traits::{StructuredSerializedData, WorkerScriptLoadOrigin};
use uuid::Uuid;

use crate::dom::abstractworker::{SimpleWorkerErrorHandler, WorkerScriptMsg};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::PostMessageOptions;
use crate::dom::bindings::codegen::Bindings::WorkerBinding::{WorkerMethods, WorkerOptions};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::dedicatedworkerglobalscope::{
    DedicatedWorkerGlobalScope, DedicatedWorkerScriptMsg,
};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::prepare_workerscope_init;
use crate::realms::enter_realm;
use crate::script_runtime::{ContextForRequestInterrupt, JSContext};
use crate::task::TaskOnce;

pub type TrustedWorkerAddress = Trusted<Worker>;

// https://html.spec.whatwg.org/multipage/#worker
#[dom_struct]
pub struct Worker {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "Defined in std"]
    #[no_trace]
    /// Sender to the Receiver associated with the DedicatedWorkerGlobalScope
    /// this Worker created.
    sender: Sender<DedicatedWorkerScriptMsg>,
    #[ignore_malloc_size_of = "Arc"]
    closing: Arc<AtomicBool>,
    terminated: Cell<bool>,
    #[ignore_malloc_size_of = "Arc"]
    context_for_interrupt: DomRefCell<Option<ContextForRequestInterrupt>>,
}

impl Worker {
    fn new_inherited(sender: Sender<DedicatedWorkerScriptMsg>, closing: Arc<AtomicBool>) -> Worker {
        Worker {
            eventtarget: EventTarget::new_inherited(),
            sender,
            closing,
            terminated: Cell::new(false),
            context_for_interrupt: Default::default(),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        sender: Sender<DedicatedWorkerScriptMsg>,
        closing: Arc<AtomicBool>,
    ) -> DomRoot<Worker> {
        reflect_dom_object_with_proto(
            Box::new(Worker::new_inherited(sender, closing)),
            global,
            proto,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-worker
    #[allow(unsafe_code, non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        script_url: USVString,
        worker_options: &WorkerOptions,
    ) -> Fallible<DomRoot<Worker>> {
        // Step 2-4.
        let worker_url = match global.api_base_url().join(&script_url) {
            Ok(url) => url,
            Err(_) => return Err(Error::Syntax),
        };

        let (sender, receiver) = unbounded();
        let closing = Arc::new(AtomicBool::new(false));
        let worker = Worker::new(global, proto, sender.clone(), closing.clone());
        let worker_ref = Trusted::new(&*worker);

        let worker_load_origin = WorkerScriptLoadOrigin {
            referrer_url: None,
            referrer_policy: None,
            pipeline_id: global.pipeline_id(),
        };

        let browsing_context = global
            .downcast::<Window>()
            .map(|w| w.window_proxy().browsing_context_id())
            .or_else(|| {
                global
                    .downcast::<DedicatedWorkerGlobalScope>()
                    .and_then(|w| w.browsing_context())
            });

        let (devtools_sender, devtools_receiver) = ipc::channel().unwrap();
        let worker_id = WorkerId(Uuid::new_v4());
        if let Some(chan) = global.devtools_chan() {
            let pipeline_id = global.pipeline_id();
            let title = format!("Worker for {}", worker_url);
            if let Some(browsing_context) = browsing_context {
                let page_info = DevtoolsPageInfo {
                    title,
                    url: worker_url.clone(),
                };
                let _ = chan.send(ScriptToDevtoolsControlMsg::NewGlobal(
                    (browsing_context, pipeline_id, Some(worker_id)),
                    devtools_sender.clone(),
                    page_info,
                ));
            }
        }

        let init = prepare_workerscope_init(global, Some(devtools_sender), Some(worker_id));

        let (control_sender, control_receiver) = unbounded();
        let (context_sender, context_receiver) = unbounded();

        let join_handle = DedicatedWorkerGlobalScope::run_worker_scope(
            init,
            worker_url,
            devtools_receiver,
            worker_ref,
            global.script_chan(),
            sender,
            receiver,
            worker_load_origin,
            String::from(&*worker_options.name),
            worker_options.type_,
            closing.clone(),
            global.image_cache(),
            browsing_context,
            global.wgpu_id_hub(),
            control_receiver,
            context_sender,
        );

        let context = context_receiver
            .recv()
            .expect("Couldn't receive a context for worker.");

        worker.set_context_for_interrupt(context.clone());
        global.track_worker(closing, join_handle, control_sender, context);

        Ok(worker)
    }

    pub fn is_terminated(&self) -> bool {
        self.terminated.get()
    }

    pub fn set_context_for_interrupt(&self, cx: ContextForRequestInterrupt) {
        assert!(
            self.context_for_interrupt.borrow().is_none(),
            "Context for interrupt must be set only once"
        );
        *self.context_for_interrupt.borrow_mut() = Some(cx);
    }

    pub fn handle_message(address: TrustedWorkerAddress, data: StructuredSerializedData) {
        let worker = address.root();

        if worker.is_terminated() {
            return;
        }

        let global = worker.global();
        let target = worker.upcast();
        let _ac = enter_realm(target);
        rooted!(in(*GlobalScope::get_cx()) let mut message = UndefinedValue());
        if let Ok(ports) = structuredclone::read(&global, data, message.handle_mut()) {
            MessageEvent::dispatch_jsval(target, &global, message.handle(), None, None, ports);
        } else {
            // Step 4 of the "port post message steps" of the implicit messageport, fire messageerror.
            MessageEvent::dispatch_error(target, &global);
        }
    }

    pub fn dispatch_simple_error(address: TrustedWorkerAddress) {
        let worker = address.root();
        worker.upcast().fire_event(atom!("error"));
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-postmessage>
    fn post_message_impl(
        &self,
        cx: JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        let data = structuredclone::write(cx, message, Some(transfer))?;
        let address = Trusted::new(self);

        // NOTE: step 9 of https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage
        // indicates that a nonexistent communication channel should result in a silent error.
        let _ = self.sender.send(DedicatedWorkerScriptMsg::CommonWorker(
            address,
            WorkerScriptMsg::DOMMessage {
                origin: self.global().origin().immutable().clone(),
                data,
            },
        ));
        Ok(())
    }
}

impl WorkerMethods for Worker {
    /// <https://html.spec.whatwg.org/multipage/#dom-worker-postmessage>
    fn PostMessage(
        &self,
        cx: JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        self.post_message_impl(cx, message, transfer)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-worker-postmessage>
    fn PostMessage_(
        &self,
        cx: JSContext,
        message: HandleValue,
        options: RootedTraceableBox<PostMessageOptions>,
    ) -> ErrorResult {
        let mut rooted = CustomAutoRooter::new(
            options
                .transfer
                .iter()
                .map(|js: &RootedTraceableBox<Heap<*mut JSObject>>| js.get())
                .collect(),
        );
        let guard = CustomAutoRooterGuard::new(*cx, &mut rooted);
        self.post_message_impl(cx, message, guard)
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
        if let Some(cx) = self.context_for_interrupt.borrow().as_ref() {
            cx.request_interrupt()
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-worker-onmessage
    event_handler!(message, GetOnmessage, SetOnmessage);

    // https://html.spec.whatwg.org/multipage/#handler-worker-onmessageerror
    event_handler!(messageerror, GetOnmessageerror, SetOnmessageerror);

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-onerror
    event_handler!(error, GetOnerror, SetOnerror);
}

impl TaskOnce for SimpleWorkerErrorHandler<Worker> {
    #[allow(crown::unrooted_must_root)]
    fn run_once(self) {
        Worker::dispatch_simple_error(self.addr);
    }
}
