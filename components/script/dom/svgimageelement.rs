/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use style::attr::AttrValue;

use crate::dom::attr::Attr;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::AttributeMutation;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::svggraphicselement::SVGGraphicsElement;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

/// <https://svgwg.org/svg2-draft/embedded.html#Placement>
const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[dom_struct]
pub(crate) struct SVGImageElement {
    svggraphicselement: SVGGraphicsElement,
}

impl SVGImageElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGImageElement {
        SVGImageElement {
            svggraphicselement: SVGGraphicsElement::new_inherited(local_name, prefix, document),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<SVGImageElement> {
        Node::reflect_node_with_proto(
            Box::new(SVGImageElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }

    /// <https://svgwg.org/svg2-draft/linking.html#processingURL>
    fn fetch_image_resource(&self) {
        // TODO: Process and fetch the image resource (as HTMLImageElement).
        // Reject any resource fetching request immediately.
        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue_simple_event(self.upcast(), atom!("error"));
    }
}

impl VirtualMethods for SVGImageElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<SVGGraphicsElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);
        if attr.local_name() == &local_name!("href") &&
            matches!(attr.namespace(), &ns!() | &ns!(xlink))
        {
            if let AttributeMutation::Set(_) = mutation {
                self.fetch_image_resource();
            }
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("width") => AttrValue::from_u32(value.into(), DEFAULT_WIDTH),
            local_name!("height") => AttrValue::from_u32(value.into(), DEFAULT_HEIGHT),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }
}
