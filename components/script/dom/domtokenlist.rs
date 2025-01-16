/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{namespace_url, ns, LocalName};
use servo_atoms::Atom;
use style::str::HTML_SPACE_CHARACTERS;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::DOMTokenListBinding::DOMTokenListMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::node::NodeTraits;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct DOMTokenList {
    reflector_: Reflector,
    element: Dom<Element>,
    #[no_trace]
    local_name: LocalName,
    #[no_trace]
    supported_tokens: Option<Vec<Atom>>,
}

impl DOMTokenList {
    pub(crate) fn new_inherited(
        element: &Element,
        local_name: LocalName,
        supported_tokens: Option<Vec<Atom>>,
    ) -> DOMTokenList {
        DOMTokenList {
            reflector_: Reflector::new(),
            element: Dom::from_ref(element),
            local_name,
            supported_tokens,
        }
    }

    pub(crate) fn new(
        element: &Element,
        local_name: &LocalName,
        supported_tokens: Option<Vec<Atom>>,
    ) -> DomRoot<DOMTokenList> {
        reflect_dom_object(
            Box::new(DOMTokenList::new_inherited(
                element,
                local_name.clone(),
                supported_tokens,
            )),
            &*element.owner_window(),
            CanGc::note(),
        )
    }

    fn attribute(&self) -> Option<DomRoot<Attr>> {
        self.element.get_attribute(&ns!(), &self.local_name)
    }

    fn check_token_exceptions(&self, token: &str) -> Fallible<Atom> {
        match token {
            "" => Err(Error::Syntax),
            slice if slice.find(HTML_SPACE_CHARACTERS).is_some() => Err(Error::InvalidCharacter),
            slice => Ok(Atom::from(slice)),
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-dtl-update>
    fn perform_update_steps(&self, atoms: Vec<Atom>, can_gc: CanGc) {
        // Step 1
        if !self.element.has_attribute(&self.local_name) && atoms.is_empty() {
            return;
        }
        // step 2
        self.element
            .set_atomic_tokenlist_attribute(&self.local_name, atoms, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#concept-domtokenlist-validation>
    fn validation_steps(&self, token: &str) -> Fallible<bool> {
        match &self.supported_tokens {
            None => Err(Error::Type(
                "This attribute has no supported tokens".to_owned(),
            )),
            Some(supported_tokens) => {
                let token = Atom::from(token).to_ascii_lowercase();
                if supported_tokens
                    .iter()
                    .any(|supported_token| *supported_token == token)
                {
                    return Ok(true);
                }
                Ok(false)
            },
        }
    }
}

/// <https://dom.spec.whatwg.org/#domtokenlist>
impl DOMTokenListMethods<crate::DomTypeHolder> for DOMTokenList {
    /// <https://dom.spec.whatwg.org/#dom-domtokenlist-length>
    fn Length(&self) -> u32 {
        self.attribute()
            .map_or(0, |attr| attr.value().as_tokens().len()) as u32
    }

    /// <https://dom.spec.whatwg.org/#dom-domtokenlist-item>
    fn Item(&self, index: u32) -> Option<DOMString> {
        self.attribute().and_then(|attr| {
            // FIXME(ajeffrey): Convert directly from Atom to DOMString
            attr.value()
                .as_tokens()
                .get(index as usize)
                .map(|token| DOMString::from(&**token))
        })
    }

    /// <https://dom.spec.whatwg.org/#dom-domtokenlist-contains>
    fn Contains(&self, token: DOMString) -> bool {
        let token = Atom::from(token);
        self.attribute().is_some_and(|attr| {
            attr.value()
                .as_tokens()
                .iter()
                .any(|atom: &Atom| *atom == token)
        })
    }

    /// <https://dom.spec.whatwg.org/#dom-domtokenlist-add>
    fn Add(&self, tokens: Vec<DOMString>, can_gc: CanGc) -> ErrorResult {
        let mut atoms = self.element.get_tokenlist_attribute(&self.local_name);
        for token in &tokens {
            let token = self.check_token_exceptions(token)?;
            if !atoms.iter().any(|atom| *atom == token) {
                atoms.push(token);
            }
        }
        self.perform_update_steps(atoms, can_gc);
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-domtokenlist-remove>
    fn Remove(&self, tokens: Vec<DOMString>, can_gc: CanGc) -> ErrorResult {
        let mut atoms = self.element.get_tokenlist_attribute(&self.local_name);
        for token in &tokens {
            let token = self.check_token_exceptions(token)?;
            atoms
                .iter()
                .position(|atom| *atom == token)
                .map(|index| atoms.remove(index));
        }
        self.perform_update_steps(atoms, can_gc);
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-domtokenlist-toggle>
    fn Toggle(&self, token: DOMString, force: Option<bool>, can_gc: CanGc) -> Fallible<bool> {
        let mut atoms = self.element.get_tokenlist_attribute(&self.local_name);
        let token = self.check_token_exceptions(&token)?;
        match atoms.iter().position(|atom| *atom == token) {
            Some(index) => match force {
                Some(true) => Ok(true),
                _ => {
                    atoms.remove(index);
                    self.perform_update_steps(atoms, can_gc);
                    Ok(false)
                },
            },
            None => match force {
                Some(false) => Ok(false),
                _ => {
                    atoms.push(token);
                    self.perform_update_steps(atoms, can_gc);
                    Ok(true)
                },
            },
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-domtokenlist-value>
    fn Value(&self) -> DOMString {
        self.element.get_string_attribute(&self.local_name)
    }

    /// <https://dom.spec.whatwg.org/#dom-domtokenlist-value>
    fn SetValue(&self, value: DOMString, can_gc: CanGc) {
        self.element
            .set_tokenlist_attribute(&self.local_name, value, can_gc);
    }

    /// <https://dom.spec.whatwg.org/#dom-domtokenlist-replace>
    fn Replace(&self, token: DOMString, new_token: DOMString, can_gc: CanGc) -> Fallible<bool> {
        if token.is_empty() || new_token.is_empty() {
            // Step 1.
            return Err(Error::Syntax);
        }
        if token.contains(HTML_SPACE_CHARACTERS) || new_token.contains(HTML_SPACE_CHARACTERS) {
            // Step 2.
            return Err(Error::InvalidCharacter);
        }
        // Steps 3-4.
        let token = Atom::from(token);
        let new_token = Atom::from(new_token);
        let mut atoms = self.element.get_tokenlist_attribute(&self.local_name);
        let mut result = false;
        if let Some(pos) = atoms.iter().position(|atom| *atom == token) {
            match atoms.iter().position(|atom| *atom == new_token) {
                Some(redundant_pos) if redundant_pos > pos => {
                    // The replacement is already in the list, later,
                    // so we perform the replacement and remove the
                    // later copy.
                    atoms[pos] = new_token;
                    atoms.remove(redundant_pos);
                },
                Some(redundant_pos) if redundant_pos < pos => {
                    // The replacement is already in the list, earlier,
                    // so we remove the index where we'd be putting the
                    // later copy.
                    atoms.remove(pos);
                },
                Some(_) => {
                    // Else we are replacing the token with itself, nothing to change
                },
                None => {
                    // The replacement is not in the list already
                    atoms[pos] = new_token;
                },
            }

            // Step 5.
            self.perform_update_steps(atoms, can_gc);
            result = true;
        }
        Ok(result)
    }

    /// <https://dom.spec.whatwg.org/#dom-domtokenlist-supports>
    fn Supports(&self, token: DOMString) -> Fallible<bool> {
        self.validation_steps(&token)
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32) -> Option<DOMString> {
        self.Item(index)
    }
}
