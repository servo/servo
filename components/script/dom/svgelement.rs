/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::inheritance::Castable;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::Element;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::LocalName;
use style::element_state::ElementState;

#[dom_struct]
pub struct SVGElement {
    element: Element,
}

impl SVGElement {
    pub fn new_inherited_with_state(state: ElementState, tag_name: LocalName,
                                    prefix: Option<DOMString>, document: &Document)
                                    -> SVGElement {
        SVGElement {
            element:
                Element::new_inherited_with_state(state, tag_name, ns!(svg), prefix, document),
        }
    }
}

impl VirtualMethods for SVGElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<Element>() as &VirtualMethods)
    }
}
