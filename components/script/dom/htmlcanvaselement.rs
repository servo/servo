/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::{CanvasMsg, FromLayoutMsg, CanvasData};
use dom::attr::Attr;
use dom::attr::AttrValue;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::HTMLCanvasElementMethods;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLContextAttributes;
use dom::bindings::codegen::UnionTypes::CanvasRenderingContext2DOrWebGLRenderingContext;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{HeapGCValue, JS, LayoutJS, Root};
use dom::bindings::num::Finite;
use dom::bindings::str::DOMString;
use dom::canvasrenderingcontext2d::{CanvasRenderingContext2D, LayoutCanvasRenderingContext2DHelpers};
use dom::document::Document;
use dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::webglrenderingcontext::{LayoutCanvasWebGLRenderingContextHelpers, WebGLRenderingContext};
use euclid::size::Size2D;
use image::ColorType;
use image::png::PNGEncoder;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{HandleValue, JSContext};
use offscreen_gl_context::GLContextAttributes;
use rustc_serialize::base64::{STANDARD, ToBase64};
use std::iter::repeat;
use string_cache::Atom;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[must_root]
#[derive(JSTraceable, Clone, HeapSizeOf)]
pub enum CanvasContext {
    Context2d(JS<CanvasRenderingContext2D>),
    WebGL(JS<WebGLRenderingContext>),
}

impl HeapGCValue for CanvasContext {}

#[dom_struct]
pub struct HTMLCanvasElement {
    htmlelement: HTMLElement,
    context: DOMRefCell<Option<CanvasContext>>,
}

impl HTMLCanvasElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLCanvasElement {
        HTMLCanvasElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            context: DOMRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLCanvasElement> {
        let element = HTMLCanvasElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLCanvasElementBinding::Wrap)
    }

    fn recreate_contexts(&self) {
        let size = self.get_size();
        if let Some(ref context) = *self.context.borrow() {
            match *context {
                CanvasContext::Context2d(ref context) => context.set_bitmap_dimensions(size),
                CanvasContext::WebGL(ref context) => context.recreate(size),
            }
        }
    }

    pub fn get_size(&self) -> Size2D<i32> {
        Size2D::new(self.Width() as i32, self.Height() as i32)
    }

    pub fn origin_is_clean(&self) -> bool {
        match *self.context.borrow() {
            Some(CanvasContext::Context2d(ref context)) => context.origin_is_clean(),
            _ => true,
        }
    }
}

pub struct HTMLCanvasData {
    pub ipc_renderer: Option<IpcSender<CanvasMsg>>,
    pub width: u32,
    pub height: u32,
}

pub trait LayoutHTMLCanvasElementHelpers {
    fn data(&self) -> HTMLCanvasData;
}

impl LayoutHTMLCanvasElementHelpers for LayoutJS<HTMLCanvasElement> {
    #[allow(unsafe_code)]
    fn data(&self) -> HTMLCanvasData {
        unsafe {
            let canvas = &*self.unsafe_get();
            let ipc_renderer = canvas.context.borrow_for_layout().as_ref().map(|context| {
                match *context {
                    CanvasContext::Context2d(ref context) => {
                        context.to_layout().get_ipc_renderer()
                    },
                    CanvasContext::WebGL(ref context) => {
                        context.to_layout().get_ipc_renderer()
                    },
                }
            });

            let width_attr = canvas.upcast::<Element>().get_attr_for_layout(&ns!(), &atom!("width"));
            let height_attr = canvas.upcast::<Element>().get_attr_for_layout(&ns!(), &atom!("height"));
            HTMLCanvasData {
                ipc_renderer: ipc_renderer,
                width: width_attr.map_or(DEFAULT_WIDTH, |val| val.as_uint()),
                height: height_attr.map_or(DEFAULT_HEIGHT, |val| val.as_uint()),
            }
        }
    }
}


impl HTMLCanvasElement {
    pub fn ipc_renderer(&self) -> Option<IpcSender<CanvasMsg>> {
        self.context.borrow().as_ref().map(|context| {
            match *context {
                CanvasContext::Context2d(ref context) => context.ipc_renderer(),
                CanvasContext::WebGL(ref context) => context.ipc_renderer(),
            }
        })
    }

    pub fn get_or_init_2d_context(&self) -> Option<Root<CanvasRenderingContext2D>> {
        if self.context.borrow().is_none() {
            let window = window_from_node(self);
            let size = self.get_size();
            let context = CanvasRenderingContext2D::new(GlobalRef::Window(window.r()), self, size);
            *self.context.borrow_mut() = Some(CanvasContext::Context2d(JS::from_ref(&*context)));
        }

        match *self.context.borrow().as_ref().unwrap() {
            CanvasContext::Context2d(ref context) => Some(Root::from_ref(&*context)),
            _   => None,
        }
    }

