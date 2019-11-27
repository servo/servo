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
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use crate::dom::node::{Node, window_from_node, document_from_node};
use crate::dom::svggraphicselement::SVGGraphicsElement;
use crate::dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use script_layout_interface::SVGSVGData;
use style::attr::AttrValue;
use canvas_traits::webgl::{WebGLMsg, WebGLContextShareMode, WebGLMsgSender, webgl_channel, WebGLVersion, GLContextAttributes, WebGLCommand};
use std::cell::{Cell, RefCell, Ref};
use euclid::Size2D;
use crate::dom::window::Window;
use crate::dom::bindings::cell::DomRefCell;
use std::borrow::{Borrow, BorrowMut};
use crate::dom::webglcontextevent::WebGLContextEvent;
use crate::dom::event::{EventBubbles, EventCancelable, Event};
use crate::dom::webglrenderingcontext::capture_webgl_backtrace;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementBinding::ElementMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::node::ChildrenMutation;
use crate::dom::node::NodeDamage;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[dom_struct]
pub struct SVGSVGElement {
    svggraphicselement: SVGGraphicsElement,
    #[ignore_malloc_size_of = "Channels are hard"]
    webgl_sender: Option<WebGLMsgSender>,
    #[ignore_malloc_size_of = "Just a string"]
    html_string: DomRefCell<Option<String>>,
    #[ignore_malloc_size_of = "Defined in webrender"]
    webrender_image: Option<webrender_api::ImageKey>
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
            .send(WebGLMsg::CreateContext(WebGLVersion::WebGL1, size, attrs, sender, true))
            .unwrap();
        let result = receiver.recv().unwrap();
        let other_prefix = prefix.clone();
        let other_local_name = local_name.clone();

        let mapped = result.map(|ctx_data| {
            let webgl_sender = ctx_data.sender;
            SVGSVGElement{
                svggraphicselement: SVGGraphicsElement::new_inherited(other_local_name, prefix, document),
                webgl_sender: Some(webgl_sender),
                html_string: DomRefCell::new(None),
                webrender_image: Some(ctx_data.image_key)
            }
        });

        match mapped {
            Ok(elem) => elem,
            Err(msg) => {
                error!("Couldn't create SVGSVGElement with rendering information:{}", msg);
                SVGSVGElement{
                    svggraphicselement: SVGGraphicsElement::new_inherited(local_name, other_prefix, document),
                    webgl_sender: None,
                    webrender_image: None,
                    html_string: DomRefCell::new(None)
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
            let webrender_image = &SVG.webrender_image;
            let html_string_option = SVG.html_string.borrow_for_layout().as_ref();
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
                    let (resize_sender, resize_receiver) = webgl_channel().unwrap();
                    webgl_sender.send_resize(Size2D::new(width, height), resize_sender).unwrap();
                    if let Err(msg) = resize_receiver.recv().unwrap(){
                        panic!("PANIC: Error resizing rendering context for SVG: {}", msg);
                    }

                    SVGSVGData{
                        width,
                        height,
                        image_key: *webrender_image,
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

    fn children_changed(&self, mutation: &ChildrenMutation){
        let htmlString = self.upcast::<Element>().GetOuterHTML();
        match htmlString {
            Ok(domString) => {
                let svg_string = domString.to_string();
                let cloned_svg_string = svg_string.clone();
                let webgl_sender = &self.webgl_sender;
                *self.html_string.borrow_mut() = Some(svg_string);
                if let Some(webgl_sender) = webgl_sender {
                    webgl_sender.send_rebuild_svg(cloned_svg_string).unwrap();
                    document_from_node(self).add_dirty_canvas(self.webgl_sender.as_ref().context_id());
                }
            },
            _ => {}
        }
    }
}
