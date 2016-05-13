/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::AttrInfo;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::{self, AttrMethods};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap};
use dom::bindings::js::{LayoutJS, Root, RootedReference};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::element::{AttributeMutation, Element};
use dom::virtualmethods::vtable_for;
use dom::window::Window;
use std::borrow::ToOwned;
use std::cell::Ref;
use std::mem;
use string_cache::{Atom, Namespace};
use style::attr::{AttrIdentifier, AttrValue};

// https://dom.spec.whatwg.org/#interface-attr
#[dom_struct]
pub struct Attr {
    reflector_: Reflector,
    identifier: AttrIdentifier,
    value: DOMRefCell<AttrValue>,

    /// the element that owns this attribute.
    owner: MutNullableHeap<JS<Element>>,
}

impl Attr {
    fn new_inherited(local_name: Atom,
                     value: AttrValue,
                     name: Atom,
                     namespace: Namespace,
                     prefix: Option<Atom>,
                     owner: Option<&Element>)
                     -> Attr {
        Attr {
            reflector_: Reflector::new(),
            identifier: AttrIdentifier {
                local_name: local_name,
                name: name,
                namespace: namespace,
                prefix: prefix,
            },
            value: DOMRefCell::new(value),
            owner: MutNullableHeap::new(owner),
        }
    }

    pub fn new(window: &Window,
               local_name: Atom,
               value: AttrValue,
               name: Atom,
               namespace: Namespace,
               prefix: Option<Atom>,
               owner: Option<&Element>)
               -> Root<Attr> {
        reflect_dom_object(box Attr::new_inherited(local_name,
                                                   value,
                                                   name,
                                                   namespace,
                                                   prefix,
                                                   owner),
                           window,
                           AttrBinding::Wrap)
    }

    #[inline]
    pub fn name(&self) -> &Atom {
        &self.identifier.name
    }

    #[inline]
    pub fn namespace(&self) -> &Namespace {
        &self.identifier.namespace
    }

    #[inline]
    pub fn prefix(&self) -> &Option<Atom> {
        &self.identifier.prefix
    }
}

impl AttrMethods for Attr {
    // https://dom.spec.whatwg.org/#dom-attr-localname
    fn LocalName(&self) -> DOMString {
        // FIXME(ajeffrey): convert directly from Atom to DOMString
        DOMString::from(&**self.local_name())
    }

    // https://dom.spec.whatwg.org/#dom-attr-value
    fn Value(&self) -> DOMString {
        // FIXME(ajeffrey): convert directly from AttrValue to DOMString
        DOMString::from(&**self.value())
    }

    // https://dom.spec.whatwg.org/#dom-attr-value
    fn SetValue(&self, value: DOMString) {
        if let Some(owner) = self.owner() {
            let value = owner.parse_attribute(&self.identifier.namespace,
                                              self.local_name(),
                                              value);
            self.set_value(value, &owner);
        } else {
            *self.value.borrow_mut() = AttrValue::String(value.into());
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
        // FIXME(ajeffrey): convert directly from Atom to DOMString
        DOMString::from(&*self.identifier.name)
    }

    // https://dom.spec.whatwg.org/#dom-attr-nodename
    fn NodeName(&self) -> DOMString {
        self.Name()
    }

    // https://dom.spec.whatwg.org/#dom-attr-namespaceuri
    fn GetNamespaceURI(&self) -> Option<DOMString> {
        let Namespace(ref atom) = self.identifier.namespace;
        match &**atom {
            "" => None,
            url => Some(DOMString::from(url)),
        }
    }

    // https://dom.spec.whatwg.org/#dom-attr-prefix
    fn GetPrefix(&self) -> Option<DOMString> {
        // FIXME(ajeffrey): convert directly from Atom to DOMString
        self.prefix().as_ref().map(|p| DOMString::from(&**p))
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
        owner.will_mutate_attr();
        self.swap_value(&mut value);
        if self.identifier.namespace == ns!() {
            vtable_for(owner.upcast())
                .attribute_mutated(self, AttributeMutation::Set(Some(&value)));
        }
    }

    /// Used to swap the attribute's value without triggering mutation events
    pub fn swap_value(&self, value: &mut AttrValue) {
        mem::swap(&mut *self.value.borrow_mut(), value);
    }

    pub fn identifier(&self) -> &AttrIdentifier {
        &self.identifier
    }

    pub fn value(&self) -> Ref<AttrValue> {
        self.value.borrow()
    }

    pub fn local_name(&self) -> &Atom {
        &self.identifier.local_name
    }

    /// Sets the owner element. Should be called after the attribute is added
    /// or removed from its older parent.
    pub fn set_owner(&self, owner: Option<&Element>) {
        let ns = &self.identifier.namespace;
        match (self.owner(), owner) {
            (Some(old), None) => {
                // Already gone from the list of attributes of old owner.
                assert!(old.get_attribute(&ns, &self.identifier.local_name).r() != Some(self))
            }
            (Some(old), Some(new)) => assert!(&*old == new),
            _ => {},
        }
        self.owner.set(owner);
    }

    pub fn owner(&self) -> Option<Root<Element>> {
        self.owner.get()
    }

    pub fn summarize(&self) -> AttrInfo {
        let Namespace(ref ns) = self.identifier.namespace;
        AttrInfo {
            namespace: (**ns).to_owned(),
            name: String::from(self.Name()),
            value: String::from(self.Value()),
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
        (*self.unsafe_get()).identifier.local_name.clone()
    }

    #[inline]
    unsafe fn value_for_layout(&self) -> &AttrValue {
        (*self.unsafe_get()).value.borrow_for_layout()
    }
}
