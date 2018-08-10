/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::XMLHttpRequestEventTargetBinding::XMLHttpRequestEventTargetMethods;
use dom::eventtarget::EventTarget;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct XMLHttpRequestEventTarget<TH: TypeHolderTrait> {
    eventtarget: EventTarget<TH>,
}

impl<TH: TypeHolderTrait> XMLHttpRequestEventTarget<TH> {
    pub fn new_inherited() -> XMLHttpRequestEventTarget<TH> {
        XMLHttpRequestEventTarget {
            eventtarget: EventTarget::new_inherited()
        }
    }
}

impl<TH: TypeHolderTrait> XMLHttpRequestEventTargetMethods<TH> for XMLHttpRequestEventTarget<TH> {
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
