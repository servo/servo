/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use crossbeam_channel::{Sender, unbounded};
use devtools_traits::{DevtoolsPageInfo, ScriptToDevtoolsControlMsg, WorkerId};
use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use net_traits::request::{CredentialsMode, Referrer};
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use servo_base::generic_channel;
use servo_constellation_traits::{MessagePortImpl, WorkerScriptLoadOrigin};
use servo_url::{ImmutableOrigin, ServoUrl};
use uuid::Uuid;

use crate::conversions::Convert;
use crate::dom::abstractworker::SimpleWorkerErrorHandler;
use crate::dom::bindings::codegen::Bindings::SharedWorkerBinding::{
    SharedWorkerMethods, SharedWorkerOptions,
};
use crate::dom::bindings::codegen::Bindings::WorkerBinding::WorkerType;
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

// A `SharedWorkerGlobalScope` object has associated constructor origin (an origin), constructor URL (a URL record), and credentials (a credentials mode), and extended lifetime (a boolean).
#[derive(Clone)]
struct SharedWorkerRegistration {
    id: Uuid,
    storage_key: ImmutableOrigin,
    constructor_origin: ImmutableOrigin,
    constructor_url: ServoUrl,
    name: String,
    worker_type: WorkerType,
    credentials: CredentialsMode,
    extended_lifetime: bool,
    worker_is_secure_context: bool,
    closing: Arc<AtomicBool>,
    sender: Sender<SharedWorkerScriptMsg>,
    _control_sender: Sender<SharedWorkerControlMsg>,
}

// A user agent has an associated shared worker manager which is the result of starting a new parallel queue.
// Each user agent has a single shared worker manager for simplicity.
static SHARED_WORKERS: LazyLock<Mutex<Vec<SharedWorkerRegistration>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// Implements the existing shared worker lookup from:
/// <https://html.spec.whatwg.org/multipage/#dom-sharedworker>
/// <https://html.spec.whatwg.org/multipage/#shared-worker-manager>
fn find_shared_worker(
    storage_key: &ImmutableOrigin,
    constructor_origin: &ImmutableOrigin,
    constructor_url: &ServoUrl,
    name: &str,
) -> Option<SharedWorkerRegistration> {
    let mut workers = SHARED_WORKERS
        .lock()
        .expect("SharedWorker registry poisoned");
    workers.retain(|worker| !worker.closing.load(Ordering::SeqCst));

    // Step 11.2. For each scope in the list of all `SharedWorkerGlobalScope` objects:
    // Step 11.2.1. Let workerStorageKey be the result of running obtain a storage key for non-storage purposes given scope's relevant settings object.
    // Step 11.2.2. If all of the following are true:
    // workerStorageKey equals outsideStorageKey;
    // scope's closing flag is false;
    // scope's constructor URL equals urlRecord; and
    // scope's name equals options["name"],
    // `data:` URLs create a worker with an opaque origin. Both the constructor origin and constructor URL are compared so the same `data:` URL can be used within an origin to get to the same `SharedWorkerGlobalScope` object, but cannot be used to bypass the same origin restriction.
    workers
        .iter()
        .find(|worker| {
            worker.storage_key == *storage_key &&
                worker.constructor_origin == *constructor_origin &&
                worker.constructor_url == *constructor_url &&
                worker.name == name
        })
        .cloned()
}

fn register_shared_worker(worker: SharedWorkerRegistration) {
    SHARED_WORKERS
        .lock()
        .expect("SharedWorker registry poisoned")
        .push(worker);
}

impl SharedWorker {
    pub(crate) fn unregister_shared_worker(id: Uuid) {
        SHARED_WORKERS
            .lock()
            .expect("SharedWorker registry poisoned")
            .retain(|worker| worker.id != id);
    }

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

    fn queue_simple_error(global: &GlobalScope, address: TrustedSharedWorkerAddress) {
        global.task_manager().dom_manipulation_task_source().queue(
            task!(sharedworker_constructor_error: move |cx| {
                SharedWorker::dispatch_simple_error(cx, address);
            }),
        );
    }

