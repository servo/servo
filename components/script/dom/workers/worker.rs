/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam_channel::{Sender, unbounded};
use devtools_traits::{DevtoolsPageInfo, ScriptToDevtoolsControlMsg, WorkerId};
use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::{Heap, JSObject};
use js::jsval::UndefinedValue;
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleObject, HandleValue};
use net_traits::request::Referrer;
use servo_base::generic_channel;
use servo_constellation_traits::{StructuredSerializedData, WorkerScriptLoadOrigin};
use uuid::Uuid;

use crate::dom::abstractworker::{MessageData, SimpleWorkerErrorHandler, WorkerScriptMsg};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::StructuredSerializeOptions;
use crate::dom::bindings::codegen::Bindings::WorkerBinding::{WorkerMethods, WorkerOptions};
use crate::dom::bindings::codegen::UnionTypes::TrustedScriptURLOrUSVString;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::{CustomTraceable, RootedTraceableBox};
use crate::dom::dedicatedworkerglobalscope::{
    DedicatedWorkerGlobalScope, DedicatedWorkerScriptMsg,
};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::dom::trustedtypes::trustedscripturl::TrustedScriptURL;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::prepare_workerscope_init;
use crate::realms::enter_auto_realm;
use crate::script_runtime::{CanGc, ThreadSafeJSContext};
use crate::task::TaskOnce;
use crate::url::ensure_blob_referenced_by_url_is_kept_alive;

pub(crate) type TrustedWorkerAddress = Trusted<Worker>;

