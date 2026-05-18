/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crossbeam_channel::{Sender, unbounded};
use devtools_traits::{DevtoolsPageInfo, ScriptToDevtoolsControlMsg, WorkerId};
use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use net_traits::request::Referrer;
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use servo_base::generic_channel;
use servo_constellation_traits::WorkerScriptLoadOrigin;
use uuid::Uuid;

use crate::conversions::Convert;
use crate::dom::abstractworker::SimpleWorkerErrorHandler;
use crate::dom::bindings::codegen::Bindings::SharedWorkerBinding::{
    SharedWorkerMethods, SharedWorkerOptions,
};
use crate::dom::bindings::codegen::UnionTypes::{
    StringOrSharedWorkerOptions, TrustedScriptURLOrUSVString,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::CustomTraceable;
use crate::dom::bindings::transferable::Transferable;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::dom::sharedworkerglobalscope::{
    SharedWorkerControlMsg, SharedWorkerGlobalScope, SharedWorkerScriptMsg,
};
use crate::dom::trustedtypes::trustedscripturl::TrustedScriptURL;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::prepare_workerscope_init;
use crate::script_runtime::CanGc;
use crate::task::TaskOnce;
use crate::url::ensure_blob_referenced_by_url_is_kept_alive;

/// <https://html.spec.whatwg.org/multipage/#shared-workers-and-the-sharedworker-interface>
#[dom_struct]
pub(crate) struct SharedWorker {
    eventtarget: EventTarget,
    port: Dom<MessagePort>,
    _control_sender: Sender<SharedWorkerControlMsg>,
}

pub(crate) type TrustedSharedWorkerAddress = Trusted<SharedWorker>;

impl SharedWorker {
    fn new_inherited(
        port: &MessagePort,
        control_sender: Sender<SharedWorkerControlMsg>,
    ) -> SharedWorker {
        SharedWorker {
            eventtarget: EventTarget::new_inherited(),
            port: Dom::from_ref(port),
            _control_sender: control_sender,
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        port: &MessagePort,
        control_sender: Sender<SharedWorkerControlMsg>,
        cx: &mut js::context::JSContext,
    ) -> DomRoot<SharedWorker> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(SharedWorker::new_inherited(port, control_sender)),
            global,
            proto,
            cx,
        )
    }

    pub(crate) fn dispatch_simple_error(cx: &mut JSContext, address: TrustedSharedWorkerAddress) {
        let worker = address.root();
        worker.upcast().fire_event(cx, atom!("error"));
    }

    /// Step 11 of onComplete of <https://html.spec.whatwg.org/multipage/#run-a-worker>
    pub(crate) fn enable_outside_port_message_queue(
        address: TrustedSharedWorkerAddress,
        cx: &mut JSContext,
    ) {
        let worker = address.root();
        let global = worker.global();
        // Enable outside port's port message queue.
        global.start_message_port(cx, worker.port.message_port_id());
    }
}

