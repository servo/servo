/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::{Heap, JSObject};
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleValue};
use script_bindings::error::ErrorResult;
use script_bindings::reflector::{Reflector, reflect_dom_object};
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use servo_base::generic_channel::GenericSender;
use servo_base::id::ServiceWorkerId;
use servo_constellation_traits::ServiceWorkerMsg;
use servo_url::ServoUrl;

use crate::dom::bindings::codegen::Bindings::ClientBinding::{ClientMethods, FrameType};
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::StructuredSerializeOptions;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct Client {
    reflector_: Reflector,

    /// <https://w3c.github.io/ServiceWorker/#dfn-service-worker-client-service-worker-client>
    /// Note: client concept implemented via messaging with the service worker manager.
    #[no_trace]
    swmanager_sender: GenericSender<ServiceWorkerMsg>,

    #[no_trace]
    url: ServoUrl,

    /// <https://w3c.github.io/ServiceWorker/#dfn-service-worker-client-frame-type>
    frame_type: FrameType,

    #[no_trace]
    worker_id: ServiceWorkerId,
}

impl Client {
    fn new_inherited(
        swmanager_sender: GenericSender<ServiceWorkerMsg>,
        url: ServoUrl,
        frame_type: FrameType,
        worker_id: ServiceWorkerId,
    ) -> Client {
        Client {
            reflector_: Reflector::new(),
            swmanager_sender,
            url,
            frame_type,
            worker_id,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        swmanager_sender: GenericSender<ServiceWorkerMsg>,
        url: ServoUrl,
        frame_type: FrameType,
        worker_id: ServiceWorkerId,
        can_gc: CanGc,
    ) -> DomRoot<Client> {
        reflect_dom_object(
            Box::new(Client::new_inherited(
                swmanager_sender,
                url,
                frame_type,
                worker_id,
            )),
            global,
            can_gc,
        )
    }

    /// <https://w3c.github.io/ServiceWorker/#dom-client-postmessage-message-options>
    fn post_message_impl(
        &self,
        cx: &mut JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        // Step 1: Let contextObject be this.
        // Note: we're passing the worker id in the message to track contextObject.

        // Step 2: Let sourceSettings be the contextObject’s relevant settings object.
        let global = self.reflector_.global();

        // Step 4.5.1: Let origin be sourceSettings’s origin.
        // Note: done here and passing origin along in the message.
        let origin = global.origin();

        let data = structuredclone::write(cx.into(), message, Some(transfer))?;
        self.swmanager_sender
            .send(ServiceWorkerMsg::ForwardWorkerMessage {
                data,
                source: self.worker_id,
                url: self.url.clone(),
                origin: origin.immutable().clone(),
            })
            .map_err(|_| {
                Error::Type(c"Failed to send message to service worker manager".to_owned())
            })
    }
}

impl ClientMethods<crate::DomTypeHolder> for Client {
    /// <https://w3c.github.io/ServiceWorker/#dom-client-postmessage>
    fn PostMessage(
        &self,
        cx: &mut JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        self.post_message_impl(cx, message, transfer)
    }

    /// <https://w3c.github.io/ServiceWorker/#dom-client-postmessage-message-options>
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

    /// <https://w3c.github.io/ServiceWorker/#client-url-attribute>
    fn Url(&self) -> USVString {
        USVString(self.url.as_str().to_owned())
    }

    /// <https://w3c.github.io/ServiceWorker/#client-frametype>
    fn FrameType(&self) -> FrameType {
        self.frame_type
    }

    /// <https://w3c.github.io/ServiceWorker/#client-id>
    fn Id(&self) -> DOMString {
        format!("{}", self.worker_id).into()
    }
}
