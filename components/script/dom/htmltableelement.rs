/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers, AttrValue};
use dom::bindings::codegen::Bindings::HTMLTableElementBinding::HTMLTableElementMethods;
use dom::bindings::codegen::Bindings::HTMLTableElementBinding;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTableCaptionElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLTableElementDerived, NodeCast};
use dom::bindings::js::{JSRef, Rootable, Temporary};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmltablecaptionelement::HTMLTableCaptionElement;
use dom::node::{Node, NodeHelpers, NodeTypeId};
use dom::virtualmethods::VirtualMethods;

use util::str::{self, DOMString, LengthOrPercentageOrAuto};

use cssparser::RGBA;
use string_cache::Atom;

use std::cell::Cell;

#[dom_struct]
pub struct HTMLTableElement {
    htmlelement: HTMLElement,
    background_color: Cell<Option<RGBA>>,
    border: Cell<Option<u32>>,
    cellspacing: Cell<Option<u32>>,
    width: Cell<LengthOrPercentageOrAuto>,
}

impl HTMLTableElementDerived for EventTarget {
    fn is_htmltableelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableElement)))
    }
}

impl HTMLTableElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
                     -> HTMLTableElement {
        HTMLTableElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLTableElement,
                                                    localName,
                                                    prefix,
                                                    document),
            background_color: Cell::new(None),
            border: Cell::new(None),
            cellspacing: Cell::new(None),
            width: Cell::new(LengthOrPercentageOrAuto::Auto),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
               -> Temporary<HTMLTableElement> {
        let element = HTMLTableElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTableElementBinding::Wrap)
    }
}

impl<'a> HTMLTableElementMethods for JSRef<'a, HTMLTableElement> {
    //  https://www.whatwg.org/html/#dom-table-caption
    fn GetCaption(self) -> Option<Temporary<HTMLTableCaptionElement>> {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.children()
            .map(|c| c.root())
            .filter_map(|c| {
                HTMLTableCaptionElementCast::to_ref(c.r()).map(Temporary::from_rooted)
            })
            .next()
    }

    // https://www.whatwg.org/html/#dom-table-caption
    fn SetCaption(self, new_caption: Option<JSRef<HTMLTableCaptionElement>>) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let old_caption = self.GetCaption();

        match old_caption {
            Some(htmlelem) => {
                let htmlelem_root = htmlelem.root();
                let old_caption_node: JSRef<Node> = NodeCast::from_ref(htmlelem_root.r());
                assert!(node.RemoveChild(old_caption_node).is_ok());
            }
            None => ()
        }

        new_caption.map(|caption| {
            let new_caption_node: JSRef<Node> = NodeCast::from_ref(caption);
            assert!(node.AppendChild(new_caption_node).is_ok());
        });
    }
}

pub trait HTMLTableElementHelpers {
    fn get_background_color(&self) -> Option<RGBA>;
    fn get_border(&self) -> Option<u32>;
    fn get_cellspacing(&self) -> Option<u32>;
    fn get_width(&self) -> LengthOrPercentageOrAuto;
}

impl HTMLTableElementHelpers for HTMLTableElement {
    fn get_background_color(&self) -> Option<RGBA> {
        self.background_color.get()
    }

    fn get_border(&self) -> Option<u32> {
        self.border.get()
    }

    fn get_cellspacing(&self) -> Option<u32> {
        self.cellspacing.get()
    }

    fn get_width(&self) -> LengthOrPercentageOrAuto {
        self.width.get()
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLTableElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("bgcolor") => {
                self.background_color.set(str::parse_legacy_color(&attr.value()).ok())
            }
            &atom!("border") => {
                // According to HTML5 § 14.3.9, invalid values map to 1px.
                self.border.set(Some(str::parse_unsigned_integer(attr.value()
                                                                     .chars()).unwrap_or(1)))
            }
            &atom!("cellspacing") => {
                self.cellspacing.set(str::parse_unsigned_integer(attr.value().chars()))
            }
            &atom!("width") => self.width.set(str::parse_length(&attr.value())),
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        match attr.local_name() {
            &atom!("bgcolor") => self.background_color.set(None),
            &atom!("border") => self.border.set(None),
            &atom!("cellspacing") => self.cellspacing.set(None),
            &atom!("width") => self.width.set(LengthOrPercentageOrAuto::Auto),
            _ => ()
        }
    }

    fn parse_plain_attribute(&self, local_name: &Atom, value: DOMString) -> AttrValue {
        match local_name {
            &atom!("border") => AttrValue::from_u32(value, 1),
            _ => self.super_type().unwrap().parse_plain_attribute(local_name, value),
        }
    }
}

