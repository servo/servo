/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::XMLHttpRequestEventTargetBinding::XMLHttpRequestEventTargetMethods;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::XMLHttpRequestEventTargetDerived;
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};

#[derive(JSTraceable, Copy, Clone, PartialEq, HeapSizeOf)]
pub enum XMLHttpRequestEventTargetTypeId {
    XMLHttpRequest,
    XMLHttpRequestUpload,
}

#[dom_struct]
pub struct XMLHttpRequestEventTarget {
    eventtarget: EventTarget,
}

impl XMLHttpRequestEventTarget {
    pub fn new_inherited(type_id: XMLHttpRequestEventTargetTypeId) -> XMLHttpRequestEventTarget {
        XMLHttpRequestEventTarget {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::XMLHttpRequestEventTarget(type_id))
        }
    }
}

impl XMLHttpRequestEventTargetDerived for EventTarget {
    fn is_xmlhttprequesteventtarget(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::XMLHttpRequestEventTarget(_) => true,
            _ => false
        }
    }

}

impl<'a> XMLHttpRequestEventTargetMethods for &'a XMLHttpRequestEventTarget {
    event_handler!(loadstart, GetOnloadstart, SetOnloadstart);
    event_handler!(progress, GetOnprogress, SetOnprogress);
    event_handler!(abort, GetOnabort, SetOnabort);
    event_handler!(error, GetOnerror, SetOnerror);
    event_handler!(load, GetOnload, SetOnload);
    event_handler!(timeout, GetOntimeout, SetOntimeout);
    event_handler!(loadend, GetOnloadend, SetOnloadend);
}