    fn create_entangled_inside_port(
        cx: &mut JSContext,
        global: &GlobalScope,
        outside_port: &MessagePort,
    ) -> Fallible<MessagePortImpl> {
        let inside_port = MessagePort::new(global, CanGc::from_cx(cx));
        global.track_message_port(&inside_port, None);
        global.entangle_ports(
            *outside_port.message_port_id(),
            *inside_port.message_port_id(),
        );
        let (_, inside_port_impl) = inside_port.transfer(cx)?;
        Ok(inside_port_impl)
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
        let extended_lifetime = worker_options.extendedLifetime;

        // Step 3. Let outsideSettings be this's relevant settings object.

        // Step 4. Let urlRecord be the result of encoding-parsing a URL given
        // compliantScriptURL, relative to outsideSettings.
        // Step 5. If urlRecord is failure, then throw a "SyntaxError" DOMException.
        let Ok(worker_url) = global
            .encoding_parse_a_url(&compliant_script_url.str())
            .map(|url| ensure_blob_referenced_by_url_is_kept_alive(global, url))
        else {
            return Err(Error::Syntax(None));
        };
        let constructor_origin = global.origin().immutable().clone();
        let constructor_url = worker_url.url();

        // Step 6. Let outsidePort be a new MessagePort in outsideSettings's realm.
        let outside_port = MessagePort::new(global, CanGc::from_cx(cx));
        global.track_message_port(&outside_port, None);

        // Step 7. Set this's port to outsidePort.
        // Step 8. Let callerIsSecureContext be true if outsideSettings is a secure
        // context; otherwise, false.
        let caller_is_secure_context = global.is_secure_context();
        // Step 9. Let outsideStorageKey be the result of running obtain a storage
        // key for non-storage purposes given outsideSettings.
        let outside_storage_key = global.obtain_storage_key_for_non_storage_purposes();

        let worker_name_string = worker_name.to_string();
        let (control_sender, control_receiver) = unbounded();

        // Step 10. Let worker be this.
        let worker = SharedWorker::new(global, proto, &outside_port, control_sender.clone(), cx);
        let worker_addr = Trusted::new(&*worker);

        // Step 11.1. Let workerGlobalScope be null.
        let shared_worker = find_shared_worker(
            &outside_storage_key,
            &constructor_origin,
            &constructor_url,
            &worker_name_string,
        );

        if let Some(registration) = shared_worker {
            // Step 11.2.2.1. Set workerGlobalScope to scope.
            // Step 11.2.2.2. Break.
            // Step 11.3. If workerGlobalScope is not null, but the user agent has been configured to disallow communication between the worker represented by the workerGlobalScope and the scripts whose settings object is outsideSettings, then set workerGlobalScope to null.
            // TODO Step 11.3.
            // Step 11.4. If workerGlobalScope is not null, and any of the following are true:
            // workerGlobalScope's type is not equal to options["type"];
            // workerGlobalScope's credentials is not equal to options["credentials"]; or
            // workerGlobalScope's extended lifetime is not equal to options["extendedLifetime"],
            if registration.worker_type != worker_type ||
                registration.credentials != credentials ||
                registration.extended_lifetime != extended_lifetime
            {
                // Step 11.4.1. Queue a global task on the DOM manipulation task source given worker's relevant global object to fire an event named error at worker.
                SharedWorker::queue_simple_error(global, worker_addr);
                // Step 11.4.2. Abort these steps.
                return Ok(worker);
            }

            // Step 11.5. If workerGlobalScope is not null:
            // Step 11.5.1. Let insideSettings be workerGlobalScope's relevant settings object.
            // Step 11.5.2. Let workerIsSecureContext be true if insideSettings is a secure context; otherwise, false.
            // Step 11.5.3. If workerIsSecureContext is not callerIsSecureContext:
            if registration.worker_is_secure_context != caller_is_secure_context {
                // Step 11.5.3.1. Queue a global task on the DOM manipulation task source given worker's relevant global object to fire an event named error at worker.
                SharedWorker::queue_simple_error(global, worker_addr);
                // Step 11.5.3.2. Abort these steps.
                return Ok(worker);
            }

            // Step 11.5.4. Associate worker with workerGlobalScope.
            // Step 11.5.5. Let insidePort be a new MessagePort in insideSettings's realm.
            // Step 11.5.6. Entangle outsidePort and insidePort.
            let inside_port_impl =
                SharedWorker::create_entangled_inside_port(cx, global, &outside_port)?;
            // Step 11.5.7. Queue a global task on the DOM manipulation task source given workerGlobalScope to fire an event named connect at workerGlobalScope, using MessageEvent, with the data attribute initialized to the empty string, the ports attribute initialized to a new frozen array containing only insidePort, and the source attribute initialized to insidePort.
            if registration
                .sender
                .send(SharedWorkerScriptMsg::Connect(inside_port_impl))
                .is_err()
            {
                SharedWorker::queue_simple_error(global, worker_addr);
            }
            // Step 11.5.8. Append the relevant owner to add given outsideSettings to workerGlobalScope's owner set.
            // TODO Step 11.5.8.
            return Ok(worker);
        }

        let parent_event_loop_sender = global
            .event_loop_sender()
            .expect("Window global must have an event loop sender");

        let (sender, receiver) = unbounded();
        let closing = Arc::new(AtomicBool::new(false));
        let registration_id = Uuid::new_v4();

        let inside_port_impl =
            SharedWorker::create_entangled_inside_port(cx, global, &outside_port)?;

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

        let (setup_sender, setup_receiver) = unbounded();
        let (registered_sender, registered_receiver) = unbounded();

        // Step 11.6. Otherwise, in parallel, run a worker given worker, urlRecord, outsideSettings, outsidePort, and options.
        let _join_handle = SharedWorkerGlobalScope::run_shared_worker_scope(
            init,
            worker_name,
            worker_type,
            worker_url,
            worker_addr.clone(),
            parent_event_loop_sender,
            devtools_receiver,
            sender.clone(),
            receiver,
            worker_load_origin,
            closing.clone(),
            #[cfg(feature = "webgpu")]
            global.wgpu_id_hub(),
            control_receiver,
            setup_sender,
            registered_receiver,
            registration_id,
            credentials,
            extended_lifetime,
            constructor_origin.clone(),
            constructor_url.clone(),
            outside_storage_key.clone(),
            global.insecure_requests_policy(),
            global.policy_container(),
            global.font_context().cloned(),
        );

        // Step 11.5.2. Let workerIsSecureContext be true if insideSettings is a secure context; otherwise, false.
        let Ok(worker_is_secure_context) = setup_receiver.recv() else {
            SharedWorker::queue_simple_error(global, worker_addr);
            return Ok(worker);
        };

        register_shared_worker(SharedWorkerRegistration {
            id: registration_id,
            storage_key: outside_storage_key,
            constructor_origin,
            constructor_url,
            name: worker_name_string,
            worker_type,
            credentials,
            extended_lifetime,
            worker_is_secure_context,
            closing,
            sender: sender.clone(),
            _control_sender: control_sender,
        });

        if registered_sender.send(()).is_err() {
            SharedWorker::unregister_shared_worker(registration_id);
            SharedWorker::queue_simple_error(global, worker_addr);
            return Ok(worker);
        }

        // Step 13. If is shared is true, then queue a global task on the DOM manipulation task source given worker global scope to fire an event named connect at worker global scope, using MessageEvent, with the data attribute initialized to the empty string, the ports attribute initialized to a new frozen array containing inside port, and the source attribute initialized to inside port.
        if sender
            .send(SharedWorkerScriptMsg::Connect(inside_port_impl))
            .is_err()
        {
            SharedWorker::unregister_shared_worker(registration_id);
            SharedWorker::queue_simple_error(global, worker_addr);
        }

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
