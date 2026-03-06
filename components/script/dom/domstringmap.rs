/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, ns};

use crate::dom::bindings::codegen::Bindings::DOMStringMapBinding::DOMStringMapMethods;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::xmlname::matches_name_production;
use crate::dom::element::Element;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::NodeTraits;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct DOMStringMap {
    reflector_: Reflector,
    element: Dom<HTMLElement>,
}

static DATA_PREFIX: &str = "data-";
static DATA_HYPHEN_SEPARATOR: char = '\x2d';

/// <https://html.spec.whatwg.org/multipage/#concept-domstringmap-pairs>
fn to_camel_case(name: &str) -> Option<DOMString> {
    // Step 2. For each content attribute on the DOMStringMap's associated element whose
    // first five characters are the string "data-" and whose remaining characters (if any)
    // do not include any ASCII upper alphas, in the order that those attributes
    // are listed in the element's attribute list,
    // add a name-value pair to list whose name is the attribute's name with the first
    // five characters removed and whose value is the attribute's value.
    let name = name.strip_prefix(DATA_PREFIX)?;
    let has_uppercase = name.chars().any(|curr_char| curr_char.is_ascii_uppercase());
    if has_uppercase {
        return None;
    }
    // Step 3. For each name in list, for each U+002D HYPHEN-MINUS character (-)
    // in the name that is followed by an ASCII lower alpha, remove the
    // U+002D HYPHEN-MINUS character (-) and replace the character that followed
    // it by the same character converted to ASCII uppercase.
    let mut result = String::with_capacity(name.len().saturating_sub(DATA_PREFIX.len()));
    let mut name_chars = name.chars().peekable();
    while let Some(curr_char) = name_chars.next() {
        // Note that we first need to peek, since we shouldn't advance the iterator twice
        // in case there are two consecutive dashes and then followed by a ASCII lower alpha
        if curr_char == DATA_HYPHEN_SEPARATOR &&
            name_chars
                .peek()
                .is_some_and(|next_char| next_char.is_ascii_lowercase())
        {
            result.push(
                name_chars
                    .next()
                    .expect("Already called peek")
                    .to_ascii_uppercase(),
            );
            continue;
        }
        result.push(curr_char);
    }
    // Step 1. Let list be an empty list of name-value pairs.
    // Step 4. Return list.
    //
    // We do the iteration in the calling function, to avoid needlessly computing attribute
    // values when we only need the names. Therefore, we only return the name.
    Some(DOMString::from(result))
}

/// <https://html.spec.whatwg.org/multipage/#dom-domstringmap-setitem>
/// and <https://html.spec.whatwg.org/multipage/#dom-domstringmap-removeitem>
fn to_snake_case(name: &DOMString, should_throw: bool) -> Option<String> {
    let name = name.str();
    let mut result = String::with_capacity(DATA_PREFIX.len() + name.len());
    // > Insert the string data- at the front of name.
    result.push_str(DATA_PREFIX);
    let mut name_chars = name.chars();
    while let Some(curr_char) = name_chars.next() {
        if curr_char == DATA_HYPHEN_SEPARATOR {
            result.push(curr_char);

            if let Some(next_char) = name_chars.next() {
                // Only relevant for https://html.spec.whatwg.org/multipage/#dom-domstringmap-setitem
                //
                // > If name contains a U+002D HYPHEN-MINUS character (-) followed by an ASCII lower alpha,
                // > then throw a "SyntaxError" DOMException.
                if next_char.is_ascii_lowercase() {
                    if should_throw {
                        return None;
                    }
                    result.push(next_char);
                } else {
                    // > For each ASCII upper alpha in name, insert a U+002D HYPHEN-MINUS character (-) before the character
                    // > and replace the character with the same character converted to ASCII lowercase.
                    result.push(DATA_HYPHEN_SEPARATOR);
                    result.push(next_char.to_ascii_lowercase());
                }
            }
        } else {
            // > For each ASCII upper alpha in name, insert a U+002D HYPHEN-MINUS character (-) before the character
            // > and replace the character with the same character converted to ASCII lowercase.
            if curr_char.is_ascii_uppercase() {
                result.push(DATA_HYPHEN_SEPARATOR);
                result.push(curr_char.to_ascii_lowercase());
            } else {
                result.push(curr_char);
            }
        }
    }
    Some(result)
}

