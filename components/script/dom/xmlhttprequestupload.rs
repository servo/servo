/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::XMLHttpRequestUploadBinding;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct XMLHttpRequestUpload<TH: TypeHolderTrait> {
    eventtarget: XMLHttpRequestEventTarget<TH>
}

impl<TH: TypeHolderTrait> XMLHttpRequestUpload<TH> {
    fn new_inherited() -> XMLHttpRequestUpload<TH> {
        XMLHttpRequestUpload {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(),
        }
    }
    pub fn new(global: &GlobalScope<TH>) -> DomRoot<XMLHttpRequestUpload<TH>> {
        reflect_dom_object(Box::new(XMLHttpRequestUpload::new_inherited()),
                           global,
                           XMLHttpRequestUploadBinding::Wrap)
    }
}
