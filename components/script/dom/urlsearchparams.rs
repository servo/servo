/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::URLSearchParamsBinding;
use dom::bindings::codegen::Bindings::URLSearchParamsBinding::URLSearchParamsMethods;
use dom::bindings::codegen::UnionTypes::StringOrURLSearchParams;
use dom::bindings::codegen::UnionTypes::StringOrURLSearchParams::{eURLSearchParams, eString};
use dom::bindings::error::{Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Rootable, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};

use util::str::DOMString;

use encoding::all::UTF_8;
use encoding::types::{EncodingRef, EncoderTrap};

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

// https://url.spec.whatwg.org/#interface-urlsearchparams
#[dom_struct]
pub struct URLSearchParams {
    reflector_: Reflector,
    data: DOMRefCell<HashMap<DOMString, Vec<DOMString>>>,
}

impl URLSearchParams {
    fn new_inherited() -> URLSearchParams {
        URLSearchParams {
            reflector_: Reflector::new(),
            data: DOMRefCell::new(HashMap::new()),
        }
    }

    pub fn new(global: GlobalRef) -> Temporary<URLSearchParams> {
        reflect_dom_object(box URLSearchParams::new_inherited(), global,
                           URLSearchParamsBinding::Wrap)
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-urlsearchparams
    pub fn Constructor(global: GlobalRef, init: Option<StringOrURLSearchParams>) ->
                       Fallible<Temporary<URLSearchParams>> {
        let usp = URLSearchParams::new(global).root();
        match init {
            Some(eString(_s)) => {
                // XXXManishearth we need to parse the input here
                // https://url.spec.whatwg.org/#concept-urlencoded-parser
                // We can use rust-url's implementation here:
                // https://github.com/SimonSapin/rust-url/blob/master/form_urlencoded.rs#L29
            },
            Some(eURLSearchParams(u)) => {
                let u = u.root();
                let usp = usp.r();
                let mut map = usp.data.borrow_mut();
                // FIXME(https://github.com/rust-lang/rust/issues/23338)
                let r = u.r();
                let data = r.data.borrow();
                *map = data.clone();
            },
            None => {}
        }
        Ok(Temporary::from_rooted(usp.r()))
    }
}

impl<'a> URLSearchParamsMethods for JSRef<'a, URLSearchParams> {
    // https://url.spec.whatwg.org/#dom-urlsearchparams-append
    fn Append(self, name: DOMString, value: DOMString) {
        let mut data = self.data.borrow_mut();

        match data.entry(name) {
            Occupied(entry) => entry.into_mut().push(value),
            Vacant(entry) => {
                entry.insert(vec!(value));
            }
        }

        self.update_steps();
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-delete
    fn Delete(self, name: DOMString) {
        self.data.borrow_mut().remove(&name);
        self.update_steps();
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-get
    fn Get(self, name: DOMString) -> Option<DOMString> {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let data = self.data.borrow();
        data.get(&name).map(|v| v[0].clone())
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-has
    fn Has(self, name: DOMString) -> bool {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let data = self.data.borrow();
        data.contains_key(&name)
    }

    // https://url.spec.whatwg.org/#dom-urlsearchparams-set
    fn Set(self, name: DOMString, value: DOMString) {
        self.data.borrow_mut().insert(name, vec!(value));
        self.update_steps();
    }

    // https://url.spec.whatwg.org/#stringification-behavior
    fn Stringifier(self) -> DOMString {
        DOMString::from_utf8(self.serialize(None)).unwrap()
    }
}

pub trait URLSearchParamsHelpers {
    fn serialize(&self, encoding: Option<EncodingRef>) -> Vec<u8>;
    fn update_steps(&self);
}

impl URLSearchParamsHelpers for URLSearchParams {
    fn serialize(&self, encoding: Option<EncodingRef>) -> Vec<u8> {
        // https://url.spec.whatwg.org/#concept-urlencoded-serializer
        fn serialize_string(value: &str, encoding: EncodingRef) -> Vec<u8> {
            // https://url.spec.whatwg.org/#concept-urlencoded-byte-serializer

            // XXXManishearth should this be a strict encoding? Can unwrap()ing the result fail?
            let value = encoding.encode(value, EncoderTrap::Replace).unwrap();

            // Step 1.
            let mut buf = vec!();

            // Step 2.
            for i in &value {
                let append = match *i {
                    // Convert spaces:
                    // ' ' => '+'
                    0x20 => vec!(0x2B),

                    // Retain the following characters:
                    // '*', '-', '.', '0'...'9', 'A'...'Z', '_', 'a'...'z'
                    0x2A | 0x2D | 0x2E | 0x30...0x39 |
                        0x41...0x5A | 0x5F | 0x61...0x7A => vec!(*i),

                    // Encode everything else using 'percented-encoded bytes'
                    // https://url.spec.whatwg.org/#percent-encode
                    a => format!("%{:02X}", a).into_bytes(),
                };
                buf.push_all(&append);
            }

            // Step 3.
            buf
        }
        let encoding = encoding.unwrap_or(UTF_8 as EncodingRef);
        let mut buf = vec!();
        let mut first_pair = true;
        for (k, v) in self.data.borrow().iter() {
            let name = serialize_string(k, encoding);
            for val in v {
                let value = serialize_string(val, encoding);
                if first_pair {
                    first_pair = false;
                } else {
                    buf.push(0x26); // &
                }
                buf.push_all(&name);
                buf.push(0x3D); // =
                buf.push_all(&value)
            }
        }
        buf
    }

    fn update_steps(&self) {
        // XXXManishearth Implement this when the URL interface is implemented
        // https://url.spec.whatwg.org/#concept-uq-update
    }
}
