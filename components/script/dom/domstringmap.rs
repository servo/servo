/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DOMStringMapBinding::DOMStringMapMethods;
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::NodeTraits;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct DOMStringMap {
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

    pub(crate) fn new(element: &HTMLElement, can_gc: CanGc) -> DomRoot<DOMStringMap> {
        reflect_dom_object(
            Box::new(DOMStringMap::new_inherited(element)),
            &*element.owner_window(),
            can_gc,
        )
    }
}

// https://html.spec.whatwg.org/multipage/#domstringmap
impl DOMStringMapMethods<crate::DomTypeHolder> for DOMStringMap {
    // https://html.spec.whatwg.org/multipage/#dom-domstringmap-removeitem
    fn NamedDeleter(&self, name: DOMString, can_gc: CanGc) {
        self.element.delete_custom_attr(name, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-domstringmap-setitem
    fn NamedSetter(&self, name: DOMString, value: DOMString, can_gc: CanGc) -> ErrorResult {
        self.element.set_custom_attr(name, value, can_gc)
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
