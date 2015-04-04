/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::{self, AttrMethods};
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, MutNullableJS, Temporary};
use dom::bindings::js::{OptionalRootable, OptionalRootedRootable};
use dom::bindings::js::RootedReference;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::element::{Element, AttributeHandlers};
use dom::node::Node;
use dom::window::Window;
use dom::virtualmethods::vtable_for;

use devtools_traits::AttrInfo;
use util::str::{DOMString, split_html_space_chars};

use string_cache::{Atom, Namespace};

use std::borrow::ToOwned;
use std::cell::Ref;
use std::mem;

pub enum AttrSettingType {
    FirstSetAttr,
    ReplacedAttr,
}

#[derive(PartialEq, Clone)]
#[jstraceable]
pub enum AttrValue {
    String(DOMString),
    TokenList(DOMString, Vec<Atom>),
    UInt(DOMString, u32),
    Atom(Atom),
}

impl AttrValue {
    pub fn from_serialized_tokenlist(tokens: DOMString) -> AttrValue {
        let mut atoms: Vec<Atom> = vec!();
        for token in split_html_space_chars(&tokens).map(Atom::from_slice) {
            if !atoms.iter().any(|atom| *atom == token) {
                atoms.push(token);
            }
        }
        AttrValue::TokenList(tokens, atoms)
    }

    pub fn from_atomic_tokens(atoms: Vec<Atom>) -> AttrValue {
        let tokens = atoms.iter().map(Atom::as_slice).collect::<Vec<_>>().connect("\x20");
        AttrValue::TokenList(tokens, atoms)
    }

    pub fn from_u32(string: DOMString, default: u32) -> AttrValue {
        // XXX Is parse() correct?
        let result: u32 = string.parse().unwrap_or(default);
        AttrValue::UInt(string, result)
    }

    pub fn from_atomic(string: DOMString) -> AttrValue {
        let value = Atom::from_slice(&string);
        AttrValue::Atom(value)
    }

    pub fn tokens<'a>(&'a self) -> Option<&'a [Atom]> {
        match *self {
            AttrValue::TokenList(_, ref tokens) => Some(tokens.as_slice()),
            _ => None
        }
    }
}

impl Str for AttrValue {
    fn as_slice<'a>(&'a self) -> &'a str {
        match *self {
            AttrValue::String(ref value) |
                AttrValue::TokenList(ref value, _) |
                AttrValue::UInt(ref value, _) => &value,
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
    prefix: Option<DOMString>,

    /// the element that owns this attribute.
    owner: MutNullableJS<Element>,
}

impl Attr {
    fn new_inherited(local_name: Atom, value: AttrValue, name: Atom, namespace: Namespace,
                     prefix: Option<DOMString>, owner: Option<JSRef<Element>>) -> Attr {
        Attr {
            reflector_: Reflector::new(),
            local_name: local_name,
            value: DOMRefCell::new(value),
            name: name,
            namespace: namespace,
            prefix: prefix,
            owner: MutNullableJS::new(owner),
        }
    }

    pub fn new(window: JSRef<Window>, local_name: Atom, value: AttrValue,
               name: Atom, namespace: Namespace,
               prefix: Option<DOMString>, owner: Option<JSRef<Element>>) -> Temporary<Attr> {
        reflect_dom_object(
            box Attr::new_inherited(local_name, value, name, namespace, prefix, owner),
            GlobalRef::Window(window),
            AttrBinding::Wrap)
    }

    #[inline]
    pub fn name<'a>(&'a self) -> &'a Atom {
        &self.name
    }

    #[inline]
    pub fn namespace<'a>(&'a self) -> &'a Namespace {
        &self.namespace
    }

    #[inline]
    pub fn prefix<'a>(&'a self) -> &'a Option<DOMString> {
        &self.prefix
    }
}

impl<'a> AttrMethods for JSRef<'a, Attr> {
    // https://dom.spec.whatwg.org/#dom-attr-localname
    fn LocalName(self) -> DOMString {
        self.local_name().as_slice().to_owned()
    }

    // https://dom.spec.whatwg.org/#dom-attr-value
    fn Value(self) -> DOMString {
        self.value().as_slice().to_owned()
    }

