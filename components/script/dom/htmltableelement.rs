/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::attr::{Attr, AttrValue};
use dom::bindings::codegen::Bindings::HTMLTableElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableElementBinding::HTMLTableElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, RootedReference};
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::htmltablecaptionelement::HTMLTableCaptionElement;
use dom::htmltablesectionelement::HTMLTableSectionElement;
use dom::node::{Node, document_from_node};
use dom::virtualmethods::VirtualMethods;
use std::cell::Cell;
use string_cache::Atom;
use util::str::{self, DOMString, LengthOrPercentageOrAuto};

#[dom_struct]
pub struct HTMLTableElement {
    htmlelement: HTMLElement,
    background_color: Cell<Option<RGBA>>,
    border: Cell<Option<u32>>,
    cellspacing: Cell<Option<u32>>,
    width: Cell<LengthOrPercentageOrAuto>,
}

impl HTMLTableElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document)
                     -> HTMLTableElement {
        HTMLTableElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
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

impl HTMLTableElementMethods for HTMLTableElement {
    // https://html.spec.whatwg.org/multipage/#dom-table-caption
    fn GetCaption(&self) -> Option<Root<HTMLTableCaptionElement>> {
        self.upcast::<Node>().children().filter_map(Root::downcast).next()
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-caption
    fn SetCaption(&self, new_caption: Option<&HTMLTableCaptionElement>) {
        if let Some(ref caption) = self.GetCaption() {
            caption.upcast::<Node>().remove_self();
        }

        if let Some(caption) = new_caption {
            let node = self.upcast::<Node>();
            node.InsertBefore(caption.upcast(), node.GetFirstChild().r())
                .expect("Insertion failed");
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createcaption
    fn CreateCaption(&self) -> Root<HTMLElement> {
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
        Root::upcast(caption)
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-deletecaption
    fn DeleteCaption(&self) {
        if let Some(caption) = self.GetCaption() {
            caption.upcast::<Node>().remove_self();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createtbody
    fn CreateTBody(&self) -> Root<HTMLTableSectionElement> {
        let tbody = HTMLTableSectionElement::new("tbody".to_owned(),
                                                 None,
                                                 document_from_node(self).r());
        let node = self.upcast::<Node>();
        let last_tbody =
            node.rev_children()
                .filter_map(Root::downcast::<Element>)
                .find(|n| n.is::<HTMLTableSectionElement>() && n.local_name() == &atom!("tbody"));
        let reference_element =
            last_tbody.and_then(|t| t.upcast::<Node>().GetNextSibling());

        node.InsertBefore(tbody.upcast(), reference_element.r())
            .expect("Insertion failed");
        tbody
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-bgcolor
    make_getter!(BgColor);

    // https://html.spec.whatwg.org/multipage/#dom-table-bgcolor
    make_setter!(SetBgColor, "bgcolor");
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
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match *attr.local_name() {
            atom!(bgcolor) => {
                self.background_color.set(mutation.new_value(attr).and_then(|value| {
                    str::parse_legacy_color(&value).ok()
                }));
            },
            atom!(border) => {
                // According to HTML5 § 14.3.9, invalid values map to 1px.
                self.border.set(mutation.new_value(attr).map(|value| {
                    str::parse_unsigned_integer(value.chars()).unwrap_or(1)
                }));
            }
            atom!(cellspacing) => {
                self.cellspacing.set(mutation.new_value(attr).and_then(|value| {
                    str::parse_unsigned_integer(value.chars())
                }));
            },
            atom!(width) => {
                let width = mutation.new_value(attr).map(|value| {
                    str::parse_length(&value)
                });
                self.width.set(width.unwrap_or(LengthOrPercentageOrAuto::Auto));
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, local_name: &Atom, value: DOMString) -> AttrValue {
        match *local_name {
            atom!("border") => AttrValue::from_u32(value, 1),
            _ => self.super_type().unwrap().parse_plain_attribute(local_name, value),
        }
    }
}
