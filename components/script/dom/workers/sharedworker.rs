/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, LazyLock, Mutex};

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

/// Key used by the script-side SharedWorker registry.
#[derive(Clone)]
struct SharedWorkerKey {
    storage_key: ImmutableOrigin,
    constructor_origin: ImmutableOrigin,
    constructor_url: ServoUrl,
    name: String,
}

impl SharedWorkerKey {
    fn matches(
        &self,
        storage_key: &ImmutableOrigin,
        constructor_origin: &ImmutableOrigin,
        constructor_url: &ServoUrl,
        name: &str,
    ) -> bool {
        self.storage_key == *storage_key &&
            self.constructor_origin == *constructor_origin &&
            self.constructor_url == *constructor_url &&
            self.name == name
    }
}

enum SharedWorkerRegistryState {
    Creating { waiters: usize },
    Created(SharedWorkerRegistration),
    Failed { waiters: usize },
}

struct SharedWorkerRegistryEntry {
    key: SharedWorkerKey,
    state: SharedWorkerRegistryState,
}

// A `SharedWorkerGlobalScope` object has associated constructor origin (an origin), constructor URL (a URL record), and credentials (a credentials mode), and extended lifetime (a boolean).
#[derive(Clone)]
struct SharedWorkerRegistration {
    id: Uuid,
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
//
// TODO: Move this script-side approximation to a proper shared worker manager,
// likely constellation-owned.
static SHARED_WORKERS: LazyLock<(Mutex<Vec<SharedWorkerRegistryEntry>>, Condvar)> =
    LazyLock::new(|| (Mutex::new(Vec::new()), Condvar::new()));

// Servo-internal registry states used to serialize the SharedWorker constructor's
// manager lookup/create work and avoid duplicate SharedWorker creation. These
// are not spec states.
enum SharedWorkerClaimResult {
    Created(SharedWorkerRegistration),
    Claimed,
    Failed,
}

fn prune_closed_shared_workers(workers: &mut Vec<SharedWorkerRegistryEntry>) {
    workers.retain(|entry| match &entry.state {
        SharedWorkerRegistryState::Creating { .. } => true,
        SharedWorkerRegistryState::Created(worker) => !worker.closing.load(Ordering::SeqCst),
        SharedWorkerRegistryState::Failed { waiters } => *waiters > 0,
    });
}

fn find_matching_shared_worker(
    workers: &[SharedWorkerRegistryEntry],
    key: &SharedWorkerKey,
) -> Option<usize> {
    workers.iter().position(|entry| {
        entry.key.matches(
            &key.storage_key,
            &key.constructor_origin,
            &key.constructor_url,
            &key.name,
        )
    })
}

/// <https://html.spec.whatwg.org/multipage/#dom-sharedworker>
/// <https://html.spec.whatwg.org/multipage/#shared-worker-manager>
fn find_or_claim_shared_worker(key: SharedWorkerKey) -> SharedWorkerClaimResult {
    let (workers, ready) = &*SHARED_WORKERS;
    let mut workers = workers.lock().expect("SharedWorker registry poisoned");

    prune_closed_shared_workers(&mut workers);

    let Some(index) = find_matching_shared_worker(&workers, &key) else {
        workers.push(SharedWorkerRegistryEntry {
            key,
            state: SharedWorkerRegistryState::Creating { waiters: 0 },
        });
        return SharedWorkerClaimResult::Claimed;
    };

    match &mut workers[index].state {
        SharedWorkerRegistryState::Creating { waiters } => *waiters += 1,
        SharedWorkerRegistryState::Created(registration) => {
            return SharedWorkerClaimResult::Created(registration.clone());
        },
        SharedWorkerRegistryState::Failed { waiters } => *waiters += 1,
    };

    loop {
        workers = ready.wait(workers).expect("SharedWorker registry poisoned");

        let Some(index) = find_matching_shared_worker(&workers, &key) else {
            return SharedWorkerClaimResult::Failed;
        };

        match &mut workers[index].state {
            SharedWorkerRegistryState::Creating { .. } => {},
            SharedWorkerRegistryState::Created(registration) => {
                return SharedWorkerClaimResult::Created(registration.clone());
            },
            SharedWorkerRegistryState::Failed { waiters } => {
                debug_assert!(*waiters > 0);
                if *waiters > 0 {
                    *waiters -= 1;
                }
                if *waiters == 0 {
                    workers.remove(index);
                    // No notify needed here: this waiter has already observed
                    // the failure and no waiter remains blocked on the condvar.
                }
                return SharedWorkerClaimResult::Failed;
            },
        }
    }
}

fn transition_creating_to_created(
    key: &SharedWorkerKey,
    registration: SharedWorkerRegistration,
) -> bool {
    let (workers, ready) = &*SHARED_WORKERS;
    let mut workers = workers.lock().expect("SharedWorker registry poisoned");
    let index = find_matching_shared_worker(&workers, key);
    debug_assert!(index.is_some(), "claimed SharedWorker entry should exist");
    let Some(index) = index else {
        ready.notify_all();
        return false;
    };

    let entry_is_creating = matches!(
        &workers[index].state,
        SharedWorkerRegistryState::Creating { .. }
    );
    debug_assert!(
        entry_is_creating,
        "claimed SharedWorker entry should still be creating"
    );
    if !entry_is_creating {
        ready.notify_all();
        return false;
    }

    workers[index].state = SharedWorkerRegistryState::Created(registration);
    ready.notify_all();
    true
}

fn remove_creating_shared_worker(key: &SharedWorkerKey) {
    let (workers, ready) = &*SHARED_WORKERS;
    let mut workers = workers.lock().expect("SharedWorker registry poisoned");
    let index = find_matching_shared_worker(&workers, key);
    debug_assert!(index.is_some(), "claimed SharedWorker entry should exist");
    let Some(index) = index else {
        return;
    };

    let waiters = match &workers[index].state {
        SharedWorkerRegistryState::Creating { waiters } => *waiters,
        state => {
            debug_assert!(
                matches!(state, SharedWorkerRegistryState::Creating { .. }),
                "claimed SharedWorker entry should still be creating"
            );
            return;
        },
    };

    if waiters == 0 {
        workers.remove(index);
    } else {
        workers[index].state = SharedWorkerRegistryState::Failed { waiters };
    };
    ready.notify_all();
}

fn send_connect_to_created_worker(
    registration: &SharedWorkerRegistration,
    inside_port: MessagePortImpl,
) -> bool {
    registration
        .sender
        .send(SharedWorkerScriptMsg::Connect(inside_port))
        .is_err()
}

impl SharedWorker {
    pub(crate) fn unregister_shared_worker(id: Uuid) {
        let (workers, ready) = &*SHARED_WORKERS;
        let mut workers = workers.lock().expect("SharedWorker registry poisoned");
        let old_len = workers.len();
        workers.retain(|entry| {
            !matches!(&entry.state, SharedWorkerRegistryState::Created(worker) if worker.id == id)
        });
        if workers.len() != old_len {
            ready.notify_all();
        }
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

        // Step 11. Enqueue the following steps to the shared worker manager:
        // Step 11.1. Let workerGlobalScope be null.
        let shared_worker_key = SharedWorkerKey {
            storage_key: outside_storage_key.clone(),
            // Include constructor origin in the key so `data:` SharedWorkers are not reused across origins.
            constructor_origin: constructor_origin.clone(),
            constructor_url: constructor_url.clone(),
            name: worker_name_string,
        };

        // Step 11.2. For each scope in the list of all `SharedWorkerGlobalScope` objects:
        // Step 11.2.1. Let workerStorageKey be the result of running obtain a storage key for non-storage purposes given scope's relevant settings object.
        // Step 11.2.2. If all of the following are true:
        // workerStorageKey equals outsideStorageKey;
        // scope's closing flag is false;
        // scope's constructor URL equals urlRecord; and
        // scope's name equals options["name"],
        // Servo also atomically records a Creating entry here when no matching
        // scope exists, so another same-key constructor cannot race into the
        // Step 11.6 fresh-worker path.
        let shared_worker = find_or_claim_shared_worker(shared_worker_key.clone());

        match shared_worker {
            SharedWorkerClaimResult::Created(registration) => {
                // Step 11.2.2.1. Set workerGlobalScope to scope.
                // Step 11.2.2.2. Break.
                // TODO Step 11.3. If workerGlobalScope is not null, but the user agent has been configured to disallow communication between the worker represented by the workerGlobalScope and the scripts whose settings object is outsideSettings, then set workerGlobalScope to null.
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
                if send_connect_to_created_worker(&registration, inside_port_impl) {
                    SharedWorker::queue_simple_error(global, worker_addr);
                }
                // TODO Step 11.5.8. Append the relevant owner to add given outsideSettings to workerGlobalScope's owner set.
                return Ok(worker);
            },
            SharedWorkerClaimResult::Failed => {
                SharedWorker::queue_simple_error(global, worker_addr);
                return Ok(worker);
            },
            SharedWorkerClaimResult::Claimed => {},
        }

        let initial_inside_port_impl =
            match SharedWorker::create_entangled_inside_port(cx, global, &outside_port) {
                Ok(inside_port_impl) => inside_port_impl,
                Err(error) => {
                    remove_creating_shared_worker(&shared_worker_key);
                    return Err(error);
                },
            };

        let parent_event_loop_sender = global
            .event_loop_sender()
            .expect("Window global must have an event loop sender");

        let (sender, receiver) = unbounded();
        let closing = Arc::new(AtomicBool::new(false));
        let registration_id = Uuid::new_v4();

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
        let _join_handle = match SharedWorkerGlobalScope::run_shared_worker_scope(
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
            constructor_origin,
            constructor_url,
            outside_storage_key,
            global.insecure_requests_policy(),
            global.policy_container(),
            global.font_context().cloned(),
        ) {
            Ok(join_handle) => join_handle,
            Err(error) => {
                error!("Failed to spawn SharedWorker thread: {error}");
                remove_creating_shared_worker(&shared_worker_key);
                SharedWorker::queue_simple_error(global, worker_addr);
                return Ok(worker);
            },
        };

        // Step 11.5.2. Let workerIsSecureContext be true if insideSettings is a secure context; otherwise, false.
        let Ok(worker_is_secure_context) = setup_receiver.recv() else {
            remove_creating_shared_worker(&shared_worker_key);
            SharedWorker::queue_simple_error(global, worker_addr);
            return Ok(worker);
        };

        let registration = SharedWorkerRegistration {
            id: registration_id,
            worker_type,
            credentials,
            extended_lifetime,
            worker_is_secure_context,
            closing,
            sender,
            _control_sender: control_sender,
        };

        if !transition_creating_to_created(&shared_worker_key, registration.clone()) {
            SharedWorker::queue_simple_error(global, worker_addr);
            return Ok(worker);
        }

        if registered_sender.send(()).is_err() {
            SharedWorker::unregister_shared_worker(registration_id);
            SharedWorker::queue_simple_error(global, worker_addr);
            return Ok(worker);
        }

        // Step 13. If is shared is true, then queue a global task on the DOM
        // manipulation task source given worker global scope to fire an event
        // named connect at worker global scope, using MessageEvent, with the data
        // attribute initialized to the empty string, the ports attribute
        // initialized to a new frozen array containing inside port, and the
        // source attribute initialized to inside port.
        if send_connect_to_created_worker(&registration, initial_inside_port_impl) {
            SharedWorker::unregister_shared_worker(registration_id);
            SharedWorker::queue_simple_error(global, worker_addr);
            return Ok(worker);
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
