/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::XMLHttpRequestEventTargetBinding::XMLHttpRequestEventTargetMethods;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::XMLHttpRequestEventTargetDerived;
use dom::eventtarget::{EventTarget, EventTargetTypeId};

#[derive(Copy, Clone, PartialEq)]
pub enum XMLHttpRequestEventTargetTypeId {
    XMLHttpRequest,
    XMLHttpRequestUpload,
}

#[dom_struct]
pub struct XMLHttpRequestEventTarget {
    eventtarget: EventTarget,
}

impl XMLHttpRequestEventTarget {
    pub fn new_inherited() -> XMLHttpRequestEventTarget {
        XMLHttpRequestEventTarget {
            eventtarget: EventTarget::new_inherited()
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

impl XMLHttpRequestEventTargetMethods for XMLHttpRequestEventTarget {
    // https://xhr.spec.whatwg.org/#handler-xhr-onloadstart
    event_handler!(loadstart, GetOnloadstart, SetOnloadstart);

    // https://xhr.spec.whatwg.org/#handler-xhr-onprogress
    event_handler!(progress, GetOnprogress, SetOnprogress);

    // https://xhr.spec.whatwg.org/#handler-xhr-onabort
    event_handler!(abort, GetOnabort, SetOnabort);

    // https://xhr.spec.whatwg.org/#handler-xhr-onerror
    event_handler!(error, GetOnerror, SetOnerror);

    // https://xhr.spec.whatwg.org/#handler-xhr-onload
    event_handler!(load, GetOnload, SetOnload);

    // https://xhr.spec.whatwg.org/#handler-xhr-ontimeout
    event_handler!(timeout, GetOntimeout, SetOntimeout);

    // https://xhr.spec.whatwg.org/#handler-xhr-onloadend
    event_handler!(loadend, GetOnloadend, SetOnloadend);
}
