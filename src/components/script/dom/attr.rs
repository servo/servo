/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::AttrBinding;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::element::Element;
use dom::node::Node;
use dom::window::Window;
use dom::virtualmethods::vtable_for;
use servo_util::namespace;
use servo_util::namespace::Namespace;
use servo_util::str::DOMString;

pub enum AttrSettingType {
    FirstSetAttr,
    ReplacedAttr,
}

#[deriving(Encodable)]
pub struct Attr {
    pub reflector_: Reflector,
    pub local_name: DOMString,
    pub value: DOMString,
    pub name: DOMString,
    pub namespace: Namespace,
    pub prefix: Option<DOMString>,

    /// the element that owns this attribute.
    pub owner: JS<Element>,
}

impl Reflectable for Attr {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}

impl Attr {
    fn new_inherited(local_name: DOMString, value: DOMString,
                     name: DOMString, namespace: Namespace,
                     prefix: Option<DOMString>, owner: JS<Element>) -> Attr {
        Attr {
            reflector_: Reflector::new(),
            local_name: local_name,
            value: value,
            name: name, //TODO: Intern attribute names
            namespace: namespace,
            prefix: prefix,
            owner: owner,
        }
    }

    pub fn new(window: &JS<Window>, local_name: DOMString, value: DOMString,
               name: DOMString, namespace: Namespace,
               prefix: Option<DOMString>, owner: JS<Element>) -> JS<Attr> {
        let attr = Attr::new_inherited(local_name, value, name, namespace, prefix, owner);
        reflect_dom_object(~attr, window, AttrBinding::Wrap)
    }

    pub fn set_value(&mut self, set_type: AttrSettingType, value: DOMString) {
        let node: JS<Node> = NodeCast::from(&self.owner);
        let namespace_is_null = self.namespace == namespace::Null;

        match set_type {
            ReplacedAttr => {
                if namespace_is_null {
                    vtable_for(&node).before_remove_attr(self.local_name.clone(), self.value.clone());
                }
            }
            FirstSetAttr => {}
        }

        self.value = value;

        if namespace_is_null {
            vtable_for(&node).after_set_attr(self.local_name.clone(), self.value.clone());
        }
    }

    pub fn value_ref<'a>(&'a self) -> &'a str {
        self.value.as_slice()
    }
}

impl Attr {
    pub fn LocalName(&self) -> DOMString {
        self.local_name.clone()
    }

    pub fn Value(&self) -> DOMString {
        self.value.clone()
    }

    pub fn SetValue(&mut self, value: DOMString) {
        self.set_value(ReplacedAttr, value);
    }

    pub fn Name(&self) -> DOMString {
        self.name.clone()
    }

    pub fn GetNamespaceURI(&self) -> Option<DOMString> {
        match self.namespace.to_str() {
            "" => None,
            url => Some(url.to_owned()),
        }
    }

    pub fn GetPrefix(&self) -> Option<DOMString> {
        self.prefix.clone()
    }
}
