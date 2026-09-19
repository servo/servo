/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};

use crate::dom::bindings::inheritance::Castable;
use crate::dom::document::Document;
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::svg::svgtextcontentelement::SVGTextContentElement;

#[dom_struct]
pub(crate) struct SVGTextPositioningElement {
    svgtextcontentelement: SVGTextContentElement,
}

impl SVGTextPositioningElement {
    pub(crate) fn new_inherited(
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGTextPositioningElement {
        SVGTextPositioningElement {
            svgtextcontentelement: SVGTextContentElement::new_inherited(tag_name, prefix, document),
        }
    }
}

impl VirtualMethods for SVGTextPositioningElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<SVGTextContentElement>() as &dyn VirtualMethods)
    }
}
