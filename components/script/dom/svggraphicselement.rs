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
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct SVGGraphicsElement<TH: TypeHolderTrait> {
    svgelement: SVGElement<TH>,
}

impl<TH: TypeHolderTrait> SVGGraphicsElement<TH> {
    pub fn new_inherited(tag_name: LocalName, prefix: Option<Prefix>,
                         document: &Document<TH>) -> SVGGraphicsElement<TH> {
        SVGGraphicsElement::new_inherited_with_state(ElementState::empty(), tag_name, prefix, document)
    }

    pub fn new_inherited_with_state(state: ElementState, tag_name: LocalName,
                                    prefix: Option<Prefix>, document: &Document<TH>)
                                    -> SVGGraphicsElement<TH> {
        SVGGraphicsElement {
            svgelement:
                SVGElement::new_inherited_with_state(state, tag_name, prefix, document),
        }
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for SVGGraphicsElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<SVGElement<TH>>() as &VirtualMethods<TH>)
    }
}
