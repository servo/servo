/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::CanvasMsg;
use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::HTMLCanvasElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLCanvasElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::codegen::UnionTypes::CanvasRenderingContext2DOrWebGLRenderingContext;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLContextAttributes;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, LayoutJS, MutNullableHeap, HeapGCValue, Rootable};
use dom::bindings::js::Temporary;
use dom::bindings::js::Unrooted;
use dom::bindings::utils::{Reflectable};
use dom::canvasrenderingcontext2d::{CanvasRenderingContext2D, LayoutCanvasRenderingContext2DHelpers};
use dom::document::Document;
use dom::element::{Element, AttributeHandlers};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::webglrenderingcontext::{WebGLRenderingContext, LayoutCanvasWebGLRenderingContextHelpers};

use util::str::{DOMString, parse_unsigned_integer};
use js::jsapi::{JSContext};
use js::jsval::JSVal;
use offscreen_gl_context::GLContextAttributes;

use geom::size::Size2D;

use std::cell::Cell;
use std::default::Default;
use std::sync::mpsc::Sender;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[jstraceable]
#[must_root]
#[derive(Clone, Copy)]
pub enum CanvasContext {
    Context2d(JS<CanvasRenderingContext2D>),
    WebGL(JS<WebGLRenderingContext>),
}

impl HeapGCValue for CanvasContext {}

#[dom_struct]
pub struct HTMLCanvasElement {
    htmlelement: HTMLElement,
    context: MutNullableHeap<CanvasContext>,
    width: Cell<u32>,
    height: Cell<u32>,
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
                     document: JSRef<Document>) -> HTMLCanvasElement {
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
               document: JSRef<Document>) -> Temporary<HTMLCanvasElement> {
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
        Size2D(self.width.get() as i32, self.height.get() as i32)
    }
}

pub trait LayoutHTMLCanvasElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_renderer(&self) -> Option<Sender<CanvasMsg>>;
    #[allow(unsafe_code)]
    unsafe fn get_canvas_width(&self) -> u32;
    #[allow(unsafe_code)]
    unsafe fn get_canvas_height(&self) -> u32;
}

impl LayoutHTMLCanvasElementHelpers for LayoutJS<HTMLCanvasElement> {
    #[allow(unsafe_code)]
    unsafe fn get_renderer(&self) -> Option<Sender<CanvasMsg>> {
        let ref canvas = *self.unsafe_get();
        if let Some(context) = canvas.context.get() {
            match context {
                CanvasContext::Context2d(context)
                    => Some(context.to_layout().get_renderer()),
                CanvasContext::WebGL(context)
                    => Some(context.to_layout().get_renderer()),
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
    fn get_or_init_2d_context(self) -> Option<Temporary<CanvasRenderingContext2D>>;
    fn get_or_init_webgl_context(self,
                                 cx: *mut JSContext,
                                 attrs: Option<&JSVal>) -> Option<Temporary<WebGLRenderingContext>>;
    fn is_valid(self) -> bool;
}

impl<'a> HTMLCanvasElementHelpers for JSRef<'a, HTMLCanvasElement> {
    fn get_or_init_2d_context(self) -> Option<Temporary<CanvasRenderingContext2D>> {
        if self.context.get().is_none() {
            let window = window_from_node(self).root();
            let size = self.get_size();
            let context = CanvasRenderingContext2D::new(GlobalRef::Window(window.r()), self, size);
            self.context.set(Some(CanvasContext::Context2d(JS::from_rooted(context))));
        }

        match self.context.get().unwrap() {
            CanvasContext::Context2d(context) => Some(Temporary::from_rooted(context)),
            _   => None,
        }
    }

    fn get_or_init_webgl_context(self,
                                 cx: *mut JSContext,
                                 attrs: Option<&JSVal>) -> Option<Temporary<WebGLRenderingContext>> {
        if self.context.get().is_none() {
            let window = window_from_node(self).root();
            let size = self.get_size();

            let attrs = if let Some(webgl_attributes) = attrs {
                if let Ok(ref attrs) = WebGLContextAttributes::new(cx, *webgl_attributes) {
                    From::from(attrs)
                } else {
                    debug!("Unexpected error on conversion of WebGLContextAttributes");
                    return None;
                }
            } else {
                GLContextAttributes::default()
            };

            let maybe_ctx = WebGLRenderingContext::new(GlobalRef::Window(window.r()), self, size, attrs);

            self.context.set(maybe_ctx.map( |ctx| CanvasContext::WebGL(JS::from_rooted(ctx))));
        }

        if let Some(context) = self.context.get() {
            match context {
                CanvasContext::WebGL(context) => Some(Temporary::from_rooted(context)),
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

impl<'a> HTMLCanvasElementMethods for JSRef<'a, HTMLCanvasElement> {
    fn Width(self) -> u32 {
        self.width.get()
    }

    fn SetWidth(self, width: u32) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_uint_attribute(&atom!("width"), width)
    }

    fn Height(self) -> u32 {
        self.height.get()
    }

    fn SetHeight(self, height: u32) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_uint_attribute(&atom!("height"), height)
    }

    fn GetContext(self,
                  cx: *mut JSContext,
                  id: DOMString,
                  attributes: Vec<JSVal>)
        -> Option<CanvasRenderingContext2DOrWebGLRenderingContext> {
        match &*id {
            "2d" => {
                self.get_or_init_2d_context()
                    .map(|ctx| CanvasRenderingContext2DOrWebGLRenderingContext::eCanvasRenderingContext2D(
                                   Unrooted::from_temporary(ctx)))
            }
            "webgl" | "experimental-webgl" => {
                self.get_or_init_webgl_context(cx, attributes.get(0))
                    .map(|ctx| CanvasRenderingContext2DOrWebGLRenderingContext::eWebGLRenderingContext(
                                   Unrooted::from_temporary(ctx)))
            }
            _ => None
        }
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLCanvasElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let element: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(element as &VirtualMethods)
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
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

    fn after_set_attr(&self, attr: JSRef<Attr>) {
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

