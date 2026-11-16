/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, allow(crown::jscontext_first_arg))]

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use net_traits::request::Referrer;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_base::generic_channel::GenericCallback;
use servo_base::id::WorkerId;
use servo_constellation_traits::{
    MessagePortImpl, ScriptToConstellationMessage, SharedWorkerClientMessage, SharedWorkerConnect,
    SharedWorkerCreationData, SharedWorkerOptions as WorkerOptions, SharedWorkerType,
};
use uuid::Uuid;

use crate::conversions::Convert;
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
use crate::dom::bindings::transferable::Transferable;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::dom::trustedtypes::trustedscripturl::TrustedScriptURL;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::prepare_workerscope_init;
use crate::realms::enter_auto_realm;
use crate::url::ensure_blob_referenced_by_url_is_kept_alive;

/// <https://html.spec.whatwg.org/multipage/#shared-workers-and-the-sharedworker-interface>
#[dom_struct]
pub(crate) struct SharedWorker {
    eventtarget: EventTarget,
    port: Dom<MessagePort>,
}

impl SharedWorker {
    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        port: &MessagePort,
        cx: &mut JSContext,
    ) -> DomRoot<SharedWorker> {
        reflect_dom_object_with_proto(
            cx,
            Box::new(Self {
                eventtarget: EventTarget::new_inherited(),
                port: Dom::from_ref(port),
            }),
            global,
            proto,
        )
    }

    fn queue_simple_error(&self) {
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue_simple_event(self.upcast(), atom!("error"));
    }

    fn create_entangled_inside_port(
        cx: &mut JSContext,
        global: &GlobalScope,
        outside_port: &MessagePort,
    ) -> Fallible<MessagePortImpl> {
        // Step 11.5.5. Let insidePort be a new MessagePort in insideSettings's realm.
        let inside_port = MessagePort::new(cx, global);
        global.track_message_port(&inside_port, None);
        // Step 11.5.6. Entangle outsidePort and insidePort.
        global.entangle_ports(
            *outside_port.message_port_id(),
            *inside_port.message_port_id(),
        );
        let (_, inside_port_impl) = inside_port.transfer(cx)?;
        Ok(inside_port_impl)
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
        let options = match options {
            StringOrSharedWorkerOptions::String(name) => {
                let mut options = SharedWorkerOptions::empty();
                options.parent.name = name;
                options
            },
            StringOrSharedWorkerOptions::SharedWorkerOptions(options) => options,
        };
        // Step 3. Let outsideSettings be this's relevant settings object.
        // Step 4. Let urlRecord be the result of encoding-parsing a URL given
        // compliantScriptURL, relative to outsideSettings.
        // Step 5. If urlRecord is failure, then throw a "SyntaxError" DOMException.
        let Ok(url) = global
            .encoding_parse_a_url(&compliant_script_url.str())
            .map(|url| ensure_blob_referenced_by_url_is_kept_alive(global, url))
        else {
            return Err(Error::Syntax(None));
        };

        // Step 6. Let outsidePort be a new MessagePort in outsideSettings's realm.
        let outside_port = MessagePort::new(cx, global);
        global.track_message_port(&outside_port, None);
        // Step 7. Set this's port to outsidePort.
        // Step 10. Let worker be this.
        let worker = Self::new(global, proto, &outside_port, cx);
        let port = Self::create_entangled_inside_port(cx, global, &outside_port)?;
        let task_source = global
            .task_manager()
            .dom_manipulation_task_source()
            .to_sendable();
        let callback_address = Trusted::new(&*worker);
        let reply = match GenericCallback::new(move |message| {
            let address = callback_address.clone();
            task_source.queue(task!(sharedworker_manager_reply: move |cx| {
                let worker = address.root();
                let global = worker.global();
                match message {
                    Ok(SharedWorkerClientMessage::Connected { enable_port }) => {
                        if enable_port {
                            // Step 11 of onComplete of <https://html.spec.whatwg.org/multipage/#run-a-worker>
                            // Enable outside port's port message queue.
                            global.start_message_port(cx, worker.port.message_port_id());
                        }
                    },
                    Ok(SharedWorkerClientMessage::Error) => {
                        worker.upcast().fire_event(cx, atom!("error"));
                    },
                    Ok(SharedWorkerClientMessage::CspViolations(violations)) => {
                        let mut realm = enter_auto_realm(cx, &*global);
                        global.run_worker_csp_violation_report_tasks(
                            violations,
                            &mut realm.current_realm(),
                        );
                    },
                    Err(error) => {
                        warn!("Failed to receive SharedWorker manager reply: {error}");
                        worker.upcast().fire_event(cx, atom!("error"));
                    },
                }
            }));
        }) {
            Ok(reply) => reply,
            Err(error) => {
                error!("Failed to create SharedWorker reply channel: {error}");
                global.disentangle_port(cx, &outside_port);
                worker.queue_simple_error();
                return Ok(worker);
            },
        };

        let worker_id = WorkerId(Uuid::new_v4());
        // Step 8. Let callerIsSecureContext be true if outsideSettings is a secure
        // context; otherwise, false.
        let init = prepare_workerscope_init(
            global,
            #[cfg(feature = "devtools")]
            None,
            Some(worker_id),
            #[cfg(feature = "webgl")]
            window.webgl_chan(),
        );
        // Step 9. Let outsideStorageKey be the result of running obtain a storage
        // key for non-storage purposes given outsideSettings.
        // The snapshot carries its origin; the manager resolves the top-level site.
        let referrer = match global.get_referrer() {
            Referrer::Client(url) => Referrer::ReferrerUrl(url),
            referrer => referrer,
        };
        let request = SharedWorkerConnect {
            connection_id: *outside_port.message_port_id(),
            port,
            reply,
            creation: SharedWorkerCreationData {
                init,
                webview_id: window.webview_id(),
                browsing_context_id: window.window_proxy().browsing_context_id(),
                options: WorkerOptions {
                    name: options.parent.name.to_string(),
                    worker_type: match options.parent.type_ {
                        WorkerType::Classic => SharedWorkerType::Classic,
                        WorkerType::Module => SharedWorkerType::Module,
                    },
                    credentials: options.parent.credentials.convert(),
                    extended_lifetime: options.extendedLifetime,
                },
                url: url.into_transfer(),
                referrer,
                policy_container: global.policy_container(),
                insecure_requests_policy: global.insecure_requests_policy(),
                has_trustworthy_ancestor_origin: global.has_trustworthy_ancestor_origin(),
                is_nested_browsing_context: global.is_nested_browsing_context(),
            },
        };
        let blob_transfer = request.creation.url.transfer_guard();
        // Step 11. Enqueue the following steps to the shared worker manager:
        match global.script_to_constellation_chan().send(
            ScriptToConstellationMessage::ConnectSharedWorker(Box::new(request)),
        ) {
            Ok(()) => blob_transfer.commit(),
            Err(error) => {
                error!("Failed to request SharedWorker connection: {error}");
                global.disentangle_port(cx, &outside_port);
                worker.queue_simple_error();
            },
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
