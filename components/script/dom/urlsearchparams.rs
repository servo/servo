/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use url::form_urlencoded;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::URLSearchParamsBinding::URLSearchParamsMethods;
use crate::dom::bindings::codegen::UnionTypes::USVStringSequenceSequenceOrUSVStringUSVStringRecordOrUSVString;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::iterable::Iterable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::weakref::MutableWeakRef;
use crate::dom::globalscope::GlobalScope;
use crate::dom::url::URL;

/// <https://url.spec.whatwg.org/#interface-urlsearchparams>
#[dom_struct]
pub struct URLSearchParams {
    reflector_: Reflector,
    /// <https://url.spec.whatwg.org/#concept-urlsearchparams-list>
    list: DomRefCell<Vec<(String, String)>>,
    /// <https://url.spec.whatwg.org/#concept-urlsearchparams-url-object>
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
        Self::new_with_proto(global, None, url)
    }

    pub fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        url: Option<&URL>,
    ) -> DomRoot<URLSearchParams> {
        reflect_dom_object_with_proto(Box::new(URLSearchParams::new_inherited(url)), global, proto)
    }

    /// <https://url.spec.whatwg.org/#dom-urlsearchparams-urlsearchparams>
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        init: USVStringSequenceSequenceOrUSVStringUSVStringRecordOrUSVString,
    ) -> Fallible<DomRoot<URLSearchParams>> {
        // Step 1.
        let query = URLSearchParams::new_with_proto(global, proto, None);
        match init {
            USVStringSequenceSequenceOrUSVStringUSVStringRecordOrUSVString::USVStringSequenceSequence(init) => {
                // Step 2.

                // Step 2-1.
                if init.iter().any(|pair| pair.len() != 2) {
                    return Err(Error::Type("Sequence initializer must only contain pair elements.".to_string()));
                }

                // Step 2-2.
                *query.list.borrow_mut() =
                    init.iter().map(|pair| (pair[0].to_string(), pair[1].to_string())).collect::<Vec<_>>();
            },
            USVStringSequenceSequenceOrUSVStringUSVStringRecordOrUSVString::USVStringUSVStringRecord(init) => {
                // Step 3.
                *query.list.borrow_mut() =
                    (*init).iter().map(|(name, value)| (name.to_string(), value.to_string())).collect::<Vec<_>>();
            },
            USVStringSequenceSequenceOrUSVStringUSVStringRecordOrUSVString::USVString(init) => {
                // Step 4.
                let init_bytes = match init.0.chars().next() {
                    Some('?') => {
                        let (_, other_bytes) = init.0.as_bytes().split_at(1);

                        other_bytes
                    },
                    _ => init.0.as_bytes(),
                };

                *query.list.borrow_mut() =
                    form_urlencoded::parse(init_bytes).into_owned().collect();
            }
        }

        // Step 5.
        Ok(query)
    }

    pub fn set_list(&self, list: Vec<(String, String)>) {
        *self.list.borrow_mut() = list;
    }
}

impl URLSearchParamsMethods for URLSearchParams {
    /// <https://url.spec.whatwg.org/#dom-urlsearchparams-size>
    fn Size(&self) -> u32 {
        self.list.borrow().len() as u32
    }

    /// <https://url.spec.whatwg.org/#dom-urlsearchparams-append>
    fn Append(&self, name: USVString, value: USVString) {
        // Step 1.
        self.list.borrow_mut().push((name.0, value.0));
        // Step 2.
        self.update_steps();
    }

    /// <https://url.spec.whatwg.org/#dom-urlsearchparams-delete>
    fn Delete(&self, name: USVString, value: Option<USVString>) {
        // Step 1.
        self.list.borrow_mut().retain(|(k, v)| match &value {
            Some(value) => !(k == &name.0 && v == &value.0),
            None => k != &name.0,
        });
        // Step 2.
        self.update_steps();
    }

    /// <https://url.spec.whatwg.org/#dom-urlsearchparams-get>
    fn Get(&self, name: USVString) -> Option<USVString> {
        let list = self.list.borrow();
        list.iter()
            .find(|&kv| kv.0 == name.0)
            .map(|kv| USVString(kv.1.clone()))
    }

    /// <https://url.spec.whatwg.org/#dom-urlsearchparams-getall>
    fn GetAll(&self, name: USVString) -> Vec<USVString> {
        let list = self.list.borrow();
        list.iter()
            .filter_map(|(k, v)| {
                if k == &name.0 {
                    Some(USVString(v.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// <https://url.spec.whatwg.org/#dom-urlsearchparams-has>
    fn Has(&self, name: USVString, value: Option<USVString>) -> bool {
        let list = self.list.borrow();
        list.iter().any(|(k, v)| match &value {
            Some(value) => k == &name.0 && v == &value.0,
            None => k == &name.0,
        })
    }

    /// <https://url.spec.whatwg.org/#dom-urlsearchparams-set>
    fn Set(&self, name: USVString, value: USVString) {
        {
            // Step 1.
            let mut list = self.list.borrow_mut();
            let mut index = None;
            let mut i = 0;
            list.retain(|(k, _)| {
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
        } // Un-borrow self.list
          // Step 3.
        self.update_steps();
    }

    /// <https://url.spec.whatwg.org/#dom-urlsearchparams-sort>
    fn Sort(&self) {
        // Step 1.
        self.list
            .borrow_mut()
            .sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));

        // Step 2.
        self.update_steps();
    }

    /// <https://url.spec.whatwg.org/#stringification-behavior>
    fn Stringifier(&self) -> DOMString {
        DOMString::from(self.serialize_utf8())
    }
}

impl URLSearchParams {
    /// <https://url.spec.whatwg.org/#concept-urlencoded-serializer>
    pub fn serialize_utf8(&self) -> String {
        let list = self.list.borrow();
        form_urlencoded::Serializer::new(String::new())
            .extend_pairs(&*list)
            .finish()
    }

    /// <https://url.spec.whatwg.org/#concept-urlsearchparams-update>
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
