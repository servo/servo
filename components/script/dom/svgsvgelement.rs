/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::SVGSVGElementBinding;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use crate::dom::node::Node;
use crate::dom::svggraphicselement::SVGGraphicsElement;
use crate::dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use script_layout_interface::SVGSVGData;
use style::attr::AttrValue;
use canvas_traits::webgl::{WebGLMsg, WebGLContextShareMode, WebGLMsgSender, webgl_channel, WebGLVersion, GLContextAttributes};
use std::cell::{Cell, RefCell};
use euclid::Size2D;
use crate::dom::window::Window;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

struct SVGRenderingContext{
    webgl_sender: WebGLMsgSender,
    share_mode: WebGLContextShareMode,
    webrender_image: Cell<Option<webrender_api::ImageKey>>,
}

impl SVGRenderingContext{
    fn extract_image_key(&self) -> webrender_api::ImageKey{
        match self.share_mode {
            WebGLContextShareMode::SharedTexture => {
                // WR using ExternalTexture requires a single update message.
                self.webrender_image.get().unwrap_or_else(|| {
                    let (sender, receiver) = webgl_channel().unwrap();
                    self.webgl_sender.send_update_wr_image(sender).unwrap();
                    let image_key = receiver.recv().unwrap();
                    self.webrender_image.set(Some(image_key));

                    image_key
                })
            },
            WebGLContextShareMode::Readback => {
                // WR using Readback requires to update WR image every frame
                // in order to send the new raw pixels.
                let (sender, receiver) = webgl_channel().unwrap();
                self.webgl_sender.send_update_wr_image(sender).unwrap();
                receiver.recv().unwrap()
            },
        }
    }
}

#[dom_struct]
pub struct SVGSVGElement {
    svggraphicselement: SVGGraphicsElement,
}

impl SVGSVGElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGSVGElement {
        SVGSVGElement {
            svggraphicselement: SVGGraphicsElement::new_inherited(local_name, prefix, document),
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
}

pub trait LayoutSVGSVGElementHelpers {
    fn data(&self) -> SVGSVGData;
}

impl LayoutSVGSVGElementHelpers for LayoutDom<SVGSVGElement> {
    #[allow(unsafe_code)]
    fn data(&self) -> SVGSVGData {
        unsafe {
            let SVG = &*self.unsafe_get();

            let width_attr = SVG
                .upcast::<Element>()
                .get_attr_for_layout(&ns!(), &local_name!("width"));
            let height_attr = SVG
                .upcast::<Element>()
                .get_attr_for_layout(&ns!(), &local_name!("height"));
            let width = width_attr.map_or(DEFAULT_WIDTH, |val| val.as_uint());
            let height = height_attr.map_or(DEFAULT_HEIGHT, |val| val.as_uint());
            let window = SVG.global().as_window();

            let webgl_chan = match window.webgl_chan() {
                Some(chan) => chan,
                None => panic!("Crash the system!"),
            };
            let size = Size2D::new(width, height);

            let (sender, receiver) = webgl_channel()
                .unwrap();
            let attrs = GLContextAttributes{
                depth: false,
                stencil: false,
                alpha: true,
                antialias: true,
                premultiplied_alpha: true,
                preserve_drawing_buffer: false,
            };
            webgl_chan
                .send(WebGLMsg::CreateContext(WebGLVersion::WebGL1, size, attrs, sender))
                .unwrap();
            let result = receiver.recv().unwrap();

            let mapped = result.map(|ctx_data| {
                let webgl_sender = ctx_data.sender;
                let share_mode = ctx_data.share_mode;
                SVGRenderingContext{
                    webgl_sender,
                    share_mode,
                    webrender_image: Cell::new(None),
                }
            });

            match mapped {
                Ok(ctx) => {
                    let image_key = ctx.extract_image_key();
                    SVGSVGData{
                        width,
                        height,
                        image_key: Some(image_key),
                    }
                },
                Err(msg) => {
                    error!("Couldn't create SVGRenderingContext:{}",msg);
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
