/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::HTMLCanvasElementMethods;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLContextAttributes;
use dom::bindings::codegen::InheritTypes::HTMLCanvasElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::codegen::UnionTypes::CanvasRenderingContext2DOrWebGLRenderingContext;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap, HeapGCValue, Root};
use dom::bindings::utils::{Reflectable};
use dom::canvasrenderingcontext2d::{CanvasRenderingContext2D, LayoutCanvasRenderingContext2DHelpers};
use dom::document::Document;
use dom::element::AttributeHandlers;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::webglrenderingcontext::{WebGLRenderingContext, LayoutCanvasWebGLRenderingContextHelpers};

use canvas_traits::CanvasMsg;
use ipc_channel::ipc::IpcSender;
use js::jsapi::{JSContext, HandleValue};
use offscreen_gl_context::GLContextAttributes;
use util::str::{DOMString, parse_unsigned_integer};

use euclid::size::Size2D;

use std::cell::Cell;
use std::default::Default;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[must_root]
#[derive(JSTraceable, Clone, Copy, HeapSizeOf)]
pub enum CanvasContext {
    Context2d(JS<CanvasRenderingContext2D>),
    WebGL(JS<WebGLRenderingContext>),
}

impl HeapGCValue for CanvasContext {}

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLCanvasElement {
    htmlelement: HTMLElement,
    context: MutNullableHeap<CanvasContext>,
    width: Cell<u32>,
    height: Cell<u32>,
}

impl PartialEq for HTMLCanvasElement {
    fn eq(&self, other: &HTMLCanvasElement) -> bool {
        self as *const HTMLCanvasElement == &*other
    }
}

impl HTMLCanvasElementDerived for EventTarget {
    fn is_htmlcanvaselement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement)))
    }
}

impl HTMLCanvasElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLCanvasElement {
        HTMLCanvasElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLCanvasElement, localName, prefix, document),
            context: Default::default(),
            width: Cell::new(DEFAULT_WIDTH),
            height: Cell::new(DEFAULT_HEIGHT),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLCanvasElement> {
        let element = HTMLCanvasElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLCanvasElementBinding::Wrap)
    }

    fn recreate_contexts(&self) {
        let size = self.get_size();
        if let Some(context) = self.context.get() {
            match context {
                CanvasContext::Context2d(context) => context.root().r().recreate(size),
                CanvasContext::WebGL(context) => context.root().r().recreate(size),
            }
        }
    }

    pub fn get_size(&self) -> Size2D<i32> {
        Size2D::new(self.width.get() as i32, self.height.get() as i32)
    }
}

pub trait LayoutHTMLCanvasElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_renderer_id(&self) -> Option<usize>;
    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> Option<IpcSender<CanvasMsg>>;
    #[allow(unsafe_code)]
    unsafe fn get_canvas_width(&self) -> u32;
    #[allow(unsafe_code)]
    unsafe fn get_canvas_height(&self) -> u32;
}

impl LayoutHTMLCanvasElementHelpers for LayoutJS<HTMLCanvasElement> {
    #[allow(unsafe_code)]
    unsafe fn get_renderer_id(&self) -> Option<usize> {
        let ref canvas = *self.unsafe_get();
        if let Some(context) = canvas.context.get() {
            match context {
                CanvasContext::Context2d(context) => Some(context.to_layout().get_renderer_id()),
                CanvasContext::WebGL(context) => Some(context.to_layout().get_renderer_id()),
            }
        } else {
            None
        }
    }

    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> Option<IpcSender<CanvasMsg>> {
        let ref canvas = *self.unsafe_get();
        if let Some(context) = canvas.context.get() {
            match context {
                CanvasContext::Context2d(context) => Some(context.to_layout().get_ipc_renderer()),
                CanvasContext::WebGL(context) => Some(context.to_layout().get_ipc_renderer()),
            }
        } else {
            None
        }
    }

    #[allow(unsafe_code)]
    unsafe fn get_canvas_width(&self) -> u32 {
        (*self.unsafe_get()).width.get()
    }

    #[allow(unsafe_code)]
    unsafe fn get_canvas_height(&self) -> u32 {
        (*self.unsafe_get()).height.get()
    }
}

pub trait HTMLCanvasElementHelpers {
    fn get_or_init_2d_context(self) -> Option<Root<CanvasRenderingContext2D>>;
    fn get_or_init_webgl_context(self,
                                 cx: *mut JSContext,
                                 attrs: Option<HandleValue>) -> Option<Root<WebGLRenderingContext>>;

    fn is_valid(self) -> bool;
}

