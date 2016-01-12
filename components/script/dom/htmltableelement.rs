/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::attr::{Attr, AttrValue};
use dom::bindings::codegen::Bindings::HTMLTableElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableElementBinding::HTMLTableElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap, Root, RootedReference};
use dom::document::Document;
use dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::HTMLElement;
use dom::htmltablecaptionelement::HTMLTableCaptionElement;
use dom::htmltablecolelement::HTMLTableColElement;
use dom::htmltablesectionelement::HTMLTableSectionElement;
use dom::node::{Node, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use std::cell::Cell;
use string_cache::Atom;
use util::str::{self, DOMString, LengthOrPercentageOrAuto};

#[derive(JSTraceable)]
struct TBodiesFilter;
impl CollectionFilter for TBodiesFilter {
    fn filter(&self, elem: &Element, root: &Node) -> bool {
        elem.is::<HTMLTableSectionElement>()
            && elem.local_name() == &atom!("tbody")
            && elem.upcast::<Node>().GetParentNode().r() == Some(root)
    }
}

#[dom_struct]
pub struct HTMLTableElement {
    htmlelement: HTMLElement,
    border: Cell<Option<u32>>,
    cellspacing: Cell<Option<u32>>,
    tbodies: MutNullableHeap<JS<HTMLCollection>>,
}

impl HTMLTableElement {
    fn new_inherited(localName: Atom, prefix: Option<DOMString>, document: &Document)
                     -> HTMLTableElement {
        HTMLTableElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            border: Cell::new(None),
            cellspacing: Cell::new(None),
            tbodies: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom, prefix: Option<DOMString>, document: &Document)
               -> Root<HTMLTableElement> {
        let element = HTMLTableElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTableElementBinding::Wrap)
    }

    pub fn get_border(&self) -> Option<u32> {
        self.border.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-thead
    // https://html.spec.whatwg.org/multipage/#dom-table-tfoot
    fn get_first_section_of_type(&self, atom: &Atom) -> Option<Root<HTMLTableSectionElement>> {
        self.upcast::<Node>()
            .child_elements()
            .find(|n| n.is::<HTMLTableSectionElement>() && n.local_name() == atom)
            .and_then(|n| n.downcast().map(Root::from_ref))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-thead
    // https://html.spec.whatwg.org/multipage/#dom-table-tfoot
    fn set_first_section_of_type<P>(&self,
                                    atom: &Atom,
                                    section: Option<&HTMLTableSectionElement>,
                                    reference_predicate: P)
                                        -> ErrorResult where P: FnMut(&Root<Element>) -> bool {
        match section {
            Some(e) if e.upcast::<Element>().local_name() != atom =>
                Err(Error::HierarchyRequest),
            _ => {
                if let Some(first_section) = self.get_first_section_of_type(atom) {
                    first_section.upcast::<Node>().remove_self()
                }

                let node = self.upcast::<Node>();

                if let Some(section) = section {
                    let reference_element =
                        node.child_elements()
                            .find(reference_predicate);
                    let reference_element = reference_element.r();
                    let reference_node = reference_element.map(|e| e.upcast());

                    assert!(node.InsertBefore(section.upcast(),
                                              reference_node).is_ok());
                }

                Ok(())
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createthead
    // https://html.spec.whatwg.org/multipage/#dom-table-createtfoot
    fn create_section_of_type(&self, atom: &Atom) -> Root<HTMLTableSectionElement> {
        match self.get_first_section_of_type(atom) {
            Some(section) => section,
            None => {
                let section = HTMLTableSectionElement::new(DOMString::from(atom.to_string()),
                                                           None,
                                                           document_from_node(self).r());
                assert!(match atom {
                    &atom!("thead") => self.SetTHead(Some(section.r())),
                    &atom!("tfoot") => self.SetTFoot(Some(section.r())),
                    _ => unreachable!("unexpected section type")
                }.is_ok());
                section
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-deletethead
    // https://html.spec.whatwg.org/multipage/#dom-table-deletetfoot
    fn delete_first_section_of_type(&self, atom: &Atom) {
        if let Some(thead) = self.get_first_section_of_type(atom) {
            thead.upcast::<Node>().remove_self();
        }
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
                let caption = HTMLTableCaptionElement::new(atom!("caption"),
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

    // https://html.spec.whatwg.org/multipage/#dom-table-thead
    fn GetTHead(&self) -> Option<Root<HTMLTableSectionElement>> {
        self.get_first_section_of_type(&atom!("thead"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-thead
    fn SetTHead(&self, thead: Option<&HTMLTableSectionElement>) -> ErrorResult {
        self.set_first_section_of_type(&atom!("thead"),
                                       thead,
                                       |n| !(n.is::<HTMLTableCaptionElement>() ||
                                             n.is::<HTMLTableColElement>()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createthead
    fn CreateTHead(&self) -> Root<HTMLTableSectionElement> {
        self.create_section_of_type(&atom!("thead"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-deletethead
    fn DeleteTHead(&self) {
        self.delete_first_section_of_type(&atom!("thead"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-tfoot
    fn GetTFoot(&self) -> Option<Root<HTMLTableSectionElement>> {
        self.get_first_section_of_type(&atom!("tfoot"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-tfoot
    fn SetTFoot(&self, tfoot: Option<&HTMLTableSectionElement>) -> ErrorResult {
        self.set_first_section_of_type(&atom!("tfoot"),
                                       tfoot,
                                       |n| !(n.is::<HTMLTableCaptionElement>() ||
                                             n.is::<HTMLTableColElement>() ||
                                             (n.is::<HTMLTableSectionElement>() &&
                                                 n.local_name() == &atom!("thead"))))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createtfoot
    fn CreateTFoot(&self) -> Root<HTMLTableSectionElement> {
        self.create_section_of_type(&atom!("tfoot"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-deletetfoot
    fn DeleteTFoot(&self) {
        self.delete_first_section_of_type(&atom!("tfoot"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-tbodies
    fn TBodies(&self) -> Root<HTMLCollection> {
        self.tbodies.or_init(|| {
            let window = window_from_node(self);
            let filter = box TBodiesFilter;
            HTMLCollection::create(window.r(), self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-table-createtbody
    fn CreateTBody(&self) -> Root<HTMLTableSectionElement> {
        let tbody = HTMLTableSectionElement::new(atom!("tbody"),
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
    make_getter!(BgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-table-bgcolor
    make_legacy_color_setter!(SetBgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-table-width
    make_getter!(Width, "width");

    // https://html.spec.whatwg.org/multipage/#dom-table-width
    make_nonzero_dimension_setter!(SetWidth, "width");
}

pub trait HTMLTableElementLayoutHelpers {
    fn get_background_color(&self) -> Option<RGBA>;
    fn get_border(&self) -> Option<u32>;
    fn get_cellspacing(&self) -> Option<u32>;
    fn get_width(&self) -> LengthOrPercentageOrAuto;
}

impl HTMLTableElementLayoutHelpers for LayoutJS<HTMLTableElement> {
    #[allow(unsafe_code)]
    fn get_background_color(&self) -> Option<RGBA> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("bgcolor"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }

    #[allow(unsafe_code)]
    fn get_border(&self) -> Option<u32> {
        unsafe {
            (*self.unsafe_get()).border.get()
        }
    }

    #[allow(unsafe_code)]
    fn get_cellspacing(&self) -> Option<u32> {
        unsafe {
            (*self.unsafe_get()).cellspacing.get()
        }
    }

    #[allow(unsafe_code)]
    fn get_width(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("width"))
                .map(AttrValue::as_dimension)
                .cloned()
                .unwrap_or(LengthOrPercentageOrAuto::Auto)
        }
    }
}

impl VirtualMethods for HTMLTableElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match *attr.local_name() {
            atom!("border") => {
                // According to HTML5 § 14.3.9, invalid values map to 1px.
                self.border.set(mutation.new_value(attr).map(|value| {
                    str::parse_unsigned_integer(value.chars()).unwrap_or(1)
                }));
            }
            atom!("cellspacing") => {
                self.cellspacing.set(mutation.new_value(attr).and_then(|value| {
                    str::parse_unsigned_integer(value.chars())
                }));
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, local_name: &Atom, value: DOMString) -> AttrValue {
        match *local_name {
            atom!("border") => AttrValue::from_u32(value, 1),
            atom!("width") => AttrValue::from_nonzero_dimension(value),
            atom!("bgcolor") => AttrValue::from_legacy_color(value),
            _ => self.super_type().unwrap().parse_plain_attribute(local_name, value),
        }
    }
}
