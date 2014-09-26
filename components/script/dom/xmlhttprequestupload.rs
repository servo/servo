/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::XMLHttpRequestUploadDerived;
use dom::bindings::codegen::Bindings::XMLHttpRequestUploadBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Temporary;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::eventtarget::{EventTarget, XMLHttpRequestTargetTypeId};
use dom::xmlhttprequest::{XMLHttpRequestUploadTypeId};
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;

#[jstraceable]
#[must_root]
pub struct XMLHttpRequestUpload {
    eventtarget: XMLHttpRequestEventTarget
}

impl XMLHttpRequestUpload {
    fn new_inherited() -> XMLHttpRequestUpload {
        XMLHttpRequestUpload {
            eventtarget:XMLHttpRequestEventTarget::new_inherited(XMLHttpRequestUploadTypeId)
        }
    }
    pub fn new(global: &GlobalRef) -> Temporary<XMLHttpRequestUpload> {
        reflect_dom_object(box XMLHttpRequestUpload::new_inherited(),
                           global,
                           XMLHttpRequestUploadBinding::Wrap)
    }
}
impl Reflectable for XMLHttpRequestUpload {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}

impl XMLHttpRequestUploadDerived for EventTarget {
    fn is_xmlhttprequestupload(&self) -> bool {
        self.type_id == XMLHttpRequestTargetTypeId(XMLHttpRequestUploadTypeId)
    }
}
