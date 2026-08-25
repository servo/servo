/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::node::Node;
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::svg::svggeometryelement::SVGGeometryElement;

#[dom_struct]
pub(crate) struct SVGPathElement {
    svggeometryelement: SVGGeometryElement,
}

impl SVGPathElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGPathElement {
        SVGPathElement {
            svggeometryelement: SVGGeometryElement::new_inherited(local_name, prefix, document),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<SVGPathElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(SVGPathElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }
}

impl VirtualMethods for SVGPathElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<SVGGeometryElement>() as &dyn VirtualMethods)
    }
}
