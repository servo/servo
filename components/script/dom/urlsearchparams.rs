/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::URLSearchParamsBinding;
use dom::bindings::codegen::Bindings::URLSearchParamsBinding::URLSearchParamsMethods;
use dom::bindings::codegen::UnionTypes::StringOrURLSearchParams::{StringOrURLSearchParams, eURLSearchParams, eString};
use dom::bindings::error::{Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};

use servo_util::str::DOMString;

use encoding::all::UTF_8;
use encoding::types::{EncodingRef, EncodeReplace};

use std::cell::RefCell;
use std::collections::hashmap::HashMap;
use std::fmt::radix;
use std::ascii::OwnedStrAsciiExt;

#[jstraceable]
#[must_root]
pub struct URLSearchParams {
    data: Traceable<RefCell<HashMap<DOMString, Vec<DOMString>>>>,
    reflector_: Reflector,
}

impl URLSearchParams {
    fn new_inherited() -> URLSearchParams {
        URLSearchParams {
            data: Traceable::new(RefCell::new(HashMap::new())),
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalRef) -> Temporary<URLSearchParams> {
        reflect_dom_object(box URLSearchParams::new_inherited(), global, URLSearchParamsBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef, init: Option<StringOrURLSearchParams>) -> Fallible<Temporary<URLSearchParams>> {
        let usp = URLSearchParams::new(global).root();
        match init {
            Some(eString(_s)) => {
                // XXXManishearth we need to parse the input here
                // http://url.spec.whatwg.org/#concept-urlencoded-parser
                // We can use rust-url's implementation here:
                // https://github.com/SimonSapin/rust-url/blob/master/form_urlencoded.rs#L29
            },
            Some(eURLSearchParams(u)) => {
                let u = u.root();
                let mut map = usp.deref().data.deref().borrow_mut();
                *map = u.data.deref().borrow().clone();
            },
            None => {}
        }
        Ok(Temporary::from_rooted(*usp))
    }
}

impl<'a> URLSearchParamsMethods for JSRef<'a, URLSearchParams> {
    fn Append(self, name: DOMString, value: DOMString) {
        self.data.deref().borrow_mut().insert_or_update_with(name, vec!(value.clone()),
                                                             |_k, v| v.push(value.clone()));
        self.update_steps();
    }

    fn Delete(self, name: DOMString) {
        self.data.deref().borrow_mut().remove(&name);
        self.update_steps();
    }

    fn Get(self, name: DOMString) -> Option<DOMString> {
        self.data.deref().borrow().find_equiv(&name).map(|v| v[0].clone())
    }

    fn Has(self, name: DOMString) -> bool {
        self.data.deref().borrow().contains_key_equiv(&name)
    }

    fn Set(self, name: DOMString, value: DOMString) {
        self.data.deref().borrow_mut().insert(name, vec!(value));
        self.update_steps();
    }
}

impl Reflectable for URLSearchParams {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

pub trait URLSearchParamsHelpers {
    fn serialize(&self, encoding: Option<EncodingRef>) -> Vec<u8>;
    fn update_steps(&self);
}

impl URLSearchParamsHelpers for URLSearchParams {
    fn serialize(&self, encoding: Option<EncodingRef>) -> Vec<u8> {
        // http://url.spec.whatwg.org/#concept-urlencoded-serializer
        fn serialize_string(value: &DOMString, encoding: EncodingRef) -> Vec<u8> {
            // http://url.spec.whatwg.org/#concept-urlencoded-byte-serializer

            let value = value.as_slice();
            // XXXManishearth should this be a strict encoding? Can unwrap()ing the result fail?
            let value = encoding.encode(value, EncodeReplace).unwrap();
            let mut buf = vec!();
            for i in value.iter() {
                let append = match *i {
                    0x20 => vec!(0x2B),
                    0x2A | 0x2D | 0x2E |
                    0x30 .. 0x39 | 0x41 .. 0x5A |
                    0x5F | 0x61..0x7A => vec!(*i),
                    a => {
                        // http://url.spec.whatwg.org/#percent-encode
                        let mut encoded = vec!(0x25); // %
                        let s = format!("{}", radix(a, 16)).into_ascii_upper();
                        let bytes = s.as_bytes();
                        encoded.push_all(bytes);
                        encoded
                    }
                };
                buf.push_all(append.as_slice());
            }
            buf
        }
        let encoding = encoding.unwrap_or(UTF_8 as EncodingRef);
        let mut buf = vec!();
        let mut first_pair = true;
        for (k, v) in self.data.deref().borrow().iter() {
            let name = serialize_string(k, encoding);
            for val in v.iter() {
                let value = serialize_string(val, encoding);
                if first_pair {
                    first_pair = false;
                } else {
                    buf.push(0x26); // &
                }
                buf.push_all(name.as_slice());
                buf.push(0x3D); // =
                buf.push_all(value.as_slice())
            }
        }
        buf
    }

    fn update_steps(&self) {
        // XXXManishearth Implement this when the URL interface is implemented
        // http://url.spec.whatwg.org/#concept-uq-update
    }
}