impl SharedWorkerMethods<crate::DomTypeHolder> for SharedWorker {
    /// <https://html.spec.whatwg.org/multipage/#dom-sharedworker>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        script_url: TrustedScriptURLOrUSVString,
        options: StringOrSharedWorkerOptions,
    ) -> Fallible<DomRoot<SharedWorker>> {
        let global = window.upcast::<GlobalScope>();

        // Step 1. Let compliantScriptURL be the result of invoking the get trusted type
        // compliant string algorithm with TrustedScriptURL, this's relevant global object,
        // scriptURL, "SharedWorker constructor", and "script".
        let compliant_script_url = TrustedScriptURL::get_trusted_type_compliant_string(
            cx,
            global,
            script_url,
            "SharedWorker constructor",
        )?;

        // Step 2. If options is a DOMString, set options to a new WorkerOptions
        // dictionary whose name member is set to the value of options and whose other
        // members are set to their default values.
        let worker_options = match options {
            StringOrSharedWorkerOptions::String(name) => {
                let mut options = SharedWorkerOptions::empty();
                options.parent.name = name;
                options
            },
            StringOrSharedWorkerOptions::SharedWorkerOptions(options) => options,
        };
        let worker_name = worker_options.parent.name.clone();
        let worker_type = worker_options.parent.type_;
        let credentials = worker_options.parent.credentials.convert();

        // Step 3. Let outsideSettings be this's relevant settings object.
        // (outsideSettings is `global` throughout.)

        // Step 4. Let urlRecord be the result of encoding-parsing a URL given
        // compliantScriptURL, relative to outsideSettings.
        // Step 5. If urlRecord is failure, then throw a "SyntaxError" DOMException.
        let Ok(worker_url) = global
            .encoding_parse_a_url(&compliant_script_url.str())
            .map(|url| ensure_blob_referenced_by_url_is_kept_alive(global, url))
        else {
            return Err(Error::Syntax(None));
        };

        // Step 6. Let outsidePort be a new MessagePort in outsideSettings's realm.
        let outside_port = MessagePort::new(global, CanGc::from_cx(cx));
        global.track_message_port(&outside_port, None);

        let (control_sender, control_receiver) = unbounded();

        // Step 7. Set this's port to outsidePort.
        // Step 8. Let callerIsSecureContext be true if outsideSettings is a secure
        // context; otherwise, false.
        let _caller_is_secure_context = global.is_secure_context();
        // TODO Step 9.
        // Step 10. Let worker be this.
        let worker = SharedWorker::new(global, proto, &outside_port, control_sender, cx);
        let worker_addr = Trusted::new(&*worker);
        let parent_event_loop_sender = global
            .event_loop_sender()
            .expect("Window global must have an event loop sender");

        let (sender, receiver) = unbounded();
        let closing = Arc::new(AtomicBool::new(false));

        // Step 11. Enqueue the following steps to the shared worker manager.
        // TODO Steps 12-16.
        // Until then, SharedWorker construction always takes the fresh-worker
        // branch below instead of reusing an existing SharedWorkerGlobalScope.
        // Step 17. Otherwise, in parallel, run a worker given worker, urlRecord,
        // outsideSettings, outsidePort, and options.
        let inside_port = MessagePort::new(global, CanGc::from_cx(cx));
        global.track_message_port(&inside_port, None);
        global.entangle_ports(
            *outside_port.message_port_id(),
            *inside_port.message_port_id(),
        );
        let (_, inside_port_impl) = inside_port.transfer(cx)?;

        let worker_load_origin = WorkerScriptLoadOrigin {
            referrer_url: match global.get_referrer() {
                Referrer::Client(url) => Some(url),
                Referrer::ReferrerUrl(url) => Some(url),
                _ => None,
            },
            referrer_policy: global.get_referrer_policy(),
            pipeline_id: global.pipeline_id(),
        };

        let (devtools_sender, devtools_receiver) = generic_channel::channel().unwrap();
        let worker_id = WorkerId(Uuid::new_v4());
        if let Some(chan) = global.devtools_chan() {
            let webview_id = global
                .webview_id()
                .expect("Window global must have a WebViewId");
            let page_info = DevtoolsPageInfo {
                title: format!("SharedWorker for {}", worker_url.url()),
                url: worker_url.url(),
                is_top_level_global: false,
                is_service_worker: false,
            };
            let _ = chan.send(ScriptToDevtoolsControlMsg::NewGlobal(
                (
                    window.window_proxy().browsing_context_id(),
                    global.pipeline_id(),
                    Some(worker_id),
                    webview_id,
                ),
                devtools_sender.clone(),
                page_info,
            ));
        }

        let init = prepare_workerscope_init(
            global,
            Some(devtools_sender),
            Some(worker_id),
            window.webgl_chan_value(),
        );

        let (context_sender, _context_receiver) = unbounded();

        let _join_handle = SharedWorkerGlobalScope::run_shared_worker_scope(
            init,
            worker_name,
            worker_type,
            worker_url,
            worker_addr,
            parent_event_loop_sender,
            devtools_receiver,
            sender.clone(),
            receiver,
            worker_load_origin,
            closing,
            #[cfg(feature = "webgpu")]
            global.wgpu_id_hub(),
            control_receiver,
            context_sender,
            credentials,
            global.insecure_requests_policy(),
            global.policy_container(),
            global.font_context().cloned(),
        );

        // Implementation hook for onComplete step 13: the receiving side fires
        // the connect event immediately, or defers it until execution-ready.
        sender
            .send(SharedWorkerScriptMsg::Connect(inside_port_impl))
            .expect("SharedWorker failed to receive its initial connection");

        Ok(worker)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-sharedworker-port>
    fn Port(&self) -> DomRoot<MessagePort> {
        // The port getter steps are to return this's port.
        DomRoot::from_ref(&*self.port)
    }

    // <https://html.spec.whatwg.org/multipage/#handler-abstractworker-onerror>
    event_handler!(error, GetOnerror, SetOnerror);
}

impl TaskOnce for SimpleWorkerErrorHandler<SharedWorker> {
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn run_once(self, cx: &mut JSContext) {
        SharedWorker::dispatch_simple_error(cx, self.addr);
    }
}
