/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use base::id::ServiceWorkerId;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleValue};
use script_traits::{DOMMessage, ScriptMsg};
use servo_url::ServoUrl;

use crate::dom::abstractworker::SimpleWorkerErrorHandler;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::StructuredSerializeOptions;
use crate::dom::bindings::codegen::Bindings::ServiceWorkerBinding::{
    ServiceWorkerMethods, ServiceWorkerState,
};
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext};
use crate::task::TaskOnce;

pub(crate) type TrustedServiceWorkerAddress = Trusted<ServiceWorker>;

#[dom_struct]
pub(crate) struct ServiceWorker {
    eventtarget: EventTarget,
    script_url: DomRefCell<String>,
    #[no_trace]
    scope_url: ServoUrl,
    state: Cell<ServiceWorkerState>,
    #[no_trace]
    worker_id: ServiceWorkerId,
}

impl ServiceWorker {
    fn new_inherited(
        script_url: &str,
        scope_url: ServoUrl,
        worker_id: ServiceWorkerId,
    ) -> ServiceWorker {
        ServiceWorker {
            eventtarget: EventTarget::new_inherited(),
            script_url: DomRefCell::new(String::from(script_url)),
            state: Cell::new(ServiceWorkerState::Installing),
            scope_url,
            worker_id,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        script_url: ServoUrl,
        scope_url: ServoUrl,
        worker_id: ServiceWorkerId,
        can_gc: CanGc,
    ) -> DomRoot<ServiceWorker> {
        reflect_dom_object(
            Box::new(ServiceWorker::new_inherited(
                script_url.as_str(),
                scope_url,
                worker_id,
            )),
            global,
            can_gc,
        )
    }

    pub(crate) fn dispatch_simple_error(address: TrustedServiceWorkerAddress, can_gc: CanGc) {
        let service_worker = address.root();
        service_worker.upcast().fire_event(atom!("error"), can_gc);
    }

    pub(crate) fn set_transition_state(&self, state: ServiceWorkerState, can_gc: CanGc) {
        self.state.set(state);
        self.upcast::<EventTarget>()
            .fire_event(atom!("statechange"), can_gc);
    }

    pub(crate) fn get_script_url(&self) -> ServoUrl {
        ServoUrl::parse(&self.script_url.borrow().clone()).unwrap()
    }

    /// <https://w3c.github.io/ServiceWorker/#service-worker-postmessage>
    fn post_message_impl(
        &self,
        cx: JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        // Step 1
        if let ServiceWorkerState::Redundant = self.state.get() {
            return Err(Error::InvalidState);
        }
        // Step 7
        let data = structuredclone::write(cx, message, Some(transfer))?;
        let incumbent = GlobalScope::incumbent().expect("no incumbent global?");
        let msg_vec = DOMMessage {
            origin: incumbent.origin().immutable().clone(),
            data,
        };
        let _ = self
            .global()
            .script_to_constellation_chan()
            .send(ScriptMsg::ForwardDOMMessage(
                msg_vec,
                self.scope_url.clone(),
            ));
        Ok(())
    }
}

impl ServiceWorkerMethods<crate::DomTypeHolder> for ServiceWorker {
    // https://w3c.github.io/ServiceWorker/#service-worker-state-attribute
    fn State(&self) -> ServiceWorkerState {
        self.state.get()
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-url-attribute
    fn ScriptURL(&self) -> USVString {
        USVString(self.script_url.borrow().clone())
    }

    /// <https://w3c.github.io/ServiceWorker/#service-worker-postmessage>
    fn PostMessage(
        &self,
        cx: JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        self.post_message_impl(cx, message, transfer)
    }

    /// <https://w3c.github.io/ServiceWorker/#service-worker-postmessage>
    fn PostMessage_(
        &self,
        cx: JSContext,
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
        let guard = CustomAutoRooterGuard::new(*cx, &mut rooted);
        self.post_message_impl(cx, message, guard)
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-container-onerror-attribute
    event_handler!(error, GetOnerror, SetOnerror);

    // https://w3c.github.io/ServiceWorker/#ref-for-service-worker-onstatechange-attribute-1
    event_handler!(statechange, GetOnstatechange, SetOnstatechange);
}

impl TaskOnce for SimpleWorkerErrorHandler<ServiceWorker> {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn run_once(self) {
        ServiceWorker::dispatch_simple_error(self.addr, CanGc::note());
    }
}
