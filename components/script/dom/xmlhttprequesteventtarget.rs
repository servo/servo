/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::XMLHttpRequestEventTargetBinding::XMLHttpRequestEventTargetMethods;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::XMLHttpRequestEventTargetDerived;
use dom::bindings::js::JSRef;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::eventtarget::{EventTarget, EventTargetHelpers, XMLHttpRequestTargetTypeId};
use dom::xmlhttprequest::XMLHttpRequestId;

#[deriving(Encodable)]
#[must_root]
pub struct XMLHttpRequestEventTarget {
    pub eventtarget: EventTarget,
}

impl XMLHttpRequestEventTarget {
    pub fn new_inherited(type_id: XMLHttpRequestId) -> XMLHttpRequestEventTarget {
        XMLHttpRequestEventTarget {
            eventtarget: EventTarget::new_inherited(XMLHttpRequestTargetTypeId(type_id))
        }
    }
}
impl XMLHttpRequestEventTargetDerived for EventTarget {
    fn is_xmlhttprequesteventtarget(&self) -> bool {
        match self.type_id {
            XMLHttpRequestTargetTypeId(_) => true,
            _ => false
        }
    }

}

impl Reflectable for XMLHttpRequestEventTarget {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}

impl<'a> XMLHttpRequestEventTargetMethods for JSRef<'a, XMLHttpRequestEventTarget> {
    fn GetOnloadstart(self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("loadstart")
    }

    fn SetOnloadstart(self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("loadstart", listener)
    }

    fn GetOnprogress(self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("progress")
    }

    fn SetOnprogress(self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("progress", listener)
    }

    fn GetOnabort(self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("abort")
    }

    fn SetOnabort(self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("abort", listener)
    }

    fn GetOnerror(self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("error")
    }

    fn SetOnerror(self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("error", listener)
    }

    fn GetOnload(self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("load")
    }

    fn SetOnload(self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("load", listener)
    }

    fn GetOntimeout(self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("timeout")
    }

    fn SetOntimeout(self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("timeout", listener)
    }

    fn GetOnloadend(self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("loadend")
    }

    fn SetOnloadend(self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("loadend", listener)
    }
}
