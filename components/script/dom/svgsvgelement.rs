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
use std::borrow::Borrow;
use crate::dom::webglcontextevent::WebGLContextEvent;
use crate::dom::event::{EventBubbles, EventCancelable, Event};

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[dom_struct]
pub struct SVGSVGElement {
    svggraphicselement: SVGGraphicsElement,
    #[ignore_malloc_size_of = "Channels are hard"]
    webgl_sender: Option<WebGLMsgSender>,
    share_mode: WebGLContextShareMode,
    #[ignore_malloc_size_of = "Defined in webrender"]
    webrender_image: Cell<Option<webrender_api::ImageKey>>
}

impl SVGSVGElement {
    #[allow(unrooted_must_root)]
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGSVGElement {
        let window = document.window();

        let webgl_chan = match window.webgl_chan() {
            Some(chan) => chan,
            None => panic!("Crash the system!"),
        };

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
        let size = Size2D::new(DEFAULT_WIDTH, DEFAULT_HEIGHT);
        webgl_chan
            .send(WebGLMsg::CreateContext(WebGLVersion::WebGL1, size, attrs, sender))
            .unwrap();
        let result = receiver.recv().unwrap();
        let other_prefix = prefix.clone();
        let other_local_name = local_name.clone();

        let mapped = result.map(|ctx_data| {
            let webgl_sender = ctx_data.sender;
            let share_mode = ctx_data.share_mode;
            SVGSVGElement{
                svggraphicselement: SVGGraphicsElement::new_inherited(other_local_name, prefix, document),
                webgl_sender: Some(webgl_sender),
                share_mode: share_mode,
                webrender_image: Cell::new(None),
            }
        });

        match mapped {
            Ok(elem) => elem,
            Err(msg) => {
                error!("Couldn't create SVGSVGElement with rendering information:{}", msg);
                SVGSVGElement{
                    svggraphicselement: SVGGraphicsElement::new_inherited(local_name, other_prefix, document),
                    webgl_sender: None,
                    share_mode: WebGLContextShareMode::SharedTexture,
                    webrender_image: Cell::new(None),
                }
            }
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

            let sender = &SVG.webgl_sender;
            let share_mode = SVG.share_mode;
            let webrender_image = &SVG.webrender_image;
            let width_attr = SVG
                .upcast::<Element>()
                .get_attr_for_layout(&ns!(), &local_name!("width"));
            let height_attr = SVG
                .upcast::<Element>()
                .get_attr_for_layout(&ns!(), &local_name!("height"));
            let width = width_attr.map_or(DEFAULT_WIDTH, |val| val.as_uint());
            let height = height_attr.map_or(DEFAULT_HEIGHT, |val| val.as_uint());

            match sender {
                Some(webgl_sender) => {
                    let image_key = match share_mode {
                        WebGLContextShareMode::SharedTexture => {
                            webrender_image.get().unwrap_or_else(|| {
                                let (sender, receiver) = webgl_channel().unwrap();
                                webgl_sender.send_update_wr_image(sender).unwrap();
                                let key = receiver.recv().unwrap();
                                SVG.webrender_image.set(Some(key));
                                key
                            })
                        },
                        WebGLContextShareMode::Readback => {
                            // WR using Readback requires to update WR image every frame
                            // in order to send the new raw pixels.
                            let (sender, receiver) = webgl_channel().unwrap();
                            webgl_sender.send_update_wr_image(sender).unwrap();
                            receiver.recv().unwrap()
                        },
                    };
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
