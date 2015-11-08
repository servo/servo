/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::URLSearchParamsBinding;
use dom::bindings::codegen::Bindings::URLSearchParamsBinding::URLSearchParamsMethods;
use dom::bindings::codegen::UnionTypes::USVStringOrURLSearchParams;
use dom::bindings::codegen::UnionTypes::USVStringOrURLSearchParams::{eUSVString, eURLSearchParams};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use encoding::types::EncodingRef;
use url::form_urlencoded::{parse, serialize_with_encoding};
use util::str::DOMString;

// https://url.spec.whatwg.org/#interface-urlsearchparams
#[dom_struct]
pub struct URLSearchParams {
    reflector_: Reflector,
    // https://url.spec.whatwg.org/#concept-urlsearchparams-list
    list: DOMRefCell<Vec<(String, String)>>,
}

impl URLSearchParams {
    fn new_inherited() -> URLSearchParams {
        URLSearchParams {
            reflector_: Reflector::new(),
            list: DOMRefCell::new(vec![]),
        }
    }

    pub fn new(global: GlobalRef) -> Root<URLSearchParams> {
        reflect_dom_object(box URLSearchParams::new_inherited(), global,
                           URLSearchParamsBinding::Wrap)
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-urlsearchparams
    pub fn Constructor(global: GlobalRef, init: Option<USVStringOrURLSearchParams>) ->
                       Fallible<Root<URLSearchParams>> {
        // Step 1.
        let query = URLSearchParams::new(global);
        match init {
            Some(eUSVString(init)) => {
                // Step 2.
                *query.list.borrow_mut() = parse(init.as_bytes());
            },
            Some(eURLSearchParams(init)) => {
                // Step 3.
                *query.list.borrow_mut() = init.list.borrow().clone();
            },
            None => {}
        }
        // Step 4.
        Ok(query)
    }
}

impl URLSearchParamsMethods for URLSearchParams {
    // https://url.spec.whatwg.org/#dom-urlsearchparams-append
    fn Append(&self, name: String, value: String) {
        // Step 1.
        self.list.borrow_mut().push((name, value));
        // Step 2.
        self.update_steps();
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-delete
    fn Delete(&self, name: String) {
        // Step 1.
        self.list.borrow_mut().retain(|&(ref k, _)| k != &name);
        // Step 2.
        self.update_steps();
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-get
    fn Get(&self, name: String) -> Option<String> {
        let list = self.list.borrow();
        list.iter().filter_map(|&(ref k, ref v)| {
            if k == &name {
                Some(v.clone())
            } else {
                None
            }
        }).next()
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-has
    fn Has(&self, name: String) -> bool {
        let list = self.list.borrow();
        list.iter().any(|&(ref k, _)| k == &name)
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-set
    fn Set(&self, name: String, value: String) {
        let mut list = self.list.borrow_mut();
        let mut index = None;
        let mut i = 0;
        list.retain(|&(ref k, _)| {
            if index.is_none() {
                if k == &name {
                    index = Some(i);
                } else {
                    i += 1;
                }
                true
            } else {
                k != &name
            }
        });
        match index {
            Some(index) => list[index].1 = value,
            None => list.push((name, value)),
        };
        self.update_steps();
    }

    // https://url.spec.whatwg.org/#stringification-behavior
    fn Stringifier(&self) -> DOMString {
        DOMString(self.serialize(None))
    }
}


impl URLSearchParams {
    // https://url.spec.whatwg.org/#concept-urlencoded-serializer
    pub fn serialize(&self, encoding: Option<EncodingRef>) -> String {
        let list = self.list.borrow();
        serialize_with_encoding(list.iter(), encoding)
    }
}


impl URLSearchParams {
    // https://url.spec.whatwg.org/#concept-uq-update
    fn update_steps(&self) {
        // XXXManishearth Implement this when the URL interface is implemented
    }
}
