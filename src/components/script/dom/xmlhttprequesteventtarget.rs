/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::XMLHttpRequestEventTargetDerived;
use dom::bindings::js::JSRef;
use dom::eventtarget::{EventTarget, EventTargetHelpers, XMLHttpRequestTargetTypeId};
use dom::xmlhttprequest::XMLHttpRequestId;

#[deriving(Encodable)]
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

pub trait XMLHttpRequestEventTargetMethods {
    fn GetOnloadstart(&self) -> Option<EventHandlerNonNull>;
    fn SetOnloadstart(&mut self, listener: Option<EventHandlerNonNull>);
    fn GetOnprogress(&self) -> Option<EventHandlerNonNull>;
    fn SetOnprogress(&mut self, listener: Option<EventHandlerNonNull>);
    fn GetOnabort(&self) -> Option<EventHandlerNonNull>;
    fn SetOnabort(&mut self, listener: Option<EventHandlerNonNull>);
    fn GetOnerror(&self) -> Option<EventHandlerNonNull>;
    fn SetOnerror(&mut self, listener: Option<EventHandlerNonNull>);
    fn GetOnload(&self) -> Option<EventHandlerNonNull>;
    fn SetOnload(&mut self, listener: Option<EventHandlerNonNull>);
    fn GetOntimeout(&self) -> Option<EventHandlerNonNull>;
    fn SetOntimeout(&mut self, listener: Option<EventHandlerNonNull>);
    fn GetOnloadend(&self) -> Option<EventHandlerNonNull>;
    fn SetOnloadend(&mut self, listener: Option<EventHandlerNonNull>);
}

impl<'a> XMLHttpRequestEventTargetMethods for JSRef<'a, XMLHttpRequestEventTarget> {
    fn GetOnloadstart(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("loadstart")
    }

    fn SetOnloadstart(&mut self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("loadstart", listener)
    }

    fn GetOnprogress(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("progress")
    }

    fn SetOnprogress(&mut self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("progress", listener)
    }

    fn GetOnabort(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("abort")
    }

    fn SetOnabort(&mut self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("abort", listener)
    }

    fn GetOnerror(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("error")
    }

    fn SetOnerror(&mut self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("error", listener)
    }

    fn GetOnload(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("load")
    }

    fn SetOnload(&mut self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("load", listener)
    }

    fn GetOntimeout(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("timeout")
    }

    fn SetOntimeout(&mut self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("timeout", listener)
    }

    fn GetOnloadend(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("loadend")
    }

    fn SetOnloadend(&mut self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        eventtarget.set_event_handler_common("loadend", listener)
    }
}
