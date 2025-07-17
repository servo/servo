/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;

use base::id::{MessagePortId, MessagePortIndex};
use constellation_traits::{MessagePortImpl, PortMessageTask};
use dom_struct::dom_struct;
use js::jsapi::{Heap, JS_NewObject, JSObject};
use js::jsval::UndefinedValue;
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleValue};
use script_bindings::conversions::SafeToJSValConvertible;

use crate::dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::{
    MessagePortMethods, StructuredSerializeOptions,
};
use crate::dom::bindings::conversions::root_from_object;
use crate::dom::bindings::error::{Error, ErrorResult, ErrorToJsval, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::{self, StructuredData};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::transferable::Transferable;
use crate::dom::bindings::utils::set_dictionary_property;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[dom_struct]
/// The MessagePort used in the DOM.
pub(crate) struct MessagePort {
    eventtarget: EventTarget,
    #[no_trace]
    message_port_id: MessagePortId,
    #[no_trace]
    entangled_port: RefCell<Option<MessagePortId>>,
    detached: Cell<bool>,
}

impl MessagePort {
    fn new_inherited(message_port_id: MessagePortId) -> MessagePort {
        MessagePort {
            eventtarget: EventTarget::new_inherited(),
            entangled_port: RefCell::new(None),
            detached: Cell::new(false),
            message_port_id,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#create-a-new-messageport-object>
    pub(crate) fn new(owner: &GlobalScope, can_gc: CanGc) -> DomRoot<MessagePort> {
        let port_id = MessagePortId::new();
        reflect_dom_object(Box::new(MessagePort::new_inherited(port_id)), owner, can_gc)
    }

    /// Create a new port for an incoming transfer-received one.
    pub(crate) fn new_transferred(
        owner: &GlobalScope,
        transferred_port: MessagePortId,
        entangled_port: Option<MessagePortId>,
        can_gc: CanGc,
    ) -> DomRoot<MessagePort> {
        reflect_dom_object(
            Box::new(MessagePort {
                message_port_id: transferred_port,
                eventtarget: EventTarget::new_inherited(),
                detached: Cell::new(false),
                entangled_port: RefCell::new(entangled_port),
            }),
            owner,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    pub(crate) fn entangle(&self, other_id: MessagePortId) {
        *self.entangled_port.borrow_mut() = Some(other_id);
    }

    /// <https://html.spec.whatwg.org/multipage/#disentangle>
    pub(crate) fn disentangle(&self) -> Option<MessagePortId> {
        // Disentangle initiatorPort and otherPort, so that they are no longer entangled or associated with each other.
        // Note: called from `disentangle_port` in the global, where the rest happens.
        self.entangled_port.borrow_mut().take()
    }

    /// Has the port been disentangled?
    /// Used when starting the port to fire the `close` event,
    /// to cover the case of a disentanglement while in transfer.
    pub(crate) fn disentangled(&self) -> bool {
        self.entangled_port.borrow().is_none()
    }

    pub(crate) fn message_port_id(&self) -> &MessagePortId {
        &self.message_port_id
    }

    pub(crate) fn detached(&self) -> bool {
        self.detached.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage>
    fn set_onmessage(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.set_event_handler_common("message", listener);
    }

    /// <https://html.spec.whatwg.org/multipage/#message-port-post-message-steps>
    #[allow(unsafe_code)]
    fn post_message_impl(
        &self,
        cx: SafeJSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        if self.detached.get() {
            return Ok(());
        }

        // Step 1 is the transfer argument.

        let target_port = self.entangled_port.borrow();

        // Step 3
        let mut doomed = false;

        let ports = transfer
            .iter()
            .filter_map(|&obj| unsafe { root_from_object::<MessagePort>(obj, *cx).ok() });
        for port in ports {
            // Step 2
            if port.message_port_id() == self.message_port_id() {
                return Err(Error::DataClone(None));
            }

            // Step 4
            if let Some(target_id) = target_port.as_ref() {
                if port.message_port_id() == target_id {
                    doomed = true;
                }
            }
        }

        // Step 5
        let data = structuredclone::write(cx, message, Some(transfer))?;

        if doomed {
            // TODO: The spec says to optionally report such a case to a dev console.
            return Ok(());
        }

        // Step 6, done in MessagePortImpl.

        let incumbent = match GlobalScope::incumbent() {
            None => unreachable!("postMessage called with no incumbent global"),
            Some(incumbent) => incumbent,
        };

        // Step 7
        let task = PortMessageTask {
            origin: incumbent.origin().immutable().clone(),
            data,
        };

        // Have the global proxy this call to the corresponding MessagePortImpl.
        self.global()
            .post_messageport_msg(*self.message_port_id(), task);
        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-crossrealmtransformsenderror>
    pub(crate) fn cross_realm_transform_send_error(&self, error: HandleValue, can_gc: CanGc) {
        // Perform PackAndPostMessage(port, "error", error),
        // discarding the result.
        let _ = self.pack_and_post_message("error", error, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-packandpostmessagehandlingerror>
    #[allow(unsafe_code)]
    pub(crate) fn pack_and_post_message_handling_error(
        &self,
        type_: &str,
        value: HandleValue,
        can_gc: CanGc,
    ) -> ErrorResult {
        // Let result be PackAndPostMessage(port, type, value).
        let result = self.pack_and_post_message(type_, value, can_gc);

        // If result is an abrupt completion,
        if let Err(error) = result.as_ref() {
            // Perform ! CrossRealmTransformSendError(port, result.[[Value]]).
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rooted_error = UndefinedValue());
            error
                .clone()
                .to_jsval(cx, &self.global(), rooted_error.handle_mut(), can_gc);
            self.cross_realm_transform_send_error(rooted_error.handle(), can_gc);
        }

        result
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-packandpostmessage>
    #[allow(unsafe_code)]
    pub(crate) fn pack_and_post_message(
        &self,
        type_: &str,
        value: HandleValue,
        _can_gc: CanGc,
    ) -> ErrorResult {
        let cx = GlobalScope::get_cx();

        // Let message be OrdinaryObjectCreate(null).
        rooted!(in(*cx) let mut message = unsafe { JS_NewObject(*cx, ptr::null()) });
        rooted!(in(*cx) let mut type_string = UndefinedValue());
        type_.safe_to_jsval(cx, type_string.handle_mut());

        // Perform ! CreateDataProperty(message, "type", type).
        unsafe {
            set_dictionary_property(*cx, message.handle(), "type", type_string.handle())
                .expect("Setting the message type should not fail.");
        }

        // Perform ! CreateDataProperty(message, "value", value).
        unsafe {
            set_dictionary_property(*cx, message.handle(), "value", value)
                .expect("Setting the message value should not fail.");
        }

        // Let targetPort be the port with which port is entangled, if any; otherwise let it be null.
        // Done in `global.post_messageport_msg`.

        // Let options be «[ "transfer" → « » ]».
        let mut rooted = CustomAutoRooter::new(vec![]);
        let transfer = CustomAutoRooterGuard::new(*cx, &mut rooted);

        // Run the message port post message steps providing targetPort, message, and options.
        rooted!(in(*cx) let mut message_val = UndefinedValue());
        message.safe_to_jsval(cx, message_val.handle_mut());
        self.post_message_impl(cx, message_val.handle(), transfer)
    }
}

impl Transferable for MessagePort {
    type Index = MessagePortIndex;
    type Data = MessagePortImpl;

    /// <https://html.spec.whatwg.org/multipage/#message-ports:transfer-steps>
    fn transfer(&self) -> Fallible<(MessagePortId, MessagePortImpl)> {
        // <https://html.spec.whatwg.org/multipage/#structuredserializewithtransfer>
        // Step 5.2. If transferable has a [[Detached]] internal slot and
        // transferable.[[Detached]] is true, then throw a "DataCloneError"
        // DOMException.
        if self.detached.get() {
            return Err(Error::DataClone(None));
        }

        self.detached.set(true);
        let id = self.message_port_id();

        // 1. Run local transfer logic, and return the object to be transferred.
        let transferred_port = self.global().mark_port_as_transferred(id);

        Ok((*id, transferred_port))
    }

    /// <https://html.spec.whatwg.org/multipage/#message-ports:transfer-receiving-steps>
    fn transfer_receive(
        owner: &GlobalScope,
        id: MessagePortId,
        port_impl: MessagePortImpl,
    ) -> Result<DomRoot<Self>, ()> {
        let transferred_port =
            MessagePort::new_transferred(owner, id, port_impl.entangled_port_id(), CanGc::note());
        owner.track_message_port(&transferred_port, Some(port_impl));
        Ok(transferred_port)
    }

    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<HashMap<MessagePortId, Self::Data>> {
        match data {
            StructuredData::Reader(r) => &mut r.port_impls,
            StructuredData::Writer(w) => &mut w.ports,
        }
    }
}

impl MessagePortMethods<crate::DomTypeHolder> for MessagePort {
    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    fn PostMessage(
        &self,
        cx: SafeJSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        if self.detached.get() {
            return Ok(());
        }
        self.post_message_impl(cx, message, transfer)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    fn PostMessage_(
        &self,
        cx: SafeJSContext,
        message: HandleValue,
        options: RootedTraceableBox<StructuredSerializeOptions>,
    ) -> ErrorResult {
        if self.detached.get() {
            return Ok(());
        }
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

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    fn Start(&self, can_gc: CanGc) {
        if self.detached.get() {
            return;
        }
        self.global()
            .start_message_port(self.message_port_id(), can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    fn Close(&self, can_gc: CanGc) {
        // Set this's [[Detached]] internal slot value to true.
        self.detached.set(true);

        let global = self.global();
        global.close_message_port(self.message_port_id());

        // If this is entangled, disentangle it.
        global.disentangle_port(self, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage>
    fn GetOnmessage(&self, can_gc: CanGc) -> Option<Rc<EventHandlerNonNull>> {
        if self.detached.get() {
            return None;
        }
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.get_event_handler_common("message", can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage>
    fn SetOnmessage(&self, listener: Option<Rc<EventHandlerNonNull>>, can_gc: CanGc) {
        if self.detached.get() {
            return;
        }
        self.set_onmessage(listener);
        // Note: we cannot use the event_handler macro, due to the need to start the port.
        self.global()
            .start_message_port(self.message_port_id(), can_gc);
    }

    // <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessageerror>
    event_handler!(messageerror, GetOnmessageerror, SetOnmessageerror);

    // <https://html.spec.whatwg.org/multipage/#handler-messageport-onclose>
    event_handler!(close, GetOnclose, SetOnclose);
}
