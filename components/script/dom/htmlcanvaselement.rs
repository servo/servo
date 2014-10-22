/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding;
use dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::HTMLCanvasElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLCanvasElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::global::Window;
use dom::bindings::js::{MutNullableJS, JSRef, Temporary, OptionalSettable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::canvasrenderingcontext2d::CanvasRenderingContext2D;
use dom::document::Document;
use dom::element::{Element, HTMLCanvasElementTypeId, AttributeHandlers};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;

use servo_util::str::{DOMString, parse_unsigned_integer};

use geom::size::Size2D;

use std::cell::Cell;
use std::default::Default;

static DefaultWidth: u32 = 300;
static DefaultHeight: u32 = 150;

#[dom_struct]
pub struct HTMLCanvasElement {
    htmlelement: HTMLElement,
    context: MutNullableJS<CanvasRenderingContext2D>,
    width: Cell<u32>,
    height: Cell<u32>,
}

impl HTMLCanvasElementDerived for EventTarget {
    fn is_htmlcanvaselement(&self) -> bool {
        *self.type_id() == NodeTargetTypeId(ElementNodeTypeId(HTMLCanvasElementTypeId))
    }
}

impl HTMLCanvasElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLCanvasElement {
        HTMLCanvasElement {
            htmlelement: HTMLElement::new_inherited(HTMLCanvasElementTypeId, localName, prefix, document),
            context: Default::default(),
            width: Cell::new(DefaultWidth),
            height: Cell::new(DefaultHeight),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLCanvasElement> {
        let element = HTMLCanvasElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLCanvasElementBinding::Wrap)
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

        if self.context.get().is_none() {
            let window = window_from_node(self).root();
            let (w, h) = (self.width.get() as i32, self.height.get() as i32);
            let context = CanvasRenderingContext2D::new(&Window(*window), self, Size2D(w, h));
            self.context.assign(Some(context));
        }
        self.context.get()
     }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLCanvasElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
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
                self.width.set(DefaultWidth);
                true
            }
            &atom!("height") => {
                self.height.set(DefaultHeight);
                true
            }
            _ => false,
        };

        if recreate {
            let (w, h) = (self.width.get() as i32, self.height.get() as i32);
            match self.context.get() {
                Some(ref context) => context.root().recreate(Size2D(w, h)),
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
                self.width.set(parse_unsigned_integer(value.as_slice().chars()).unwrap_or(DefaultWidth));
                true
            }
            &atom!("height") => {
                self.height.set(parse_unsigned_integer(value.as_slice().chars()).unwrap_or(DefaultHeight));
                true
            }
            _ => false,
        };

        if recreate {
            let (w, h) = (self.width.get() as i32, self.height.get() as i32);
            match self.context.get() {
                Some(ref context) => context.root().recreate(Size2D(w, h)),
                None => ()
            }
        }
    }
}

impl Reflectable for HTMLCanvasElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
