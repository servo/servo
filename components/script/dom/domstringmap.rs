/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DOMStringMapBinding::DOMStringMapMethods;
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::window_from_node;

#[dom_struct]
pub struct DOMStringMap {
    reflector_: Reflector,
    element: Dom<HTMLElement>,
}

impl DOMStringMap {
    fn new_inherited(element: &HTMLElement) -> DOMStringMap {
        DOMStringMap {
            reflector_: Reflector::new(),
            element: Dom::from_ref(element),
        }
    }

    pub fn new(element: &HTMLElement) -> DomRoot<DOMStringMap> {
        let window = window_from_node(element);
        reflect_dom_object(Box::new(DOMStringMap::new_inherited(element)), &*window)
    }
}

// https://html.spec.whatwg.org/multipage/#domstringmap
impl DOMStringMapMethods for DOMStringMap {
    // https://html.spec.whatwg.org/multipage/#dom-domstringmap-removeitem
    fn NamedDeleter(&self, name: DOMString) {
        self.element.delete_custom_attr(name)
    }

    // https://html.spec.whatwg.org/multipage/#dom-domstringmap-setitem
    fn NamedSetter(&self, name: DOMString, value: DOMString) -> ErrorResult {
        self.element.set_custom_attr(name, value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-domstringmap-nameditem
    fn NamedGetter(&self, name: DOMString) -> Option<DOMString> {
        self.element.get_custom_attr(name)
    }

    // https://html.spec.whatwg.org/multipage/#the-domstringmap-interface:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.element.supported_prop_names_custom_attr().to_vec()
    }
}
