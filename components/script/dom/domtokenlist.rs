/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::Bindings::DOMTokenListBinding;
use dom::bindings::codegen::Bindings::DOMTokenListBinding::DOMTokenListMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::error::Error::{InvalidCharacter, Syntax};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::element::{Element, AttributeHandlers};
use dom::node::window_from_node;

use util::str::{DOMString, HTML_SPACE_CHARACTERS};
use string_cache::Atom;

use std::borrow::ToOwned;

#[dom_struct]
pub struct DOMTokenList {
    reflector_: Reflector,
    element: JS<Element>,
    local_name: Atom,
}

impl DOMTokenList {
    pub fn new_inherited(element: &Element, local_name: Atom) -> DOMTokenList {
        DOMTokenList {
            reflector_: Reflector::new(),
            element: JS::from_ref(element),
            local_name: local_name,
        }
    }

    pub fn new(element: &Element, local_name: &Atom) -> Root<DOMTokenList> {
        let window = window_from_node(element);
        reflect_dom_object(box DOMTokenList::new_inherited(element, local_name.clone()),
                           GlobalRef::Window(window.r()),
                           DOMTokenListBinding::Wrap)
    }
}

trait PrivateDOMTokenListHelpers {
    fn attribute(self) -> Option<Root<Attr>>;
    fn check_token_exceptions(self, token: &str) -> Fallible<Atom>;
}

impl<'a> PrivateDOMTokenListHelpers for &'a DOMTokenList {
    fn attribute(self) -> Option<Root<Attr>> {
        let element = self.element.root();
        element.r().get_attribute(&ns!(""), &self.local_name)
    }

    fn check_token_exceptions(self, token: &str) -> Fallible<Atom> {
        match token {
            "" => Err(Syntax),
            slice if slice.find(HTML_SPACE_CHARACTERS).is_some() => Err(InvalidCharacter),
            slice => Ok(Atom::from_slice(slice))
        }
    }
}

// https://dom.spec.whatwg.org/#domtokenlist
impl<'a> DOMTokenListMethods for &'a DOMTokenList {
    // https://dom.spec.whatwg.org/#dom-domtokenlist-length
    fn Length(self) -> u32 {
        self.attribute().map(|attr| {
            let attr = attr.r();
            attr.value().tokens().map(|tokens| tokens.len()).unwrap_or(0)
        }).unwrap_or(0) as u32
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-item
    fn Item(self, index: u32) -> Option<DOMString> {
        self.attribute().and_then(|attr| {
            let attr = attr.r();
            attr.value().tokens().and_then(|tokens| {
                tokens.get(index as usize).map(|token| (**token).to_owned())
            })
        })
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<DOMString> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-contains
    fn Contains(self, token: DOMString) -> Fallible<bool> {
        self.check_token_exceptions(&token).map(|token| {
            self.attribute().map(|attr| {
                let attr = attr.r();
                attr.value()
                    .tokens()
                    .expect("Should have parsed this attribute")
                    .iter()
                    .any(|atom| *atom == token)
            }).unwrap_or(false)
        })
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-add
    fn Add(self, tokens: Vec<DOMString>) -> ErrorResult {
        let element = self.element.root();
        let mut atoms = element.r().get_tokenlist_attribute(&self.local_name);
        for token in tokens.iter() {
            let token = try!(self.check_token_exceptions(&token));
            if !atoms.iter().any(|atom| *atom == token) {
                atoms.push(token);
            }
        }
        element.r().set_atomic_tokenlist_attribute(&self.local_name, atoms);
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-remove
    fn Remove(self, tokens: Vec<DOMString>) -> ErrorResult {
        let element = self.element.root();
        let mut atoms = element.r().get_tokenlist_attribute(&self.local_name);
        for token in tokens.iter() {
            let token = try!(self.check_token_exceptions(&token));
            atoms.iter().position(|atom| *atom == token).map(|index| {
                atoms.remove(index)
            });
        }
        element.r().set_atomic_tokenlist_attribute(&self.local_name, atoms);
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-toggle
    fn Toggle(self, token: DOMString, force: Option<bool>) -> Fallible<bool> {
        let element = self.element.root();
        let mut atoms = element.r().get_tokenlist_attribute(&self.local_name);
        let token = try!(self.check_token_exceptions(&token));
        match atoms.iter().position(|atom| *atom == token) {
            Some(index) => match force {
                Some(true) => Ok(true),
                _ => {
                    atoms.remove(index);
                    element.r().set_atomic_tokenlist_attribute(&self.local_name, atoms);
                    Ok(false)
                }
            },
            None => match force {
                Some(false) => Ok(false),
                _ => {
                    atoms.push(token);
                    element.r().set_atomic_tokenlist_attribute(&self.local_name, atoms);
                    Ok(true)
                }
            }
        }
    }

    // https://dom.spec.whatwg.org/#stringification-behavior
    fn Stringifier(self) -> DOMString {
        self.element.root().r().get_string_attribute(&self.local_name)
    }
}