// https://html.spec.whatwg.org/multipage/#worker
#[dom_struct]
pub(crate) struct Worker {
    eventtarget: EventTarget,
    /// Sender to the Receiver associated with the DedicatedWorkerGlobalScope
    /// this Worker created.
    sender: Sender<DedicatedWorkerScriptMsg>,
    #[conditional_malloc_size_of]
    closing: Arc<AtomicBool>,
    terminated: Cell<bool>,
    #[ignore_malloc_size_of = "mozjs"]
    #[no_trace]
    context_for_interrupt: DomRefCell<Option<ThreadSafeJSContext>>,
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
        can_gc: CanGc,
    ) -> DomRoot<Worker> {
        reflect_dom_object_with_proto(
            Box::new(Worker::new_inherited(sender, closing)),
            global,
            proto,
            can_gc,
        )
    }

    pub(crate) fn is_terminated(&self) -> bool {
        self.terminated.get()
    }

    pub(crate) fn set_context_for_interrupt(&self, cx: ThreadSafeJSContext) {
        assert!(
            self.context_for_interrupt.borrow().is_none(),
            "Context for interrupt must be set only once"
        );
        *self.context_for_interrupt.borrow_mut() = Some(cx);
    }

    pub(crate) fn handle_message(
        address: TrustedWorkerAddress,
        data: StructuredSerializedData,
        cx: &mut JSContext,
    ) {
        let worker = address.root();

        if worker.is_terminated() {
            return;
        }

        let global = worker.global();
        let target = worker.upcast();
        let mut realm = enter_auto_realm(cx, target);
        let cx = &mut realm.current_realm();
        rooted!(&in(cx) let mut message = UndefinedValue());
        if let Ok(ports) =
            structuredclone::read(&global, data, message.handle_mut(), CanGc::from_cx(cx))
        {
            MessageEvent::dispatch_jsval(
                target,
                &global,
                message.handle(),
                None,
                None,
                ports,
                CanGc::from_cx(cx),
            );
        } else {
            // Step 4 of the "port post message steps" of the implicit messageport, fire messageerror.
            MessageEvent::dispatch_error(target, &global, CanGc::from_cx(cx));
        }
    }

    pub(crate) fn dispatch_simple_error(address: TrustedWorkerAddress, can_gc: CanGc) {
        let worker = address.root();
        worker.upcast().fire_event(atom!("error"), can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-postmessage>
    fn post_message_impl(
        &self,
        cx: &mut JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        let data = structuredclone::write(cx.into(), message, Some(transfer))?;
        let address = Trusted::new(self);

        // NOTE: step 9 of https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage
        // indicates that a nonexistent communication channel should result in a silent error.
        let _ = self.sender.send(DedicatedWorkerScriptMsg::CommonWorker(
            address,
            WorkerScriptMsg::DOMMessage(MessageData {
                origin: self.global().origin().immutable().clone(),
                data: Box::new(data),
            }),
        ));
        Ok(())
    }
}

impl WorkerMethods<crate::DomTypeHolder> for Worker {
    /// <https://html.spec.whatwg.org/multipage/#dom-worker>
    fn Constructor(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        script_url: TrustedScriptURLOrUSVString,
        worker_options: &WorkerOptions,
    ) -> Fallible<DomRoot<Worker>> {
        // Step 1: Let compliantScriptURL be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedScriptURL,
        // this's relevant global object, scriptURL, "Worker constructor", and "script".
        let compliant_script_url = TrustedScriptURL::get_trusted_type_compliant_string(
            cx,
            global,
            script_url,
            "Worker constructor",
        )?;
        // Step 2. Let outsideSettings be this's relevant settings object.
        // Step 3. Let workerURL be the result of encoding-parsing a URL given compliantScriptURL,
        // relative to outsideSettings.
        // TODO: Locking the URL should eventually happen inside encoding_parse_a_url, since most callers
        // will expect their blobs to be kept alive...
        let Ok(worker_url) = global
            .encoding_parse_a_url(&compliant_script_url.str())
            .map(|url| ensure_blob_referenced_by_url_is_kept_alive(global, url))
        else {
            // Step 4. If workerURL is failure, then throw a "SyntaxError" DOMException.
            return Err(Error::Syntax(None));
        };

        let (sender, receiver) = unbounded();
        let closing = Arc::new(AtomicBool::new(false));
        let worker = Worker::new(
            global,
            proto,
            sender.clone(),
            closing.clone(),
            CanGc::from_cx(cx),
        );
        let worker_ref = Trusted::new(&*worker);

        let worker_load_origin = WorkerScriptLoadOrigin {
            referrer_url: match global.get_referrer() {
                Referrer::Client(url) => Some(url),
                Referrer::ReferrerUrl(url) => Some(url),
                _ => None,
            },
            referrer_policy: global.get_referrer_policy(),
            pipeline_id: global.pipeline_id(),
        };

        let webview_id = global
            .webview_id()
            .expect("Global object must have a WebViewId");
        let browsing_context = global
            .downcast::<Window>()
            .map(|w| w.window_proxy().browsing_context_id())
            .or_else(|| {
                global
                    .downcast::<DedicatedWorkerGlobalScope>()
                    .and_then(|w| w.browsing_context())
            });

        let (devtools_sender, devtools_receiver) = generic_channel::channel().unwrap();
        let worker_id = WorkerId(Uuid::new_v4());
        if let Some(chan) = global.devtools_chan() {
            let pipeline_id = global.pipeline_id();
            let title = format!("Worker for {}", worker_url.url());
            if let Some(browsing_context) = browsing_context {
                let page_info = DevtoolsPageInfo {
                    title,
                    url: worker_url.url(),
                    is_top_level_global: false,
                    is_service_worker: false,
                };
                let _ = chan.send(ScriptToDevtoolsControlMsg::NewGlobal(
                    (browsing_context, pipeline_id, Some(worker_id), webview_id),
                    devtools_sender.clone(),
                    page_info,
                ));
            }
        }

        let webgl_chan = global
            .downcast::<Window>()
            .and_then(|window| window.webgl_chan_value());
        let init =
            prepare_workerscope_init(global, Some(devtools_sender), Some(worker_id), webgl_chan);

        let (control_sender, control_receiver) = unbounded();
        let (context_sender, context_receiver) = unbounded();

        let event_loop_sender = global
            .event_loop_sender()
            .expect("Tried to create a worker in a worker while not handling a message?");
        let join_handle = DedicatedWorkerGlobalScope::run_worker_scope(
            init,
            webview_id,
            worker_url,
            devtools_receiver,
            worker_ref,
            event_loop_sender,
            sender,
            receiver,
            worker_load_origin,
            worker_options,
            closing.clone(),
            global.image_cache(),
            browsing_context,
            #[cfg(feature = "webgpu")]
            global.wgpu_id_hub(),
            control_receiver,
            context_sender,
            global.insecure_requests_policy(),
            global.policy_container(),
            global.font_context().cloned(),
        );

        let context = context_receiver
            .recv()
            .expect("Couldn't receive a context for worker.");

        worker.set_context_for_interrupt(context.clone());
        global.track_worker(closing, join_handle, control_sender, context);

        Ok(worker)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-worker-postmessage>
    fn PostMessage(
        &self,
        cx: &mut JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        self.post_message_impl(cx, message, transfer)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-worker-postmessage>
    fn PostMessage_(
        &self,
        cx: &mut JSContext,
        message: HandleValue,
        options: RootedTraceableBox<StructuredSerializeOptions>,
    ) -> ErrorResult {
        let mut rooted = CustomAutoRooter::new(
            options
                .transfer
                .iter()
                .map(|js: &RootedTraceableBox<Heap<*mut JSObject>>| js.get())
                .collect(),
        );
        #[expect(unsafe_code)]
        let guard = unsafe { CustomAutoRooterGuard::new(cx.raw_cx(), &mut rooted) };
        self.post_message_impl(cx, message, guard)
    }

    /// <https://html.spec.whatwg.org/multipage/#terminate-a-worker>
    fn Terminate(&self) {
        // Step 1
        if self.closing.swap(true, Ordering::SeqCst) {
            return;
        }

        // Step 2
        self.terminated.set(true);

        // Step 3
        if let Some(cx) = self.context_for_interrupt.borrow().as_ref() {
            cx.request_interrupt_callback()
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
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn run_once(self, cx: &mut JSContext) {
        Worker::dispatch_simple_error(self.addr, CanGc::from_cx(cx));
    }
}
