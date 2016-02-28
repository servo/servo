/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::DOMTokenListBinding;
use dom::bindings::codegen::Bindings::DOMTokenListBinding::DOMTokenListMethods;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::element::Element;
use dom::node::window_from_node;
use string_cache::Atom;
use util::str::{DOMString, HTML_SPACE_CHARACTERS};

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

    fn attribute(&self) -> Option<Root<Attr>> {
        self.element.get_attribute(&ns!(), &self.local_name)
    }

    fn check_token_exceptions(&self, token: &str) -> Fallible<Atom> {
        match token {
            "" => Err(Error::Syntax),
            slice if slice.find(HTML_SPACE_CHARACTERS).is_some() => Err(Error::InvalidCharacter),
            slice => Ok(Atom::from(slice)),
        }
    }
}

// https://dom.spec.whatwg.org/#domtokenlist
impl DOMTokenListMethods for DOMTokenList {
    // https://dom.spec.whatwg.org/#dom-domtokenlist-length
    fn Length(&self) -> u32 {
        self.attribute().map_or(0, |attr| {
            let attr = attr.r();
            attr.value().as_tokens().len()
        }) as u32
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-item
    fn Item(&self, index: u32) -> Option<DOMString> {
        self.attribute().and_then(|attr| {
            // FIXME(ajeffrey): Convert directly from Atom to DOMString
            attr.value().as_tokens().get(index as usize).map(|token| DOMString::from(&**token))
        })
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-contains
    fn Contains(&self, token: DOMString) -> Fallible<bool> {
        self.check_token_exceptions(&token).map(|token| {
            self.attribute().map_or(false, |attr| {
                let attr = attr.r();
                attr.value()
                    .as_tokens()
                    .iter()
                    .any(|atom: &Atom| *atom == token)
            })
        })
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-add
    fn Add(&self, tokens: Vec<DOMString>) -> ErrorResult {
        let mut atoms = self.element.get_tokenlist_attribute(&self.local_name);
        for token in &tokens {
            let token = try!(self.check_token_exceptions(&token));
            if !atoms.iter().any(|atom| *atom == token) {
                atoms.push(token);
            }
        }
        self.element.set_atomic_tokenlist_attribute(&self.local_name, atoms);
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-remove
    fn Remove(&self, tokens: Vec<DOMString>) -> ErrorResult {
        let mut atoms = self.element.get_tokenlist_attribute(&self.local_name);
        for token in &tokens {
            let token = try!(self.check_token_exceptions(&token));
            atoms.iter().position(|atom| *atom == token).map(|index| atoms.remove(index));
        }
        self.element.set_atomic_tokenlist_attribute(&self.local_name, atoms);
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-toggle
    fn Toggle(&self, token: DOMString, force: Option<bool>) -> Fallible<bool> {
        let mut atoms = self.element.get_tokenlist_attribute(&self.local_name);
        let token = try!(self.check_token_exceptions(&token));
        match atoms.iter().position(|atom| *atom == token) {
            Some(index) => match force {
                Some(true) => Ok(true),
                _ => {
                    atoms.remove(index);
                    self.element.set_atomic_tokenlist_attribute(&self.local_name, atoms);
                    Ok(false)
                }
            },
            None => match force {
                Some(false) => Ok(false),
                _ => {
                    atoms.push(token);
                    self.element.set_atomic_tokenlist_attribute(&self.local_name, atoms);
                    Ok(true)
                }
            },
        }
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-value
    fn Value(&self) -> DOMString {
        self.element.get_string_attribute(&self.local_name)
    }

    // https://dom.spec.whatwg.org/#dom-domtokenlist-value
    fn SetValue(&self, value: DOMString) {
        self.element.set_tokenlist_attribute(&self.local_name, value);
    }

    // https://dom.spec.whatwg.org/#concept-dtl-serialize
    fn Stringifier(&self) -> DOMString {
        self.element.get_string_attribute(&self.local_name)
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<DOMString> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}
