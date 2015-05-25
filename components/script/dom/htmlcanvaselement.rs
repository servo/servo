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
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, LayoutJS, MutNullableHeap, Rootable};
use dom::bindings::js::Temporary;
use dom::bindings::js::Unrooted;
use dom::bindings::utils::{Reflectable, get_dictionary_property};
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
use js::jsapi::{JSContext, JSObject};
use js::jsval::JSVal;
use offscreen_gl_context::GLContextAttributes;

use geom::size::Size2D;

use std::cell::Cell;
use std::default::Default;
use std::sync::mpsc::Sender;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[dom_struct]
pub struct HTMLCanvasElement {
    htmlelement: HTMLElement,
    context_2d: MutNullableHeap<JS<CanvasRenderingContext2D>>,
    context_webgl: MutNullableHeap<JS<WebGLRenderingContext>>,
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
            context_2d: Default::default(),
            context_webgl: Default::default(),
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
        if let Some(context) = self.context_2d.get() {
            context.root().r().recreate(size)
        }
        if let Some(context) = self.context_webgl.get() {
            context.root().r().recreate(size)
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
        if canvas.context_2d.get().is_some() {
            let context = canvas.context_2d.get_inner_as_layout();
            context.map(|cx| cx.get_renderer())
        } else if canvas.context_webgl.get().is_some() {
            let context = canvas.context_webgl.get_inner_as_layout();
            context.map(|cx| cx.get_renderer())
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
    fn get_2d_context(self) -> Temporary<CanvasRenderingContext2D>;
    fn get_webgl_context(self) -> Temporary<WebGLRenderingContext>;
    fn is_valid(self) -> bool;
}

impl<'a> HTMLCanvasElementHelpers for JSRef<'a, HTMLCanvasElement> {
    fn get_2d_context(self) -> Temporary<CanvasRenderingContext2D> {
        let canvas_ref = self.extended_deref();

        canvas_ref.context_2d.get()
            .map(Unrooted::from_js)
            .map(Temporary::from_unrooted)
            .expect("Wrong Context Type: Expected 2d context")
    }

    fn get_webgl_context(self) -> Temporary<WebGLRenderingContext> {
        let canvas_ref = self.extended_deref();

        canvas_ref.context_webgl.get()
            .map(Unrooted::from_js)
            .map(Temporary::from_unrooted)
            .expect("Wrong Context Type: Expected WebGL context")
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
                  webgl_attributes: JSVal)
        -> Option<CanvasRenderingContext2DOrWebGLRenderingContext> {
        match &*id {
            "2d" => {
                if self.context_webgl.get().is_some() {
                    debug!("Trying to get a 2d context for a canvas with an already initialized WebGL context");
                    return None;
                }

                if !webgl_attributes.is_null() {
                    debug!("WebGL attributes found for a 2d context");
                    return None;
                }

                let context_2d = self.context_2d.or_init(|| {
                    let window = window_from_node(self).root();
                    let size = self.get_size();
                    CanvasRenderingContext2D::new(GlobalRef::Window(window.r()), self, size)
                });
                Some(
                    CanvasRenderingContext2DOrWebGLRenderingContext::eCanvasRenderingContext2D(
                        Unrooted::from_temporary(context_2d)))
            }
            "webgl" | "experimental-webgl" => {
                if self.context_2d.get().is_some() {
                    debug!("Trying to get a WebGL context for a canvas with an already initialized 2d context");
                    return None;
                }

                if !self.context_webgl.get().is_some() {
                    let window = window_from_node(self).root();
                    let size = self.get_size();

                    let attrs = if webgl_attributes.is_null() {
                        GLContextAttributes::default()
                    } else if webgl_attributes.is_object() {
                        GLContextAttributes::from_js_object(cx, webgl_attributes.to_object())
                    } else {
                        debug!("WebGL attributes should be an object or null");
                        return None;
                    };


                    self.context_webgl.set(
                        WebGLRenderingContext::new(GlobalRef::Window(window.r()), self, size, attrs)
                            .map(JS::from_rooted))
                }

                self.context_webgl.get().map( |ctx|
                    CanvasRenderingContext2DOrWebGLRenderingContext::eWebGLRenderingContext(Unrooted::from_js(ctx)))
            }
            _ => None
        }
    }
}

// TODO(ecoal95): put this in a common place for reuse?
trait FromJSObject {
    fn from_js_object(context: *mut JSContext, object: *mut JSObject) -> Self;
}

impl FromJSObject for GLContextAttributes {
    fn from_js_object(ctx: *mut JSContext, obj: *mut JSObject) -> GLContextAttributes {
        let mut attrs = GLContextAttributes::default();

        if obj.is_null() {
            return attrs;
        }

        if let Some(alpha) = get_dictionary_property(ctx, obj, "alpha").unwrap() {
            if alpha.is_boolean() {
                attrs.alpha = alpha.to_boolean();
            }
        }

        if let Some(depth) = get_dictionary_property(ctx, obj, "depth").unwrap() {
            if depth.is_boolean() {
                attrs.depth = depth.to_boolean();
            }
        }

        if let Some(stencil) = get_dictionary_property(ctx, obj, "stencil").unwrap() {
            if stencil.is_boolean() {
                attrs.stencil = stencil.to_boolean();
            }
        }

        if let Some(antialias) = get_dictionary_property(ctx, obj, "antialias").unwrap() {
            if antialias.is_boolean() {
                attrs.antialias = antialias.to_boolean();
            }
        }

        if let Some(premultiplied_alpha) = get_dictionary_property(ctx, obj, "premultipliedAlpha").unwrap() {
            if premultiplied_alpha.is_boolean() {
                attrs.premultiplied_alpha = premultiplied_alpha.to_boolean();
            }
        }

        // TODO(ecoal95): add support for preserve_drawing_buffer, and maybe other optional
        // arguments
        attrs
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
