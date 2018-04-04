/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::dom::abstractworker::SimpleWorkerErrorHandler;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ServiceWorkerBinding::{
    ServiceWorkerMethods, ServiceWorkerState, Wrap,
};
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::structuredclone::StructuredCloneData;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::task::TaskOnce;
use dom_struct::dom_struct;
use js::jsapi::JSContext;
use js::jsval::UndefinedValue;
use js::rust::HandleValue;
use script_traits::{DOMMessage, ScriptMsg};
use servo_url::ServoUrl;
use std::cell::Cell;

pub type TrustedServiceWorkerAddress = Trusted<ServiceWorker>;

#[dom_struct]
pub struct ServiceWorker {
    eventtarget: EventTarget,
    script_url: DomRefCell<String>,
    scope_url: ServoUrl,
    state: Cell<ServiceWorkerState>,
    skip_waiting: Cell<bool>,
}

impl ServiceWorker {
    fn new_inherited(script_url: &str, skip_waiting: bool, scope_url: ServoUrl) -> ServiceWorker {
        ServiceWorker {
            eventtarget: EventTarget::new_inherited(),
            script_url: DomRefCell::new(String::from(script_url)),
            state: Cell::new(ServiceWorkerState::Installing),
            scope_url: scope_url,
            skip_waiting: Cell::new(skip_waiting),
        }
    }

    pub fn install_serviceworker(
        global: &GlobalScope,
        script_url: ServoUrl,
        scope_url: ServoUrl,
        skip_waiting: bool,
    ) -> DomRoot<ServiceWorker> {
        reflect_dom_object(
            Box::new(ServiceWorker::new_inherited(
                script_url.as_str(),
                skip_waiting,
                scope_url,
            )),
            global,
            Wrap,
        )
    }

    pub fn dispatch_simple_error(address: TrustedServiceWorkerAddress) {
        let service_worker = address.root();
        service_worker.upcast().fire_event(atom!("error"));
    }

    pub fn set_transition_state(&self, state: ServiceWorkerState) {
        self.state.set(state);
        self.upcast::<EventTarget>()
            .fire_event(atom!("statechange"));
    }

    pub fn get_script_url(&self) -> ServoUrl {
        ServoUrl::parse(&self.script_url.borrow().clone()).unwrap()
    }
}

impl ServiceWorkerMethods for ServiceWorker {
    // https://w3c.github.io/ServiceWorker/#service-worker-state-attribute
    fn State(&self) -> ServiceWorkerState {
        self.state.get()
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-url-attribute
    fn ScriptURL(&self) -> USVString {
        USVString(self.script_url.borrow().clone())
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/ServiceWorker/#service-worker-postmessage
    unsafe fn PostMessage(&self, cx: *mut JSContext, message: HandleValue) -> ErrorResult {
        // Step 1
        if let ServiceWorkerState::Redundant = self.state.get() {
            return Err(Error::InvalidState);
        }
        // Step 7
        rooted!(in(cx) let transfer = UndefinedValue());
        let data = StructuredCloneData::write(cx, message, transfer.handle())?;
        let msg_vec = DOMMessage {
            origin: self.global().origin().immutable().ascii_serialization(),
            data: data.move_to_arraybuffer(),
        };
        let _ = self
            .global()
            .script_to_constellation_chan()
            .send(ScriptMsg::ForwardDOMMessage(
                msg_vec,
                self.scope_url.clone()
            ));
        Ok(())
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-container-onerror-attribute
    event_handler!(error, GetOnerror, SetOnerror);

    // https://w3c.github.io/ServiceWorker/#ref-for-service-worker-onstatechange-attribute-1
    event_handler!(statechange, GetOnstatechange, SetOnstatechange);
}

impl TaskOnce for SimpleWorkerErrorHandler<ServiceWorker> {
    #[allow(unrooted_must_root)]
    fn run_once(self) {
        ServiceWorker::dispatch_simple_error(self.addr);
    }
}
