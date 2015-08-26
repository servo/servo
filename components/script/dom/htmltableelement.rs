/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrValue};
use dom::bindings::codegen::Bindings::HTMLTableElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableElementBinding::HTMLTableElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::HTMLTableSectionElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, HTMLTableCaptionElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLTableElementDerived, NodeCast};
use dom::bindings::js::{Root, RootedReference};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmltablecaptionelement::HTMLTableCaptionElement;
use dom::htmltablesectionelement::HTMLTableSectionElement;
use dom::node::{Node, NodeTypeId, document_from_node};
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
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableElement)))
    }
}

impl HTMLTableElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document)
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
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: &Document)
               -> Root<HTMLTableElement> {
        let element = HTMLTableElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTableElementBinding::Wrap)
    }
}

impl<'a> HTMLTableElementMethods for &'a HTMLTableElement {
    // https://html.spec.whatwg.org/multipage/#dom-table-caption
    fn GetCaption(self) -> Option<Root<HTMLTableCaptionElement>> {
        let node = NodeCast::from_ref(self);
        node.children()
            .filter_map(|c| {
                HTMLTableCaptionElementCast::to_ref(c.r()).map(Root::from_ref)
            })
            .next()
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-caption
    fn SetCaption(self, new_caption: Option<&HTMLTableCaptionElement>) {
        let node = NodeCast::from_ref(self);

        if let Some(ref caption) = self.GetCaption() {
            NodeCast::from_ref(caption.r()).remove_self();
        }

        if let Some(caption) = new_caption {
            assert!(node.InsertBefore(NodeCast::from_ref(caption),
                                      node.GetFirstChild().as_ref().map(|n| n.r())).is_ok());
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createcaption
    fn CreateCaption(self) -> Root<HTMLElement> {
        let caption = match self.GetCaption() {
            Some(caption) => caption,
            None => {
                let caption = HTMLTableCaptionElement::new("caption".to_owned(),
                                                           None,
                                                           document_from_node(self).r());
                self.SetCaption(Some(caption.r()));
                caption
            }
        };
        HTMLElementCast::from_root(caption)
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-deletecaption
    fn DeleteCaption(self) {
        if let Some(caption) = self.GetCaption() {
            NodeCast::from_ref(caption.r()).remove_self();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createtbody
    fn CreateTBody(self) -> Root<HTMLTableSectionElement> {
        let tbody = HTMLTableSectionElement::new("tbody".to_owned(),
                                                 None,
                                                 document_from_node(self).r());
        let node = NodeCast::from_ref(self);
        let last_tbody =
            node.rev_children()
                .filter_map(ElementCast::to_root)
                .find(|n| n.is_htmltablesectionelement() && n.local_name() == &atom!("tbody"));
        let reference_element =
            last_tbody.and_then(|t| NodeCast::from_root(t).GetNextSibling());

        assert!(node.InsertBefore(NodeCast::from_ref(tbody.r()),
                                  reference_element.r()).is_ok());
        tbody
    }
}


impl HTMLTableElement {
    pub fn get_background_color(&self) -> Option<RGBA> {
        self.background_color.get()
    }

    pub fn get_border(&self) -> Option<u32> {
        self.border.get()
    }

    pub fn get_cellspacing(&self) -> Option<u32> {
        self.cellspacing.get()
    }

    pub fn get_width(&self) -> LengthOrPercentageOrAuto {
        self.width.get()
    }
}

impl VirtualMethods for HTMLTableElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: &Attr) {
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

    fn before_remove_attr(&self, attr: &Attr) {
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
