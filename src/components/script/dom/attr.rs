/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::AttrBinding;
use dom::bindings::utils::{Reflectable, Reflector, DOMString};
use dom::bindings::utils::reflect_dom_object;
use dom::namespace::{Namespace, Null};
use dom::window::Window;

use std::str::eq_slice;

pub struct Attr {
    reflector_: Reflector,
    priv local_name: Option<~str>,
    value: ~str,
    name: ~str,
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
    fn new_inherited(name: ~str, value: ~str, local_name: Option<~str>,
                         namespace: Namespace, prefix: Option<~str>) -> Attr {
        Attr {
            reflector_: Reflector::new(),
            local_name: local_name,
            value: value,
            name: name, //TODO: Intern attribute names
            namespace: namespace,
            prefix: prefix
        }
    }

    pub fn new(window: &Window, name: ~str, value: ~str) -> @mut Attr {
        Attr::new_helper(window, name, value, None, Null, None)
    }

    pub fn new_ns(window: &Window, name: ~str, value: ~str, local_name: ~str, namespace: Namespace,
                  prefix: Option<~str>) -> @mut Attr {
        let local_name = if eq_slice(local_name, name) {
            None
        } else {
            Some(local_name)
        };
        Attr::new_helper(window, name, value, local_name, namespace, prefix)
    }

    fn new_helper(window: &Window, name: ~str, value: ~str, local_name: Option<~str>,
                  namespace: Namespace, prefix: Option<~str>) -> @mut Attr {
        let attr = Attr::new_inherited(name, value, local_name, namespace, prefix);
        reflect_dom_object(@mut attr, window, AttrBinding::Wrap)
    }

    pub fn local_name<'a>(&'a self) -> &'a str {
        match self.local_name {
            Some(ref x) => x.as_slice(),
            None => self.name.as_slice()
        }
    }

    pub fn LocalName(&self) -> DOMString {
        self.local_name().to_owned()
    }

    pub fn Value(&self) -> DOMString {
        self.value.clone()
    }

    pub fn SetValue(&mut self, value: &DOMString) {
        self.value = value.clone();
    }

    pub fn Name(&self) -> DOMString {
        self.name.clone()
    }

    pub fn GetNamespaceURI(&self) -> Option<DOMString> {
        self.namespace.to_str()
    }

    pub fn GetPrefix(&self) -> Option<DOMString> {
        self.prefix.clone()
    }
}
