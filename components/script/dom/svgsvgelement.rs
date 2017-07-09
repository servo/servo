/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::SVGSVGElementBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{LayoutJS, Root};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, UnbindContext, window_from_node};
use dom::svggraphicselement::SVGGraphicsElement;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use script_layout_interface::SVGImageData;
use std::cell::Cell;
use style::attr::AttrValue;
use webrender_api::GeometryKey;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[dom_struct]
pub struct SVGSVGElement {
    svggraphicselement: SVGGraphicsElement,
    geometry_key: Cell<Option<GeometryKey>>,
}

impl SVGSVGElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document)
                     -> SVGSVGElement {
        SVGSVGElement {
            svggraphicselement: SVGGraphicsElement::new_inherited(local_name, prefix, document),
            geometry_key: Cell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document)
               -> Root<SVGSVGElement> {
        Node::reflect_node(box SVGSVGElement::new_inherited(local_name, prefix, document),
                           document,
                           SVGSVGElementBinding::Wrap)
    }
}

pub trait LayoutSVGSVGElementHelpers {
    fn data(&self) -> Option<SVGImageData>;
}

impl LayoutSVGSVGElementHelpers for LayoutJS<SVGSVGElement> {
    #[allow(unsafe_code)]
    fn data(&self) -> Option<SVGImageData> {
        unsafe {
            let element = &*self.unsafe_get();

            let width_attr = element
                .upcast::<Element>()
                .get_attr_for_layout(&ns!(), &local_name!("width"));
            let height_attr = element
                .upcast::<Element>()
                .get_attr_for_layout(&ns!(), &local_name!("height"));

            element.geometry_key.get().map(|key| {
                SVGImageData {
                    width: width_attr.map_or(DEFAULT_WIDTH, |val| val.as_uint()),
                    height: height_attr.map_or(DEFAULT_HEIGHT, |val| val.as_uint()),
                    geometry_key: key
                }
            })
        }
    }
}

impl VirtualMethods for SVGSVGElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<SVGGraphicsElement>() as &VirtualMethods)
    }

    // When the <svg> element is appended to a tree and its parent is an html element,
    // ensure a geometry key for it.
    fn bind_to_tree(&self, tree_in_doc: bool) {
        if self.upcast::<Node>().GetParentNode().unwrap().is::<HTMLElement>() {
                if self.geometry_key.get().is_none() {
                    let window = window_from_node(self);
                    let key = window.image_cache().create_geometry_key();
                    self.geometry_key.set(Some(key));
                }
            }
        self.super_type().unwrap().bind_to_tree(tree_in_doc);
    }

    // When the <svg> element is removed from the tree, delete the corresponding
    // geometry key if it exists.
    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(key) = self.geometry_key.get() {
            let window = window_from_node(self);
            window.image_cache().delete_geometry(key);
            self.geometry_key.set(None);
        }
        self.super_type().unwrap().unbind_from_tree(context);
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("width") => AttrValue::from_u32(value.into(), DEFAULT_WIDTH),
            &local_name!("height") => AttrValue::from_u32(value.into(), DEFAULT_HEIGHT),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}
