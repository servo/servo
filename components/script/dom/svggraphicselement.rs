/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::inheritance::Castable;
use dom::document::Document;
use dom::svgelement::SVGElement;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use style::element_state::ElementState;

#[dom_struct]
pub struct SVGGraphicsElement {
    svgelement: SVGElement,
}

impl SVGGraphicsElement {
    pub fn new_inherited(tag_name: LocalName, prefix: Option<Prefix>,
                         document: &Document) -> SVGGraphicsElement {
        SVGGraphicsElement::new_inherited_with_state(ElementState::empty(), tag_name, prefix, document)
    }

    pub fn new_inherited_with_state(state: ElementState, tag_name: LocalName,
                                    prefix: Option<Prefix>, document: &Document)
                                    -> SVGGraphicsElement {
        SVGGraphicsElement {
            svgelement:
                SVGElement::new_inherited_with_state(state, tag_name, prefix, document),
        }
    }
}

impl VirtualMethods for SVGGraphicsElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<SVGElement>() as &VirtualMethods)
    }
}
