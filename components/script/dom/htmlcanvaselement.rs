/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas::canvas_paint_task::CanvasMsg;
use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::HTMLCanvasElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLCanvasElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{MutNullableJS, JS, JSRef, Temporary};
use dom::canvasrenderingcontext2d::{CanvasRenderingContext2D, LayoutCanvasRenderingContext2DHelpers};
use dom::document::Document;
use dom::element::{Element, AttributeHandlers};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;

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
    context: MutNullableJS<CanvasRenderingContext2D>,
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
            context: Default::default(),
            width: Cell::new(DEFAULT_WIDTH),
            height: Cell::new(DEFAULT_HEIGHT),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLCanvasElement> {
        let element = HTMLCanvasElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLCanvasElementBinding::Wrap)
    }
}

pub trait LayoutHTMLCanvasElementHelpers {
    unsafe fn get_renderer(&self) -> Option<Sender<CanvasMsg>>;
    unsafe fn get_canvas_width(&self) -> u32;
    unsafe fn get_canvas_height(&self) -> u32;
}

impl LayoutHTMLCanvasElementHelpers for JS<HTMLCanvasElement> {
    unsafe fn get_renderer(&self) -> Option<Sender<CanvasMsg>> {
        let context = (*self.unsafe_get()).context.get_inner();
        context.map(|cx| cx.get_renderer())
    }

    unsafe fn get_canvas_width(&self) -> u32 {
        (*self.unsafe_get()).width.get()
    }

    unsafe fn get_canvas_height(&self) -> u32 {
        (*self.unsafe_get()).height.get()
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

    fn GetContext(self, id: DOMString) -> Option<Temporary<CanvasRenderingContext2D>> {
        if id.as_slice() != "2d" {
            return None;
        }

        Some(self.context.or_init(|| {
            let window = window_from_node(self).root();
            let (w, h) = (self.width.get() as i32, self.height.get() as i32);
            CanvasRenderingContext2D::new(GlobalRef::Window(window.r()), self, Size2D(w, h))
        }))
     }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLCanvasElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let element: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(element as &VirtualMethods)
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(attr),
            _ => ()
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
            let (w, h) = (self.width.get() as i32, self.height.get() as i32);
            match self.context.get() {
                Some(context) => context.root().r().recreate(Size2D(w, h)),
                None => ()
            }
        }
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
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
            let (w, h) = (self.width.get() as i32, self.height.get() as i32);
            match self.context.get() {
                Some(context) => context.root().r().recreate(Size2D(w, h)),
                None => ()
            }
        }
    }
}

