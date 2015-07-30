/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers, AttrValue};
use dom::bindings::codegen::Bindings::HTMLTableElementBinding::HTMLTableElementMethods;
use dom::bindings::codegen::Bindings::HTMLTableElementBinding;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTableCaptionElementCast};
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLTableSectionElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLTableElementDerived, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLTableCaptionElementDerived, HTMLTableColElementDerived};
use dom::bindings::codegen::InheritTypes::HTMLTableSectionElementDerived;
use dom::bindings::error::ErrorResult;
use dom::bindings::error::Error::HierarchyRequest;
use dom::bindings::js::{JS, Root, MutNullableHeap};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::{Element, ElementHelpers, ElementTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmltablecaptionelement::HTMLTableCaptionElement;
use dom::htmltablesectionelement::HTMLTableSectionElement;
use dom::node::{Node, NodeHelpers, NodeTypeId, SuppressObserver};
use dom::node::{document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;

use util::str::{self, DOMString, LengthOrPercentageOrAuto};

use cssparser::RGBA;
use string_cache::Atom;

use std::cell::Cell;

#[derive(JSTraceable)]
struct TBodiesFilter;
impl CollectionFilter for TBodiesFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is_htmltablesectionelement() && elem.local_name() == &atom!("tbody")
    }
}

