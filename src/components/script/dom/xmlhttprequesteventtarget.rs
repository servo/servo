/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::XMLHttpRequestEventTargetDerived;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::eventtarget::{EventTarget, XMLHttpRequestTargetTypeId};
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

impl Reflectable for XMLHttpRequestEventTarget {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.eventtarget.mut_reflector()
    }
}

pub trait XMLHttpRequestEventTargetMethods {
}