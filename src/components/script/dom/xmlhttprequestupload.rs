/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::XMLHttpRequestUploadDerived;
use dom::bindings::codegen::Bindings::XMLHttpRequestUploadBinding;
use dom::bindings::js::{Temporary, JSRef};
use dom::bindings::utils::reflect_dom_object;
use dom::eventtarget::{EventTarget, XMLHttpRequestTargetTypeId};
use dom::window::Window;
use dom::xmlhttprequest::{XMLHttpRequestUploadTypeId};
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;

#[deriving(Encodable)]
pub struct XMLHttpRequestUpload {
    pub xmlhttprequesteventtarget: XMLHttpRequestEventTarget
}

impl XMLHttpRequestUpload {
    pub fn new_inherited() -> XMLHttpRequestUpload {
        XMLHttpRequestUpload {
            xmlhttprequesteventtarget:XMLHttpRequestEventTarget::new_inherited(XMLHttpRequestUploadTypeId)
        }
    }
    pub fn new(window: &JSRef<Window>) -> Temporary<XMLHttpRequestUpload> {
        reflect_dom_object(box XMLHttpRequestUpload::new_inherited(),
                           window,
                           XMLHttpRequestUploadBinding::Wrap)
    }
}

impl XMLHttpRequestUploadDerived for EventTarget {
    fn is_xmlhttprequestupload(&self) -> bool {
        self.type_id == XMLHttpRequestTargetTypeId(XMLHttpRequestUploadTypeId)
    }
}

pub trait XMLHttpRequestUploadMethods {
}