#[dom_struct]
pub struct HTMLTableElement {
    htmlelement: HTMLElement,
    background_color: Cell<Option<RGBA>>,
    border: Cell<Option<u32>>,
    cellspacing: Cell<Option<u32>>,
    tbodies: MutNullableHeap<JS<HTMLCollection>>,
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
            tbodies: Default::default(),
            width: Cell::new(LengthOrPercentageOrAuto::Auto),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: &Document)
               -> Root<HTMLTableElement> {
        let element = HTMLTableElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTableElementBinding::Wrap)
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-thead
    // https://html.spec.whatwg.org/multipage/#dom-table-tfoot
    fn get_first_section_of_type(&self, atom: &Atom) -> Option<Root<HTMLTableSectionElement>> {
        let node = NodeCast::from_ref(self);
        node.child_elements()
            .find(|n| n.is_htmltablesectionelement() && n.r().local_name() == atom)
            .and_then(HTMLTableSectionElementCast::to_root)
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-thead
    // https://html.spec.whatwg.org/multipage/#dom-table-tfoot
    fn set_first_section_of_type<P>(&self,
                                    atom: &Atom,
                                    section: Option<&HTMLTableSectionElement>,
                                    reference_predicate: P)
                                        -> ErrorResult where P: FnMut(&Root<Element>) -> bool {
        match section {
            Some(e) if ElementCast::from_ref(e).local_name() != atom =>
                Err(HierarchyRequest),
            _ => {
                if let Some(first_section) = self.get_first_section_of_type(atom) {
                    NodeCast::from_ref(first_section.r()).remove_self()
                }

                let node = NodeCast::from_ref(self);

                if let Some(section) = section {
                    let reference_element =
                        node.child_elements().find(reference_predicate);

                    Node::insert(NodeCast::from_ref(section),
                                 node,
                                 reference_element.as_ref().map(|n| NodeCast::from_ref(n.r())),
                                 SuppressObserver::Unsuppressed);
                }

                Ok(())
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createthead
    // https://html.spec.whatwg.org/multipage/#dom-table-createtfoot
    fn create_section_of_type(&self, atom: &Atom) -> Root<HTMLElement> {
        let section = match self.get_first_section_of_type(atom) {
            Some(section) => section,
            None => {
                let section = HTMLTableSectionElement::new(atom.to_string(),
                                                           None,
                                                           document_from_node(self).r());
                assert!(match atom {
                    &atom!("thead") => self.SetTHead(Some(section.r())),
                    &atom!("tfoot") => self.SetTFoot(Some(section.r())),
                    _ => unreachable!("unexpected section type")
                }.is_ok());
                section
            }
        };
        HTMLElementCast::from_root(section)
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-deletethead
    // https://html.spec.whatwg.org/multipage/#dom-table-deletetfoot
    fn delete_first_section_of_type(&self, atom: &Atom) {
        if let Some(thead) = self.get_first_section_of_type(atom) {
            NodeCast::from_ref(thead.r()).remove_self();
        }
    }
}

impl<'a> HTMLTableElementMethods for &'a HTMLTableElement {
    // https://html.spec.whatwg.org/multipage/#dom-table-caption
    fn GetCaption(self) -> Option<Root<HTMLTableCaptionElement>> {
        let node = NodeCast::from_ref(self);
        node.children()
            .filter_map(HTMLTableCaptionElementCast::to_root)
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

    // https://html.spec.whatwg.org/multipage/#dom-table-thead
    fn GetTHead(self) -> Option<Root<HTMLTableSectionElement>> {
        self.get_first_section_of_type(&atom!("thead"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-thead
    fn SetTHead(self, thead: Option<&HTMLTableSectionElement>) -> ErrorResult {
        self.set_first_section_of_type(&atom!("thead"),
                                       thead,
                                       |n| !(n.is_htmltablecaptionelement() ||
                                             n.is_htmltablecolelement()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createthead
    fn CreateTHead(self) -> Root<HTMLElement> {
        self.create_section_of_type(&atom!("thead"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-deletethead
    fn DeleteTHead(self) {
        self.delete_first_section_of_type(&atom!("thead"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-tfoot
    fn GetTFoot(self) -> Option<Root<HTMLTableSectionElement>> {
        self.get_first_section_of_type(&atom!("tfoot"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-tfoot
    fn SetTFoot(self, tfoot: Option<&HTMLTableSectionElement>) -> ErrorResult {
        self.set_first_section_of_type(&atom!("tfoot"),
                                       tfoot,
                                       |n| !(n.is_htmltablecaptionelement() ||
                                             n.is_htmltablecolelement() ||
                                             (n.is_htmltablesectionelement() &&
                                                 n.r().local_name() == &atom!("thead"))))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createtfoot
    fn CreateTFoot(self) -> Root<HTMLElement> {
        self.create_section_of_type(&atom!("tfoot"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-deletetfoot
    fn DeleteTFoot(self) {
        self.delete_first_section_of_type(&atom!("tfoot"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-tbodies
    fn TBodies(self) -> Root<HTMLCollection> {
        self.tbodies.or_init(|| {
            let window = window_from_node(self);
            let filter = box TBodiesFilter;
            HTMLCollection::create(window.r(), NodeCast::from_ref(self), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createtbody
    fn CreateTBody(self) -> Root<HTMLElement> {
        let tbody = HTMLTableSectionElement::new("tbody".to_owned(),
                                                 None,
                                                 document_from_node(self).r());
        let node = NodeCast::from_ref(self);
        let reference_element =
            node.child_elements().find(|n| n.is_htmltablesectionelement() &&
                                           n.local_name() == &atom!("tbody"));

        Node::insert(NodeCast::from_ref(tbody.r()),
                     node,
                     reference_element.as_ref().map(|n| NodeCast::from_ref(n.r())),
                     SuppressObserver::Unsuppressed);
        HTMLElementCast::from_root(tbody)
    }
}

pub trait HTMLTableElementHelpers {
    fn get_background_color(self) -> Option<RGBA>;
    fn get_border(self) -> Option<u32>;
    fn get_cellspacing(self) -> Option<u32>;
    fn get_width(self) -> LengthOrPercentageOrAuto;
}

impl<'a> HTMLTableElementHelpers for &'a HTMLTableElement {
    fn get_background_color(self) -> Option<RGBA> {
        self.background_color.get()
    }

    fn get_border(self) -> Option<u32> {
        self.border.get()
    }

    fn get_cellspacing(self) -> Option<u32> {
        self.cellspacing.get()
    }

    fn get_width(self) -> LengthOrPercentageOrAuto {
        self.width.get()
    }
}

impl<'a> VirtualMethods for &'a HTMLTableElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &&HTMLElement = HTMLElementCast::from_borrowed_ref(self);
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

