/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::Bindings::DOMTokenListBinding;
use dom::bindings::codegen::Bindings::DOMTokenListBinding::DOMTokenListMethods;
use dom::bindings::error::{Fallible, InvalidCharacter, Syntax};
use dom::bindings::global::Window;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalRootable};
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};
use dom::element::{Element, AttributeHandlers};
use dom::node::window_from_node;

use servo_util::atom::Atom;
use servo_util::namespace::Null;
use servo_util::str::{DOMString, HTML_SPACE_CHARACTERS};

#[deriving(Encodable)]
#[must_root]
pub struct DOMTokenList {
    reflector_: Reflector,
    element: JS<Element>,
    local_name: &'static str,
}

impl DOMTokenList {
    pub fn new_inherited(element: JSRef<Element>,
                         local_name: &'static str) -> DOMTokenList {
        DOMTokenList {
            reflector_: Reflector::new(),
            element: JS::from_rooted(element),
            local_name: local_name,
        }
    }

    pub fn new(element: JSRef<Element>,
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
    fn attribute(self) -> Option<Temporary<Attr>>;
    fn check_token_exceptions<'a>(self, token: &'a str) -> Fallible<&'a str>;
}

impl<'a> PrivateDOMTokenListHelpers for JSRef<'a, DOMTokenList> {
    fn attribute(self) -> Option<Temporary<Attr>> {
        let element = self.element.root();
        element.deref().get_attribute(Null, self.local_name)
    }

    fn check_token_exceptions<'a>(self, token: &'a str) -> Fallible<&'a str> {
        match token {
            "" => Err(Syntax),
            token if token.find(HTML_SPACE_CHARACTERS).is_some() => Err(InvalidCharacter),
            token => Ok(token)
        }
    }
}

// http://dom.spec.whatwg.org/#domtokenlist
impl<'a> DOMTokenListMethods for JSRef<'a, DOMTokenList> {
    // http://dom.spec.whatwg.org/#dom-domtokenlist-length
    fn Length(self) -> u32 {
        self.attribute().root().map(|attr| {
            attr.value().tokens().map(|tokens| tokens.len()).unwrap_or(0)
        }).unwrap_or(0) as u32
    }

    // http://dom.spec.whatwg.org/#dom-domtokenlist-item
    fn Item(self, index: u32) -> Option<DOMString> {
        self.attribute().root().and_then(|attr| attr.value().tokens().and_then(|mut tokens| {
            tokens.idx(index as uint).map(|token| token.as_slice().to_string())
        }))
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<DOMString> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }

    // http://dom.spec.whatwg.org/#dom-domtokenlist-contains
    fn Contains(self, token: DOMString) -> Fallible<bool> {
        self.check_token_exceptions(token.as_slice()).map(|slice| {
            self.attribute().root().and_then(|attr| attr.value().tokens().map(|mut tokens| {
                let atom = Atom::from_slice(slice);
                tokens.any(|token| *token == atom)
            })).unwrap_or(false)
        })
    }
}