    // https://dom.spec.whatwg.org/#dom-attr-value
    fn SetValue(self, value: DOMString) {
        match self.owner() {
            None => *self.value.borrow_mut() = AttrValue::String(value),
            Some(o) => {
                let owner = o.root();
                let value = owner.r().parse_attribute(&self.namespace, self.local_name(), value);
                self.set_value(AttrSettingType::ReplacedAttr, value, owner.r());
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-attr-textcontent
    fn TextContent(self) -> DOMString {
        self.Value()
    }

    // https://dom.spec.whatwg.org/#dom-attr-textcontent
    fn SetTextContent(self, value: DOMString) {
        self.SetValue(value)
    }

    // https://dom.spec.whatwg.org/#dom-attr-nodevalue
    fn NodeValue(self) -> DOMString {
        self.Value()
    }

    // https://dom.spec.whatwg.org/#dom-attr-nodevalue
    fn SetNodeValue(self, value: DOMString) {
        self.SetValue(value)
    }

    // https://dom.spec.whatwg.org/#dom-attr-name
    fn Name(self) -> DOMString {
        self.name.as_slice().to_owned()
    }

    // https://dom.spec.whatwg.org/#dom-attr-namespaceuri
    fn GetNamespaceURI(self) -> Option<DOMString> {
        let Namespace(ref atom) = self.namespace;
        match atom.as_slice() {
            "" => None,
            url => Some(url.to_owned()),
        }
    }

    // https://dom.spec.whatwg.org/#dom-attr-prefix
    fn GetPrefix(self) -> Option<DOMString> {
        self.prefix.clone()
    }

    // https://dom.spec.whatwg.org/#dom-attr-ownerelement
    fn GetOwnerElement(self) -> Option<Temporary<Element>> {
        self.owner()
    }

    // https://dom.spec.whatwg.org/#dom-attr-specified
    fn Specified(self) -> bool {
        true // Always returns true
    }
}

pub trait AttrHelpers<'a> {
    fn set_value(self, set_type: AttrSettingType, value: AttrValue, owner: JSRef<Element>);
    fn value(self) -> Ref<'a, AttrValue>;
    fn local_name(self) -> &'a Atom;
    fn set_owner(self, owner: Option<JSRef<Element>>);
    fn owner(self) -> Option<Temporary<Element>>;
    fn summarize(self) -> AttrInfo;
}

impl<'a> AttrHelpers<'a> for JSRef<'a, Attr> {
    fn set_value(self, set_type: AttrSettingType, value: AttrValue, owner: JSRef<Element>) {
        assert!(Some(owner) == self.owner().root().r());

        let node: JSRef<Node> = NodeCast::from_ref(owner);
        let namespace_is_null = self.namespace == ns!("");

        match set_type {
            AttrSettingType::ReplacedAttr if namespace_is_null =>
                vtable_for(&node).before_remove_attr(self),
            _ => ()
        }

        *self.value.borrow_mut() = value;

        if namespace_is_null {
            vtable_for(&node).after_set_attr(self)
        }
    }

    fn value(self) -> Ref<'a, AttrValue> {
        self.extended_deref().value.borrow()
    }

    fn local_name(self) -> &'a Atom {
        &self.extended_deref().local_name
    }

    /// Sets the owner element. Should be called after the attribute is added
    /// or removed from its older parent.
    fn set_owner(self, owner: Option<JSRef<Element>>) {
        let ref ns = self.namespace;
        match (self.owner().root().r(), owner) {
            (None, Some(new)) => {
                // Already in the list of attributes of new owner.
                assert!(new.get_attribute(&ns, &self.local_name).root().r() == Some(self))
            }
            (Some(old), None) => {
                // Already gone from the list of attributes of old owner.
                assert!(old.get_attribute(&ns, &self.local_name).is_none())
            }
            (old, new) => assert!(old == new)
        }
        self.owner.assign(owner)
    }

    fn owner(self) -> Option<Temporary<Element>> {
        self.owner.get()
    }

    fn summarize(self) -> AttrInfo {
        let Namespace(ref ns) = self.namespace;
        AttrInfo {
            namespace: ns.as_slice().to_owned(),
            name: self.Name(),
            value: self.Value(),
        }
    }
}

#[allow(unsafe_code)]
pub trait AttrHelpersForLayout {
    unsafe fn value_ref_forever(&self) -> &'static str;
    unsafe fn value_atom_forever(&self) -> Option<Atom>;
    unsafe fn value_tokens_forever(&self) -> Option<&'static [Atom]>;
    unsafe fn local_name_atom_forever(&self) -> Atom;
}

#[allow(unsafe_code)]
impl AttrHelpersForLayout for Attr {
    #[inline]
    unsafe fn value_ref_forever(&self) -> &'static str {
        // This transmute is used to cheat the lifetime restriction.
        let value = mem::transmute::<&AttrValue, &AttrValue>(self.value.borrow_for_layout());
        value.as_slice()
    }

    #[inline]
    unsafe fn value_atom_forever(&self) -> Option<Atom> {
        let value = self.value.borrow_for_layout();
        match *value {
            AttrValue::Atom(ref val) => Some(val.clone()),
            _ => None,
        }
    }

    #[inline]
    unsafe fn value_tokens_forever(&self) -> Option<&'static [Atom]> {
        // This transmute is used to cheat the lifetime restriction.
        let value = mem::transmute::<&AttrValue, &AttrValue>(self.value.borrow_for_layout());
        match *value {
            AttrValue::TokenList(_, ref tokens) => Some(tokens.as_slice()),
            _ => None,
        }
    }

    #[inline]
    unsafe fn local_name_atom_forever(&self) -> Atom {
        self.local_name.clone()
    }
}
