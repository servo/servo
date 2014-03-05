/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::AttrBinding;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
use servo_util::namespace::{Namespace, Null};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct Attr {
    reflector_: Reflector,
    local_name: DOMString,
    value: DOMString,
    name: DOMString,
    namespace: Namespace,
    prefix: Option<DOMString>
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
                     prefix: Option<DOMString>) -> Attr {
        Attr {
            reflector_: Reflector::new(),
            local_name: local_name,
            value: value,
            name: name, //TODO: Intern attribute names
            namespace: namespace,
            prefix: prefix
        }
    }

    pub fn new(window: &JS<Window>, local_name: DOMString, value: DOMString) -> JS<Attr> {
        let name = local_name.clone();
        Attr::new_helper(window, local_name, value, name, Null, None)
    }

    pub fn new_ns(window: &JS<Window>, local_name: DOMString, value: DOMString,
                  name: DOMString, namespace: Namespace,
                  prefix: Option<DOMString>) -> JS<Attr> {
        Attr::new_helper(window, local_name, value, name, namespace, prefix)
    }

    fn new_helper(window: &JS<Window>, local_name: DOMString, value: DOMString,
                  name: DOMString, namespace: Namespace,
                  prefix: Option<DOMString>) -> JS<Attr> {
        let attr = Attr::new_inherited(local_name, value, name, namespace, prefix);
        reflect_dom_object(~attr, window, AttrBinding::Wrap)
    }

    pub fn set_value(&mut self, value: DOMString) {
        self.value = value;
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
        self.value = value;
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
