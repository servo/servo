/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMStringMapBinding;
use dom::bindings::codegen::Bindings::DOMStringMapBinding::DOMStringMapMethods;
use dom::bindings::error::ErrorResult;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::htmlelement::HTMLElement;
use dom::node::window_from_node;
use util::str::DOMString;

#[dom_struct]
pub struct DOMStringMap {
    reflector_: Reflector,
    element: JS<HTMLElement>,
}

impl DOMStringMap {
    fn new_inherited(element: &HTMLElement) -> DOMStringMap {
        DOMStringMap {
            reflector_: Reflector::new(),
            element: JS::from_ref(element),
        }
    }

    pub fn new(element: &HTMLElement) -> Root<DOMStringMap> {
        let window = window_from_node(element);
        reflect_dom_object(box DOMStringMap::new_inherited(element),
                           GlobalRef::Window(window.r()), DOMStringMapBinding::Wrap)
    }
}

// https://html.spec.whatwg.org/#domstringmap
impl DOMStringMapMethods for DOMStringMap {
    // https://html.spec.whatwg.org/multipage/#dom-domstringmap-additem
    fn NamedCreator(&self, name: DOMString, value: DOMString) -> ErrorResult {
        self.NamedSetter(name, value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-domstringmap-removeitem
    fn NamedDeleter(&self, name: DOMString) {
        let element = self.element.root();
        element.r().delete_custom_attr(name)
    }

    // https://html.spec.whatwg.org/multipage/#dom-domstringmap-setitem
    fn NamedSetter(&self, name: DOMString, value: DOMString) -> ErrorResult {
        let element = self.element.root();
        element.r().set_custom_attr(name, value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-domstringmap-nameditem
    fn NamedGetter(&self, name: DOMString, found: &mut bool) -> DOMString {
        let element = self.element.root();
        match element.r().get_custom_attr(name) {
            Some(value) => {
                *found = true;
                value.clone()
            },
            None => {
                *found = false;
                String::new()
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#domstringmap
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        // FIXME: unimplemented (https://github.com/servo/servo/issues/7273)
        vec![]
    }
}