impl DOMStringMap {
    fn new_inherited(element: &HTMLElement) -> DOMStringMap {
        DOMStringMap {
            reflector_: Reflector::new(),
            element: Dom::from_ref(element),
        }
    }

    pub(crate) fn new(element: &HTMLElement, can_gc: CanGc) -> DomRoot<DOMStringMap> {
        reflect_dom_object(
            Box::new(DOMStringMap::new_inherited(element)),
            &*element.owner_window(),
            can_gc,
        )
    }

    fn as_element(&self) -> &Element {
        self.element.upcast::<Element>()
    }
}

// https://html.spec.whatwg.org/multipage/#domstringmap
impl DOMStringMapMethods<crate::DomTypeHolder> for DOMStringMap {
    /// <https://html.spec.whatwg.org/multipage/#dom-domstringmap-removeitem>
    fn NamedDeleter(&self, cx: &mut js::context::JSContext, name: DOMString) {
        // Step 1. For each ASCII upper alpha in name, insert a U+002D HYPHEN-MINUS character (-) before the character
        // and replace the character with the same character converted to ASCII lowercase.
        // Step 2. Insert the string data- at the front of name.
        let name = to_snake_case(&name, false).expect("Must always succeed");
        // Step 3. Remove an attribute by name given name and the DOMStringMap's associated element.
        self.as_element()
            .remove_attribute(&ns!(), &LocalName::from(name), CanGc::from_cx(cx));
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-domstringmap-setitem>
    fn NamedSetter(
        &self,
        cx: &mut js::context::JSContext,
        name: DOMString,
        value: DOMString,
    ) -> ErrorResult {
        // Step 2. For each ASCII upper alpha in name, insert a U+002D HYPHEN-MINUS character (-)
        // before the character and replace the character with the same character converted to ASCII lowercase.
        // Step 3. Insert the string data- at the front of name.
        let Some(name) = to_snake_case(&name, true) else {
            // Step 1. If name contains a U+002D HYPHEN-MINUS character (-) followed by an ASCII lower alpha,
            // then throw a "SyntaxError" DOMException.
            return Err(Error::Syntax(None));
        };
        // Step 4. If name is not a valid attribute local name, then throw an "InvalidCharacterError" DOMException.
        if !matches_name_production(&name) {
            return Err(Error::InvalidCharacter(None));
        }
        // Step 5. Set an attribute value for the DOMStringMap's associated element using name and value.
        let name = LocalName::from(name);
        let element = self.as_element();
        let value = element.parse_attribute(&ns!(), &name, value);
        element.set_attribute_with_namespace(cx, name.clone(), value, name, ns!(), None);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-domstringmap-nameditem>
    fn NamedGetter(&self, name: DOMString) -> Option<DOMString> {
        // > To determine the value of a named property name for a DOMStringMap,
        // > return the value component of the name-value pair whose name component is
        // > name in the list returned from getting the DOMStringMap's name-value pairs.
        self.as_element()
            .attrs()
            .iter()
            .find(|attr| to_camel_case(attr.local_name()).as_ref() == Some(&name))
            .map(|attr| DOMString::from(&**attr.value()))
    }

    /// <https://html.spec.whatwg.org/multipage/#the-domstringmap-interface:supported-property-names>
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        // > The supported property names on a DOMStringMap object at any instant are
        // > the names of each pair returned from getting the DOMStringMap's name-value
        // > pairs at that instant, in the order returned.
        self.as_element()
            .attrs()
            .iter()
            .filter_map(|attr| to_camel_case(attr.local_name()))
            .collect()
    }
}
