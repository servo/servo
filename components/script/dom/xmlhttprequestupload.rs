/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::XMLHttpRequestUploadBinding;
use dom::bindings::codegen::InheritTypes::XMLHttpRequestUploadDerived;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::reflect_dom_object;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTargetTypeId;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct XMLHttpRequestUpload {
    eventtarget: XMLHttpRequestEventTarget
}

impl XMLHttpRequestUpload {
    fn new_inherited() -> XMLHttpRequestUpload {
        XMLHttpRequestUpload {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(
                XMLHttpRequestEventTargetTypeId::XMLHttpRequestUpload)
        }
    }
    pub fn new(global: GlobalRef) -> Root<XMLHttpRequestUpload> {
        reflect_dom_object(box XMLHttpRequestUpload::new_inherited(),
                           global,
                           XMLHttpRequestUploadBinding::Wrap)
    }
}

impl XMLHttpRequestUploadDerived for EventTarget {
    fn is_xmlhttprequestupload(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::XMLHttpRequestEventTarget(XMLHttpRequestEventTargetTypeId::XMLHttpRequestUpload)
    }
}
