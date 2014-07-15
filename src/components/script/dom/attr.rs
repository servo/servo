/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::AttrBinding;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::global::Window;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::element::{Element, AttributeHandlers};
use dom::node::Node;
use dom::window::Window;
use dom::virtualmethods::vtable_for;
use servo_util::namespace;
use servo_util::namespace::Namespace;
use servo_util::str::{DOMString, HTML_SPACE_CHARACTERS};
use std::cell::{Ref, Cell, RefCell};
use std::mem;

pub enum AttrSettingType {
    FirstSetAttr,
    ReplacedAttr,
}

#[deriving(PartialEq, Clone, Encodable)]
pub enum AttrValue {
    StringAttrValue(DOMString),
    TokenListAttrValue(DOMString, Vec<(uint, uint)>),
    UIntAttrValue(DOMString, u32),
}

impl AttrValue {
    pub fn from_tokenlist(list: DOMString) -> AttrValue {
        let mut indexes = vec![];
        let mut last_index: uint = 0;
        for (index, ch) in list.as_slice().char_indices() {
            if HTML_SPACE_CHARACTERS.iter().any(|&space| space == ch) {
                indexes.push((last_index, index));
                last_index = index + 1;
            }
        }
        return TokenListAttrValue(list, indexes);
    }

    pub fn from_u32(string: DOMString, default: u32) -> AttrValue {
        let result: u32 = from_str(string.as_slice()).unwrap_or(default);
        UIntAttrValue(string, result)
    }

    pub fn as_slice<'a>(&'a self) -> &'a str {
        match *self {
            StringAttrValue(ref value) |
            TokenListAttrValue(ref value, _) |
            UIntAttrValue(ref value, _) => value.as_slice(),
        }
    }
}

#[deriving(Encodable)]
pub struct Attr {
    reflector_: Reflector,
    pub local_name: DOMString,
    value: Traceable<RefCell<AttrValue>>,
    pub name: DOMString,
    pub namespace: Namespace,
    pub prefix: Option<DOMString>,

    /// the element that owns this attribute.
    owner: Cell<JS<Element>>,
}

impl Reflectable for Attr {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

impl Attr {
    fn new_inherited(local_name: DOMString, value: AttrValue,
                     name: DOMString, namespace: Namespace,
                     prefix: Option<DOMString>, owner: &JSRef<Element>) -> Attr {
        Attr {
            reflector_: Reflector::new(),
            local_name: local_name,
            value: Traceable::new(RefCell::new(value)),
            name: name, //TODO: Intern attribute names
            namespace: namespace,
            prefix: prefix,
            owner: Cell::new(JS::from_rooted(owner)),
        }
    }

    pub fn new(window: &JSRef<Window>, local_name: DOMString, value: AttrValue,
               name: DOMString, namespace: Namespace,
               prefix: Option<DOMString>, owner: &JSRef<Element>) -> Temporary<Attr> {
        let attr = Attr::new_inherited(local_name, value, name, namespace, prefix, owner);
        reflect_dom_object(box attr, &Window(*window), AttrBinding::Wrap)
    }

    pub fn set_value(&self, set_type: AttrSettingType, value: AttrValue) {
        let owner = self.owner.get().root();
        let node: &JSRef<Node> = NodeCast::from_ref(&*owner);
        let namespace_is_null = self.namespace == namespace::Null;

        match set_type {
            ReplacedAttr => {
                if namespace_is_null {
                    vtable_for(node).before_remove_attr(self.local_name.clone(), self.value.deref().borrow().as_slice().to_string());
                }
            }
            FirstSetAttr => {}
        }

        *self.value.deref().borrow_mut() = value;

        if namespace_is_null {
            vtable_for(node).after_set_attr(self.local_name.clone(), self.value.deref().borrow().as_slice().to_string());
        }
    }

    pub fn value<'a>(&'a self) -> Ref<'a, AttrValue> {
        self.value.deref().borrow()
    }
}

pub trait AttrMethods {
    fn LocalName(&self) -> DOMString;
    fn Value(&self) -> DOMString;
    fn SetValue(&self, value: DOMString);
    fn Name(&self) -> DOMString;
    fn GetNamespaceURI(&self) -> Option<DOMString>;
    fn GetPrefix(&self) -> Option<DOMString>;
}

impl<'a> AttrMethods for JSRef<'a, Attr> {
    fn LocalName(&self) -> DOMString {
        self.local_name.clone()
    }

    fn Value(&self) -> DOMString {
        self.value.deref().borrow().as_slice().to_string()
    }

    fn SetValue(&self, value: DOMString) {
        let owner = self.owner.get().root();
        let value = owner.deref().parse_attribute(
            &self.namespace, self.deref().local_name.as_slice(), value);
        self.set_value(ReplacedAttr, value);
    }

    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    fn GetNamespaceURI(&self) -> Option<DOMString> {
        match self.namespace.to_str() {
            "" => None,
            url => Some(url.to_string()),
        }
    }

    fn GetPrefix(&self) -> Option<DOMString> {
        self.prefix.clone()
    }
}

pub trait AttrHelpersForLayout {
    unsafe fn value_ref_forever(&self) -> &'static str;
}

impl AttrHelpersForLayout for Attr {
    unsafe fn value_ref_forever(&self) -> &'static str {
        // cast to point to T in RefCell<T> directly
        let value = mem::transmute::<&RefCell<AttrValue>, &AttrValue>(self.value.deref());
        value.as_slice()
    }
}
