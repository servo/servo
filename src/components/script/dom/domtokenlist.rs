/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, TokenListAttrValue};
use dom::bindings::codegen::Bindings::DOMTokenListBinding;
use dom::bindings::global::Window;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalRootable};
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};
use dom::element::{Element, AttributeHandlers};
use dom::node::window_from_node;

use servo_util::namespace::Null;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct DOMTokenList {
    reflector_: Reflector,
    element: JS<Element>,
    local_name: &'static str,
}

impl DOMTokenList {
    pub fn new_inherited(element: &JSRef<Element>,
                         local_name: &'static str) -> DOMTokenList {
        DOMTokenList {
            reflector_: Reflector::new(),
            element: JS::from_rooted(element),
            local_name: local_name,
        }
    }

    pub fn new(element: &JSRef<Element>,
               local_name: &'static str) -> Temporary<DOMTokenList> {
        let window = window_from_node(element).root();
        reflect_dom_object(box DOMTokenList::new_inherited(element, local_name),
                           &Window(*window), DOMTokenListBinding::Wrap)
    }
}

impl Reflectable for DOMTokenList {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

trait PrivateDOMTokenListHelpers {
    fn attribute(&self) -> Option<Temporary<Attr>>;
}

impl<'a> PrivateDOMTokenListHelpers for JSRef<'a, DOMTokenList> {
    fn attribute(&self) -> Option<Temporary<Attr>> {
        let element = self.element.root();
        element.deref().get_attribute(Null, self.local_name)
    }
}

pub trait DOMTokenListMethods {
    fn Length(&self) -> u32;
    fn Item(&self, index: u32) -> Option<DOMString>;
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<DOMString>;
}

// http://dom.spec.whatwg.org/#domtokenlist
impl<'a> DOMTokenListMethods for JSRef<'a, DOMTokenList> {
    // http://dom.spec.whatwg.org/#dom-domtokenlist-length
    fn Length(&self) -> u32 {
        let attribute = self.attribute().root();
        match attribute {
            Some(attribute) => {
                match *attribute.deref().value() {
                    TokenListAttrValue(_, ref indexes) => indexes.len() as u32,
                    _ => fail!("Expected a TokenListAttrValue"),
                }
            }
            None => 0,
        }
    }

    // http://dom.spec.whatwg.org/#dom-domtokenlist-item
    fn Item(&self, index: u32) -> Option<DOMString> {
        let attribute = self.attribute().root();
        attribute.and_then(|attribute| {
            match *attribute.deref().value() {
                TokenListAttrValue(ref value, ref indexes) => {
                    indexes.as_slice().get(index as uint).map(|&(start, end)| {
                        value.as_slice().slice(start, end).to_string()
                    })
                },
                _ => fail!("Expected a TokenListAttrValue"),
            }
        })
    }

    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<DOMString> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}
