/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use stylo_dom::ElementState;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::document::Document;
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::svg::svgelement::SVGElement;

#[dom_struct]
pub(crate) struct SVGGradientElement {
    svgelement: SVGElement,
}

impl SVGGradientElement {
    pub(crate) fn new_inherited(
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGGradientElement {
        SVGGradientElement::new_inherited_with_state(
            ElementState::empty(),
            tag_name,
            prefix,
            document,
        )
    }

    pub(crate) fn new_inherited_with_state(
        state: ElementState,
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGGradientElement {
        SVGGradientElement {
            svgelement: SVGElement::new_inherited_with_state(state, tag_name, prefix, document),
        }
    }
}

impl VirtualMethods for SVGGradientElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<SVGElement>() as &dyn VirtualMethods)
    }
}
