/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::URLSearchParamsBinding::URLSearchParamsMethods;
use dom::bindings::codegen::Bindings::URLSearchParamsBinding::URLSearchParamsWrap;
use dom::bindings::codegen::UnionTypes::USVStringOrURLSearchParams;
use dom::bindings::error::Fallible;
use dom::bindings::iterable::Iterable;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::{DOMString, USVString};
use dom::bindings::weakref::MutableWeakRef;
use dom::globalscope::GlobalScope;
use dom::url::URL;
use dom_struct::dom_struct;
use encoding::types::EncodingRef;
use url::form_urlencoded;

// https://url.spec.whatwg.org/#interface-urlsearchparams
#[dom_struct]
pub struct URLSearchParams {
    reflector_: Reflector,
    // https://url.spec.whatwg.org/#concept-urlsearchparams-list
    list: DomRefCell<Vec<(String, String)>>,
    // https://url.spec.whatwg.org/#concept-urlsearchparams-url-object
    url: MutableWeakRef<URL>,
}

impl URLSearchParams {
    fn new_inherited(url: Option<&URL>) -> URLSearchParams {
        URLSearchParams {
            reflector_: Reflector::new(),
            list: DomRefCell::new(url.map_or(Vec::new(), |url| url.query_pairs())),
            url: MutableWeakRef::new(url),
        }
    }

    pub fn new(global: &GlobalScope, url: Option<&URL>) -> DomRoot<URLSearchParams> {
        reflect_dom_object(Box::new(URLSearchParams::new_inherited(url)), global,
                           URLSearchParamsWrap)
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-urlsearchparams
    pub fn Constructor(global: &GlobalScope, init: Option<USVStringOrURLSearchParams>) ->
                       Fallible<DomRoot<URLSearchParams>> {
        // Step 1.
        let query = URLSearchParams::new(global, None);
        match init {
            Some(USVStringOrURLSearchParams::USVString(init)) => {
                // Step 2.
                *query.list.borrow_mut() = form_urlencoded::parse(init.0.as_bytes())
                    .into_owned().collect();
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
        {
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
        }  // Un-borrow self.list
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
        form_urlencoded::Serializer::new(String::new())
            .encoding_override(encoding)
            .extend_pairs(&*list)
            .finish()
    }
}


impl URLSearchParams {
    // https://url.spec.whatwg.org/#concept-urlsearchparams-update
    fn update_steps(&self) {
        if let Some(url) = self.url.root() {
            url.set_query_pairs(&self.list.borrow())
        }
    }
}


impl Iterable for URLSearchParams {
    type Key = USVString;
    type Value = USVString;

    fn get_iterable_length(&self) -> u32 {
        self.list.borrow().len() as u32
    }

    fn get_value_at_index(&self, n: u32) -> USVString {
        let value = self.list.borrow()[n as usize].1.clone();
        USVString(value)
    }

    fn get_key_at_index(&self, n: u32) -> USVString {
        let key = self.list.borrow()[n as usize].0.clone();
        USVString(key)
    }
}
