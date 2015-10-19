/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use devtools_traits::AttrInfo;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::{self, AttrMethods};
use dom::bindings::conversions::Castable;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap};
use dom::bindings::js::{LayoutJS, Root, RootedReference};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::element::{AttributeMutation, Element};
use dom::node::Node;
use dom::values::UNSIGNED_LONG_MAX;
use dom::virtualmethods::vtable_for;
use dom::window::Window;
use std::borrow::ToOwned;
use std::cell::Ref;
use std::mem;
use std::ops::Deref;
use string_cache::{Atom, Namespace};
use style::values::specified::Length;
use util::str::{DOMString, parse_unsigned_integer, split_html_space_chars, str_join};

#[derive(JSTraceable, PartialEq, Clone, HeapSizeOf)]
pub enum AttrValue {
    String(DOMString),
    TokenList(DOMString, Vec<Atom>),
    UInt(DOMString, u32),
    Atom(Atom),
    Length(DOMString, Option<Length>),
    Color(DOMString, Option<RGBA>),
}

impl AttrValue {
    pub fn from_serialized_tokenlist(tokens: DOMString) -> AttrValue {
        let atoms =
            split_html_space_chars(&tokens)
            .map(Atom::from_slice)
            .fold(vec![], |mut acc, atom| {
                if !acc.contains(&atom) { acc.push(atom) }
                acc
            });
        AttrValue::TokenList(tokens, atoms)
    }

    pub fn from_atomic_tokens(atoms: Vec<Atom>) -> AttrValue {
        let tokens = str_join(&atoms, "\x20");
        AttrValue::TokenList(tokens, atoms)
    }

    // https://html.spec.whatwg.org/multipage/#reflecting-content-attributes-in-idl-attributes:idl-unsigned-long
    pub fn from_u32(string: DOMString, default: u32) -> AttrValue {
        let result = parse_unsigned_integer(string.chars()).unwrap_or(default);
        let result = if result > UNSIGNED_LONG_MAX {
            default
        } else {
            result
        };
        AttrValue::UInt(string, result)
    }

    // https://html.spec.whatwg.org/multipage/#limited-to-only-non-negative-numbers-greater-than-zero
    pub fn from_limited_u32(string: DOMString, default: u32) -> AttrValue {
        let result = parse_unsigned_integer(string.chars()).unwrap_or(default);
        let result = if result == 0 || result > UNSIGNED_LONG_MAX {
            default
        } else {
            result
        };
        AttrValue::UInt(string, result)
    }

    pub fn from_atomic(string: DOMString) -> AttrValue {
        let value = Atom::from_slice(&string);
        AttrValue::Atom(value)
    }

    /// Assumes the `AttrValue` is a `TokenList` and returns its tokens
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `TokenList`
    pub fn as_tokens(&self) -> &[Atom] {
        match *self {
            AttrValue::TokenList(_, ref tokens) => tokens,
            _ => panic!("Tokens not found"),
        }
    }

    /// Assumes the `AttrValue` is an `Atom` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not an `Atom`
    pub fn as_atom(&self) -> &Atom {
        match *self {
            AttrValue::Atom(ref value) => value,
            _ => panic!("Atom not found"),
        }
    }

    /// Assumes the `AttrValue` is a `Color` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `Color`
    pub fn as_color(&self) -> Option<&RGBA> {
        match *self {
            AttrValue::Color(_, ref color) => color.as_ref(),
            _ => panic!("Color not found"),
        }
    }

    /// Assumes the `AttrValue` is a `Length` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `Length`
    pub fn as_length(&self) -> Option<&Length> {
        match *self {
            AttrValue::Length(_, ref length) => length.as_ref(),
            _ => panic!("Length not found"),
        }
    }

