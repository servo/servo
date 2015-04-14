/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas::canvas_msg::CanvasMsg;
use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::HTMLCanvasElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLCanvasElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::codegen::UnionTypes::CanvasRenderingContext2DOrWebGLRenderingContext;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{MutNullableJS, JSRef, LayoutJS, Temporary, Unrooted};
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

use geom::size::Size2D;

use std::cell::Cell;
use std::default::Default;
use std::sync::mpsc::Sender;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[dom_struct]
pub struct HTMLCanvasElement {
    htmlelement: HTMLElement,
    context_2d: MutNullableJS<CanvasRenderingContext2D>,
    context_webgl: MutNullableJS<WebGLRenderingContext>,
    width: Cell<u32>,
    height: Cell<u32>,
}

impl HTMLCanvasElementDerived for EventTarget {
    fn is_htmlcanvaselement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement)))
    }
}

impl HTMLCanvasElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLCanvasElement {
        HTMLCanvasElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLCanvasElement, localName, prefix, document),
            context_2d: Default::default(),
            context_webgl: Default::default(),
            width: Cell::new(DEFAULT_WIDTH),
            height: Cell::new(DEFAULT_HEIGHT),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLCanvasElement> {
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
        let context = self.GetContext(String::from_str("2d"));
        match context.unwrap() {
            CanvasRenderingContext2DOrWebGLRenderingContext::eCanvasRenderingContext2D(context) => {
              Temporary::new(context.root().r().unrooted())
            }
            _ => panic!("Wrong Context Type: Expected 2d context"),
        }
    }

    fn get_webgl_context(self) -> Temporary<WebGLRenderingContext> {
        let context = self.GetContext(String::from_str("webgl"));
        match context.unwrap() {
            CanvasRenderingContext2DOrWebGLRenderingContext::eWebGLRenderingContext(context) => {
              return Temporary::new(context.root().r().unrooted());
            }
            _ => panic!("Wrong Context Type: Expected webgl context"),
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

    fn GetContext(self, id: DOMString) -> Option<CanvasRenderingContext2DOrWebGLRenderingContext> {
        match id.as_slice() {
           "2d" => {
               let context_2d = self.context_2d.or_init(|| {
                   let window = window_from_node(self).root();
                   let size = self.get_size();
                   CanvasRenderingContext2D::new(GlobalRef::Window(window.r()), self, size)
               });
               Some(CanvasRenderingContext2DOrWebGLRenderingContext::eCanvasRenderingContext2D(Unrooted::from_temporary(context_2d)))
           }
           "webgl" | "experimental-webgl" => {
               let context_webgl = self.context_webgl.or_init(|| {
                   let window = window_from_node(self).root();
                   let size = self.get_size();
                   WebGLRenderingContext::new(GlobalRef::Window(window.r()), self, size)
               });
               Some(CanvasRenderingContext2DOrWebGLRenderingContext::eWebGLRenderingContext(Unrooted::from_temporary(context_webgl)))
           }
           _ => return None
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
                self.width.set(parse_unsigned_integer(value.as_slice().chars()).unwrap_or(DEFAULT_WIDTH));
                true
            }
            &atom!("height") => {
                self.height.set(parse_unsigned_integer(value.as_slice().chars()).unwrap_or(DEFAULT_HEIGHT));
                true
            }
            _ => false,
        };

        if recreate {
            self.recreate_contexts();
        }
    }
}
