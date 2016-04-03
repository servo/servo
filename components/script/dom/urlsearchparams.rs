/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::URLSearchParamsBinding;
use dom::bindings::codegen::Bindings::URLSearchParamsBinding::URLSearchParamsMethods;
use dom::bindings::codegen::UnionTypes::USVStringOrURLSearchParams;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::USVString;
use dom::bindings::weakref::MutableWeakRef;
use dom::url::URL;
use encoding::types::EncodingRef;
use url::form_urlencoded::{parse, serialize_with_encoding};
use util::str::DOMString;

// https://url.spec.whatwg.org/#interface-urlsearchparams
#[dom_struct]
pub struct URLSearchParams {
    reflector_: Reflector,
    // https://url.spec.whatwg.org/#concept-urlsearchparams-list
    list: DOMRefCell<Vec<(String, String)>>,
    // https://url.spec.whatwg.org/#concept-urlsearchparams-url-object
    url: MutableWeakRef<URL>,
}

impl URLSearchParams {
    fn new_inherited(url: Option<&URL>) -> URLSearchParams {
        URLSearchParams {
            reflector_: Reflector::new(),
            list: DOMRefCell::new(vec![]),
            url: MutableWeakRef::new(url),
        }
    }

    pub fn new(global: GlobalRef, url: Option<&URL>) -> Root<URLSearchParams> {
        reflect_dom_object(box URLSearchParams::new_inherited(url), global,
                           URLSearchParamsBinding::Wrap)
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-urlsearchparams
    pub fn Constructor(global: GlobalRef, init: Option<USVStringOrURLSearchParams>) ->
                       Fallible<Root<URLSearchParams>> {
        // Step 1.
        let query = URLSearchParams::new(global, None);
        match init {
            Some(USVStringOrURLSearchParams::USVString(init)) => {
                // Step 2.
                *query.list.borrow_mut() = parse(init.0.as_bytes());
            },
            Some(USVStringOrURLSearchParams::URLSearchParams(init)) => {
                // Step 3.
                *query.list.borrow_mut() = init.list.borrow().clone();
            },
            None => {}
        }
        // Step 4.
        Ok(query)
    }

    pub fn set_list(&self, list: Vec<(String, String)>) {
        *self.list.borrow_mut() = list;
    }
}

impl URLSearchParamsMethods for URLSearchParams {
    // https://url.spec.whatwg.org/#dom-urlsearchparams-append
    fn Append(&self, name: USVString, value: USVString) {
        // Step 1.
        self.list.borrow_mut().push((name.0, value.0));
        // Step 2.
        self.update_steps();
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-delete
    fn Delete(&self, name: USVString) {
        // Step 1.
        self.list.borrow_mut().retain(|&(ref k, _)| k != &name.0);
        // Step 2.
        self.update_steps();
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-get
    fn Get(&self, name: USVString) -> Option<USVString> {
        let list = self.list.borrow();
        list.iter().find(|&kv| kv.0 == name.0)
            .map(|ref kv| USVString(kv.1.clone()))
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-getall
    fn GetAll(&self, name: USVString) -> Vec<USVString> {
        let list = self.list.borrow();
        list.iter().filter_map(|&(ref k, ref v)| {
            if k == &name.0 {
                Some(USVString(v.clone()))
            } else {
                None
            }
        }).collect()
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-has
    fn Has(&self, name: USVString) -> bool {
        let list = self.list.borrow();
        list.iter().any(|&(ref k, _)| k == &name.0)
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-set
    fn Set(&self, name: USVString, value: USVString) {
        // Step 1.
        let mut list = self.list.borrow_mut();
        let mut index = None;
        let mut i = 0;
        list.retain(|&(ref k, _)| {
            if index.is_none() {
                if k == &name.0 {
                    index = Some(i);
                } else {
                    i += 1;
                }
                true
            } else {
                k != &name.0
            }
        });
        match index {
            Some(index) => list[index].1 = value.0,
            None => list.push((name.0, value.0)), // Step 2.
        };
        // Step 3.
        self.update_steps();
    }

    // https://url.spec.whatwg.org/#stringification-behavior
    fn Stringifier(&self) -> DOMString {
        DOMString::from(self.serialize(None))
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
    // https://url.spec.whatwg.org/#concept-urlsearchparams-update
    fn update_steps(&self) {
        if let Some(url) = self.url.root() {
            url.set_query(self.serialize(None));
        }
    }
}