    /// Return the AttrValue as its integer representation, if any.
    /// This corresponds to attribute values returned as `AttrValue::UInt(_)`
    /// by `VirtualMethods::parse_plain_attribute()`.
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `UInt`
    pub fn as_uint(&self) -> u32 {
        if let AttrValue::UInt(_, value) = *self {
            value
        } else {
            panic!("Uint not found");
        }
    }
}

impl Deref for AttrValue {
    type Target = str;

    fn deref(&self) -> &str {
        match *self {
            AttrValue::String(ref value) |
                AttrValue::TokenList(ref value, _) |
                AttrValue::UInt(ref value, _) |
                AttrValue::Length(ref value, _) |
                AttrValue::Color(ref value, _) => &value,
            AttrValue::Atom(ref value) => &value,
        }
    }
}

// https://dom.spec.whatwg.org/#interface-attr
#[dom_struct]
pub struct Attr {
    reflector_: Reflector,
    local_name: Atom,
    value: DOMRefCell<AttrValue>,
    name: Atom,
    namespace: Namespace,
    prefix: Option<Atom>,

    /// the element that owns this attribute.
    owner: MutNullableHeap<JS<Element>>,
}

impl Attr {
    fn new_inherited(local_name: Atom, value: AttrValue, name: Atom, namespace: Namespace,
                     prefix: Option<Atom>, owner: Option<&Element>) -> Attr {
        Attr {
            reflector_: Reflector::new(),
            local_name: local_name,
            value: DOMRefCell::new(value),
            name: name,
            namespace: namespace,
            prefix: prefix,
            owner: MutNullableHeap::new(owner),
        }
    }

    pub fn new(window: &Window, local_name: Atom, value: AttrValue,
               name: Atom, namespace: Namespace,
               prefix: Option<Atom>, owner: Option<&Element>) -> Root<Attr> {
        reflect_dom_object(
            box Attr::new_inherited(local_name, value, name, namespace, prefix, owner),
            GlobalRef::Window(window),
            AttrBinding::Wrap)
    }

    #[inline]
    pub fn name(&self) -> &Atom {
        &self.name
    }

    #[inline]
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    #[inline]
    pub fn prefix(&self) -> &Option<Atom> {
        &self.prefix
    }
}

impl AttrMethods for Attr {
    // https://dom.spec.whatwg.org/#dom-attr-localname
    fn LocalName(&self) -> DOMString {
        (**self.local_name()).to_owned()
    }

    // https://dom.spec.whatwg.org/#dom-attr-value
    fn Value(&self) -> DOMString {
        (**self.value()).to_owned()
    }