    #[allow(unsafe_code)]
    pub fn get_or_init_webgl_context(&self,
                                 cx: *mut JSContext,
                                 attrs: Option<HandleValue>) -> Option<Root<WebGLRenderingContext>> {
        if self.context.borrow().is_none() {
            let window = window_from_node(self);
            let size = self.get_size();

            let attrs = if let Some(webgl_attributes) = attrs {
                if let Ok(ref attrs) = unsafe { WebGLContextAttributes::new(cx, webgl_attributes) } {
                    From::from(attrs)
                } else {
                    debug!("Unexpected error on conversion of WebGLContextAttributes");
                    return None;
                }
            } else {
                GLContextAttributes::default()
            };

            let maybe_ctx = WebGLRenderingContext::new(GlobalRef::Window(window.r()), self, size, attrs);

            *self.context.borrow_mut() = maybe_ctx.map( |ctx| CanvasContext::WebGL(JS::from_ref(&*ctx)));
        }

        if let Some(CanvasContext::WebGL(ref context)) = *self.context.borrow() {
            Some(Root::from_ref(&*context))
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        self.Height() != 0 && self.Width() != 0
    }

    pub fn fetch_all_data(&self) -> Option<(Vec<u8>, Size2D<i32>)> {
        let size = self.get_size();

        if size.width == 0 || size.height == 0 {
            return None
        }

        let data = if let Some(renderer) = self.ipc_renderer() {
            let (sender, receiver) = ipc::channel().unwrap();
            let msg = CanvasMsg::FromLayout(FromLayoutMsg::SendData(sender));
            renderer.send(msg).unwrap();

            match receiver.recv().unwrap() {
                CanvasData::Pixels(pixel_data)
                    => pixel_data.image_data.to_vec(),
                CanvasData::WebGL(_)
                    // TODO(emilio): Not sure if WebGL canvas is required for 2d spec,
                    // but I think it's not.
                    => return None,
            }
        } else {
            repeat(0xffu8).take((size.height as usize) * (size.width as usize) * 4).collect()
        };

        Some((data, size))
    }
}

impl HTMLCanvasElementMethods for HTMLCanvasElement {
    // https://html.spec.whatwg.org/multipage/#dom-canvas-width
    make_uint_getter!(Width, "width", DEFAULT_WIDTH);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-width
    make_uint_setter!(SetWidth, "width", DEFAULT_WIDTH);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-height
    make_uint_getter!(Height, "height", DEFAULT_HEIGHT);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-height
    make_uint_setter!(SetHeight, "height", DEFAULT_HEIGHT);

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-canvas-getcontext
    unsafe fn GetContext(&self,
                  cx: *mut JSContext,
                  id: DOMString,
                  attributes: Vec<HandleValue>)
        -> Option<CanvasRenderingContext2DOrWebGLRenderingContext> {
        match &*id {
            "2d" => {
                self.get_or_init_2d_context()
                    .map(CanvasRenderingContext2DOrWebGLRenderingContext::CanvasRenderingContext2D)
            }
            "webgl" | "experimental-webgl" => {
                self.get_or_init_webgl_context(cx, attributes.get(0).cloned())
                    .map(CanvasRenderingContext2DOrWebGLRenderingContext::WebGLRenderingContext)
            }
            _ => None
        }
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-canvas-todataurl
    unsafe fn ToDataURL(&self,
                 _context: *mut JSContext,
                 _mime_type: Option<DOMString>,
                 _arguments: Vec<HandleValue>) -> Fallible<DOMString> {
        // Step 1.
        if let Some(CanvasContext::Context2d(ref context)) = *self.context.borrow() {
            if !context.origin_is_clean() {
                return Err(Error::Security);
            }
        }

        // Step 2.
        if self.Width() == 0 || self.Height() == 0 {
            return Ok(DOMString::from("data:,"));
        }

        // Step 3.
        let raw_data = match *self.context.borrow() {
            Some(CanvasContext::Context2d(ref context)) => {
                let window = window_from_node(self);
                let image_data = try!(context.GetImageData(Finite::wrap(0f64), Finite::wrap(0f64),
                                                           Finite::wrap(self.Width() as f64),
                                                           Finite::wrap(self.Height() as f64)));
                image_data.get_data_array(&GlobalRef::Window(window.r()))
            }
            None => {
                // Each pixel is fully-transparent black.
                vec![0; (self.Width() * self.Height() * 4) as usize]
            }
            _ => return Err(Error::NotSupported) // WebGL
        };

        // Only handle image/png for now.
        let mime_type = "image/png";

        let mut encoded = Vec::new();
        {
            let encoder: PNGEncoder<&mut Vec<u8>> = PNGEncoder::new(&mut encoded);
            encoder.encode(&raw_data, self.Width(), self.Height(), ColorType::RGBA(8)).unwrap();
        }

        let encoded = encoded.to_base64(STANDARD);
        Ok(DOMString::from(format!("data:{};base64,{}", mime_type, encoded)))
    }
}

impl VirtualMethods for HTMLCanvasElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &atom!("width") | &atom!("height") => self.recreate_contexts(),
            _ => (),
        };
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("width") => AttrValue::from_u32(value.into(), DEFAULT_WIDTH),
            &atom!("height") => AttrValue::from_u32(value.into(), DEFAULT_HEIGHT),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

impl<'a> From<&'a WebGLContextAttributes> for GLContextAttributes {
    fn from(attrs: &'a WebGLContextAttributes) -> GLContextAttributes {
        GLContextAttributes {
            alpha: attrs.alpha,
            depth: attrs.depth,
            stencil: attrs.stencil,
            antialias: attrs.antialias,
            premultiplied_alpha: attrs.premultipliedAlpha,
            preserve_drawing_buffer: attrs.preserveDrawingBuffer,
        }
    }
}

pub mod utils {
    use dom::window::Window;
    use ipc_channel::ipc;
    use net_traits::image_cache_thread::{ImageCacheChan, ImageResponse};
    use url::Url;

    pub fn request_image_from_cache(window: &Window, url: Url) -> ImageResponse {
        let image_cache = window.image_cache_thread();
        let (response_chan, response_port) = ipc::channel().unwrap();
        image_cache.request_image(url, ImageCacheChan(response_chan), None);
        let result = response_port.recv().unwrap();
        result.image_response
    }
}