impl<'a> HTMLCanvasElementHelpers for &'a HTMLCanvasElement {
    fn get_or_init_2d_context(self) -> Option<Root<CanvasRenderingContext2D>> {
        if self.context.get().is_none() {
            let window = window_from_node(self);
            let size = self.get_size();
            let context = CanvasRenderingContext2D::new(GlobalRef::Window(window.r()), self, size);
            self.context.set(Some(CanvasContext::Context2d(JS::from_rooted(&context))));
        }

        match self.context.get().unwrap() {
            CanvasContext::Context2d(context) => Some(context.root()),
            _   => None,
        }
    }

    fn get_or_init_webgl_context(self,
                                 cx: *mut JSContext,
                                 attrs: Option<HandleValue>) -> Option<Root<WebGLRenderingContext>> {
        if self.context.get().is_none() {
            let window = window_from_node(self);
            let size = self.get_size();

            let attrs = if let Some(webgl_attributes) = attrs {
                if let Ok(ref attrs) = WebGLContextAttributes::new(cx, webgl_attributes) {
                    From::from(attrs)
                } else {
                    debug!("Unexpected error on conversion of WebGLContextAttributes");
                    return None;
                }
            } else {
                GLContextAttributes::default()
            };

            let maybe_ctx = WebGLRenderingContext::new(GlobalRef::Window(window.r()), self, size, attrs);

            self.context.set(maybe_ctx.map( |ctx| CanvasContext::WebGL(JS::from_rooted(&ctx))));
        }

        if let Some(context) = self.context.get() {
            match context {
                CanvasContext::WebGL(context) => Some(context.root()),
                _ => None,
            }
        } else {
            None
        }
    }

    fn is_valid(self) -> bool {
        self.height.get() != 0 && self.width.get() != 0
    }
}

impl<'a> HTMLCanvasElementMethods for &'a HTMLCanvasElement {
    // https://html.spec.whatwg.org/multipage/#dom-canvas-width
    fn Width(self) -> u32 {
        self.width.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-canvas-width
    fn SetWidth(self, width: u32) {
        let elem = ElementCast::from_ref(self);
        elem.set_uint_attribute(&atom!("width"), width)
    }

    // https://html.spec.whatwg.org/multipage/#dom-canvas-height
    fn Height(self) -> u32 {
        self.height.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-canvas-height
    fn SetHeight(self, height: u32) {
        let elem = ElementCast::from_ref(self);
        elem.set_uint_attribute(&atom!("height"), height)
    }

    // https://html.spec.whatwg.org/multipage/#dom-canvas-getcontext
    fn GetContext(self,
                  cx: *mut JSContext,
                  id: DOMString,
                  attributes: Vec<HandleValue>)
        -> Option<CanvasRenderingContext2DOrWebGLRenderingContext> {
        match &*id {
            "2d" => {
                self.get_or_init_2d_context()
                    .map(|ctx| CanvasRenderingContext2DOrWebGLRenderingContext::eCanvasRenderingContext2D(
                                   ctx))
            }
            "webgl" | "experimental-webgl" => {
                self.get_or_init_webgl_context(cx, attributes.get(0).map(|p| *p))
                    .map(|ctx| CanvasRenderingContext2DOrWebGLRenderingContext::eWebGLRenderingContext(
                                   ctx))
            }
            _ => None
        }
    }
}

impl VirtualMethods for HTMLCanvasElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let element: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(element as &VirtualMethods)
    }

    fn before_remove_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        let recreate = match attr.local_name() {
            &atom!("width") => {
                self.width.set(DEFAULT_WIDTH);
                true
            }
            &atom!("height") => {
                self.height.set(DEFAULT_HEIGHT);
                true
            }
            _ => false,
        };

        if recreate {
           self.recreate_contexts();
        }
    }

    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        let value = attr.value();
        let recreate = match attr.local_name() {
            &atom!("width") => {
                self.width.set(parse_unsigned_integer(value.chars()).unwrap_or(DEFAULT_WIDTH));
                true
            }
            &atom!("height") => {
                self.height.set(parse_unsigned_integer(value.chars()).unwrap_or(DEFAULT_HEIGHT));
                true
            }
            _ => false,
        };

        if recreate {
            self.recreate_contexts();
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
    use net_traits::image_cache_task::{ImageCacheChan, ImageResponse};
    use url::Url;

    pub fn request_image_from_cache(window: &Window, url: Url) -> ImageResponse {
        let image_cache = window.image_cache_task();
        let (response_chan, response_port) = ipc::channel().unwrap();
        image_cache.request_image(url, ImageCacheChan(response_chan), None);
        let result = response_port.recv().unwrap();
        result.image_response
    }
}