    // https://dom.spec.whatwg.org/#dom-attr-value
    fn SetValue(&self, value: DOMString) {
        match self.owner() {
            None => *self.value.borrow_mut() = AttrValue::String(value),
            Some(owner) => {
                let value = owner.r().parse_attribute(&self.namespace, self.local_name(), value);
                self.set_value(value, owner.r());
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-attr-textcontent
    fn TextContent(&self) -> DOMString {
        self.Value()
    }

    // https://dom.spec.whatwg.org/#dom-attr-textcontent
    fn SetTextContent(&self, value: DOMString) {
        self.SetValue(value)
    }

    // https://dom.spec.whatwg.org/#dom-attr-nodevalue
    fn NodeValue(&self) -> DOMString {
        self.Value()
    }

    // https://dom.spec.whatwg.org/#dom-attr-nodevalue
    fn SetNodeValue(&self, value: DOMString) {
        self.SetValue(value)
    }

    // https://dom.spec.whatwg.org/#dom-attr-name
    fn Name(&self) -> DOMString {
        (*self.name).to_owned()
    }

    // https://dom.spec.whatwg.org/#dom-attr-namespaceuri
    fn GetNamespaceURI(&self) -> Option<DOMString> {
        let Namespace(ref atom) = self.namespace;
        match &**atom {
            "" => None,
            url => Some(url.to_owned()),
        }
    }

    // https://dom.spec.whatwg.org/#dom-attr-prefix
    fn GetPrefix(&self) -> Option<DOMString> {
        self.prefix().as_ref().map(|p| (**p).to_owned())
    }

    // https://dom.spec.whatwg.org/#dom-attr-ownerelement
    fn GetOwnerElement(&self) -> Option<Root<Element>> {
        self.owner()
    }

    // https://dom.spec.whatwg.org/#dom-attr-specified
    fn Specified(&self) -> bool {
        true // Always returns true
    }
}


impl Attr {
    pub fn set_value(&self, mut value: AttrValue, owner: &Element) {
        assert!(Some(owner) == self.owner().r());
        mem::swap(&mut *self.value.borrow_mut(), &mut value);
        if self.namespace == ns!("") {
            vtable_for(owner.upcast::<Node>()).attribute_mutated(
                self, AttributeMutation::Set(Some(&value)));
        }
    }

    pub fn value(&self) -> Ref<AttrValue> {
        self.value.borrow()
    }

    pub fn local_name(&self) -> &Atom {
        &self.local_name
    }

    /// Sets the owner element. Should be called after the attribute is added
    /// or removed from its older parent.
    pub fn set_owner(&self, owner: Option<&Element>) {
        let ref ns = self.namespace;
        match (self.owner().r(), owner) {
            (None, Some(new)) => {
                // Already in the list of attributes of new owner.
                assert!(new.get_attribute(&ns, &self.local_name) == Some(Root::from_ref(self)))
            }
            (Some(old), None) => {
                // Already gone from the list of attributes of old owner.
                assert!(old.get_attribute(&ns, &self.local_name).is_none())
            }
            (old, new) => assert!(old == new)
        }
        self.owner.set(owner);
    }

    pub fn owner(&self) -> Option<Root<Element>> {
        self.owner.get_rooted()
    }

    pub fn summarize(&self) -> AttrInfo {
        let Namespace(ref ns) = self.namespace;
        AttrInfo {
            namespace: (**ns).to_owned(),
            name: self.Name(),
            value: self.Value(),
        }
    }
}

#[allow(unsafe_code)]
pub trait AttrHelpersForLayout {
    unsafe fn value_forever(&self) -> &'static AttrValue;
    unsafe fn value_ref_forever(&self) -> &'static str;
    unsafe fn value_atom_forever(&self) -> Option<Atom>;
    unsafe fn value_tokens_forever(&self) -> Option<&'static [Atom]>;
    unsafe fn local_name_atom_forever(&self) -> Atom;
    unsafe fn value_for_layout(&self) -> &AttrValue;
}

#[allow(unsafe_code)]
impl AttrHelpersForLayout for LayoutJS<Attr> {
    #[inline]
    unsafe fn value_forever(&self) -> &'static AttrValue {
        // This transmute is used to cheat the lifetime restriction.
        mem::transmute::<&AttrValue, &AttrValue>((*self.unsafe_get()).value.borrow_for_layout())
    }

    #[inline]
    unsafe fn value_ref_forever(&self) -> &'static str {
        &**self.value_forever()
    }

    #[inline]
    unsafe fn value_atom_forever(&self) -> Option<Atom> {
        let value = (*self.unsafe_get()).value.borrow_for_layout();
        match *value {
            AttrValue::Atom(ref val) => Some(val.clone()),
            _ => None,
        }
    }

    #[inline]
    unsafe fn value_tokens_forever(&self) -> Option<&'static [Atom]> {
        // This transmute is used to cheat the lifetime restriction.
        match *self.value_forever() {
            AttrValue::TokenList(_, ref tokens) => Some(tokens),
            _ => None,
        }
    }

    #[inline]
    unsafe fn local_name_atom_forever(&self) -> Atom {
        (*self.unsafe_get()).local_name.clone()
    }

    #[inline]
    unsafe fn value_for_layout(&self) -> &AttrValue {
        (*self.unsafe_get()).value.borrow_for_layout()
    }
}
