/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::SVGSVGElementBinding;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{DomRoot, LayoutDom, Dom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use crate::dom::node::{Node, window_from_node};
use crate::dom::svggraphicselement::SVGGraphicsElement;
use crate::dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use script_layout_interface::SVGSVGData;
use style::attr::AttrValue;
use canvas_traits::webgl::{WebGLMsg, WebGLContextShareMode, WebGLMsgSender, webgl_channel, WebGLVersion, GLContextAttributes};
use std::cell::{Cell, RefCell, Ref};
use euclid::Size2D;
use crate::dom::window::Window;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::svgrenderingcontext::{
    SVGRenderingContext,
};
use std::borrow::Borrow;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[dom_struct]
pub struct SVGSVGElement {
    svggraphicselement: SVGGraphicsElement,
    context: DomRefCell<Option<Dom<SVGRenderingContext>>>,
}

impl SVGSVGElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGSVGElement {
        SVGSVGElement {
            svggraphicselement: SVGGraphicsElement::new_inherited(local_name, prefix, document),
            context: DomRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<SVGSVGElement> {
        Node::reflect_node(
            Box::new(SVGSVGElement::new_inherited(local_name, prefix, document)),
            document,
            SVGSVGElementBinding::Wrap,
        )
    }

    pub fn context(&self) -> Option<Ref<Dom<SVGRenderingContext>>>{
        ref_filter_map::ref_filter_map(self.context.borrow(), |ctx| ctx.as_ref())
    }

    pub fn get_or_init_svg_context(&self) -> Option<DomRoot<SVGRenderingContext>> {
        if let Some(ctx) = self.context() {
            return Some(DomRoot::from_ref(ctx.borrow()))
        }
        let window: DomRoot<Window> = window_from_node(self);
        let size = Size2D::new(DEFAULT_WIDTH,
                               DEFAULT_HEIGHT);
        let new_context = SVGRenderingContext::new(&*window, size, self)?;
        *self.context.borrow_mut() = Some(Dom::from_ref(&*new_context));
        Some(new_context)
    }
}

pub trait LayoutSVGSVGElementHelpers {
    fn data(&self) -> SVGSVGData;
}

impl LayoutSVGSVGElementHelpers for LayoutDom<SVGSVGElement> {
    #[allow(unsafe_code)]
    fn data(&self) -> SVGSVGData {
        unsafe {
            let SVG = &*self.unsafe_get();

            let ctx = SVG.get_or_init_svg_context();
            let width_attr = SVG
                .upcast::<Element>()
                .get_attr_for_layout(&ns!(), &local_name!("width"));
            let height_attr = SVG
                .upcast::<Element>()
                .get_attr_for_layout(&ns!(), &local_name!("height"));
            let width = width_attr.map_or(DEFAULT_WIDTH, |val| val.as_uint());
            let height = height_attr.map_or(DEFAULT_HEIGHT, |val| val.as_uint());

            match ctx {
                Some(ctx) => {
                    let image_key = ctx.extract_image_key();
                    SVGSVGData{
                        width,
                        height,
                        image_key: Some(image_key),
                    }
                },
                _ => {
                    SVGSVGData{
                        width,
                        height,
                        image_key: None
                    }
                }
            }
        }
    }
}

impl VirtualMethods for SVGSVGElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<SVGGraphicsElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("width") => AttrValue::from_u32(value.into(), DEFAULT_WIDTH),
            &local_name!("height") => AttrValue::from_u32(value.into(), DEFAULT_HEIGHT),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }
}
