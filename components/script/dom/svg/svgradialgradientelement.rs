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
use crate::dom::svg::svggradientelement::SVGGradientElement;

#[dom_struct]
pub(crate) struct SVGRadialGradientElement {
    svggradientelement: SVGGradientElement,
}

impl SVGRadialGradientElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGRadialGradientElement {
        SVGRadialGradientElement {
            svggradientelement: SVGGradientElement::new_inherited(local_name, prefix, document),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<SVGRadialGradientElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(SVGRadialGradientElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }
}

impl VirtualMethods for SVGRadialGradientElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<SVGGradientElement>() as &dyn VirtualMethods)
    }
}
