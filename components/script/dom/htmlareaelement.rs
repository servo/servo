/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;
use std::{f32, str};

use cssparser::match_ignore_ascii_case;
use dom_struct::dom_struct;
use euclid::default::Point2D;
use html5ever::{LocalName, Prefix, local_name};
use js::rust::HandleObject;
use servo_url::ServoUrl;
use style::attr::AttrValue;
use stylo_atoms::Atom;

use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLAreaElementBinding::HTMLAreaElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::document::Document;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::element::{AttributeMutation, Element, reflect_referrer_policy_attribute};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlhyperlinkelementutils::{HyperlinkElement, HyperlinkElementTraits};
use crate::dom::node::{BindContext, Node};
use crate::dom::virtualmethods::VirtualMethods;
use crate::links::{LinkRelations, follow_hyperlink};
use crate::script_runtime::CanGc;

#[derive(Debug, PartialEq)]
pub enum Area {
    Circle {
        left: f32,
        top: f32,
        radius: f32,
    },
    Rectangle {
        top_left: (f32, f32),
        bottom_right: (f32, f32),
    },
    Polygon {
        points: Vec<f32>,
    },
}

pub enum Shape {
    Circle,
    Rectangle,
    Polygon,
}

// https://html.spec.whatwg.org/multipage/#rules-for-parsing-a-list-of-floating-point-numbers
// https://html.spec.whatwg.org/multipage/#image-map-processing-model
impl Area {
    pub fn parse(coord: &str, target: Shape) -> Option<Area> {
        let points_count = match target {
            Shape::Circle => 3,
            Shape::Rectangle => 4,
            Shape::Polygon => 0,
        };

        let size = coord.len();
        let num = coord.as_bytes();
        let mut index = 0;

        // Step 4: Walk till char is not a delimiter
        while index < size {
            let val = num[index];
            match val {
                b',' | b';' | b' ' | b'\t' | b'\n' | 0x0C | b'\r' => {},
                _ => break,
            }

            index += 1;
        }

        // This vector will hold all parsed coordinates
        let mut number_list = Vec::new();
        let mut array = Vec::new();

        // Step 5: walk till end of string
        while index < size {
            // Step 5.1: walk till we hit a valid char i.e., 0 to 9, dot or dash, e, E
            while index < size {
                let val = num[index];
                match val {
                    b'0'..=b'9' | b'.' | b'-' | b'E' | b'e' => break,
                    _ => {},
                }

                index += 1;
            }

            // Step 5.2: collect valid symbols till we hit another delimiter
            while index < size {
                let val = num[index];

                match val {
                    b',' | b';' | b' ' | b'\t' | b'\n' | 0x0C | b'\r' => break,
                    _ => array.push(val),
                }

                index += 1;
            }

            // The input does not consist any valid characters
            if array.is_empty() {
                break;
            }

            // Convert String to float
            match str::from_utf8(&array)
                .ok()
                .and_then(|s| s.parse::<f32>().ok())
            {
                Some(v) => number_list.push(v),
                None => number_list.push(0.0),
            };

            array.clear();

            // For rectangle and circle, stop parsing once we have three
            // and four coordinates respectively
            if points_count > 0 && number_list.len() == points_count {
                break;
            }
        }

        let final_size = number_list.len();

        match target {
            Shape::Circle => {
                if final_size == 3 {
                    if number_list[2] <= 0.0 {
                        None
                    } else {
                        Some(Area::Circle {
                            left: number_list[0],
                            top: number_list[1],
                            radius: number_list[2],
                        })
                    }
                } else {
                    None
                }
            },

            Shape::Rectangle => {
                if final_size == 4 {
                    if number_list[0] > number_list[2] {
                        number_list.swap(0, 2);
                    }

                    if number_list[1] > number_list[3] {
                        number_list.swap(1, 3);
                    }

                    Some(Area::Rectangle {
                        top_left: (number_list[0], number_list[1]),
                        bottom_right: (number_list[2], number_list[3]),
                    })
                } else {
                    None
                }
            },

            Shape::Polygon => {
                if final_size >= 6 {
                    if final_size % 2 != 0 {
                        // Drop last element if there are odd number of coordinates
                        number_list.remove(final_size - 1);
                    }
                    Some(Area::Polygon {
                        points: number_list,
                    })
                } else {
                    None
                }
            },
        }
    }

    pub fn hit_test(&self, p: &Point2D<f32>) -> bool {
        match *self {
            Area::Circle { left, top, radius } => {
                (p.x - left) * (p.x - left) + (p.y - top) * (p.y - top) - radius * radius <= 0.0
            },

            Area::Rectangle {
                top_left,
                bottom_right,
            } => {
                p.x <= bottom_right.0 &&
                    p.x >= top_left.0 &&
                    p.y <= bottom_right.1 &&
                    p.y >= top_left.1
            },

            //TODO polygon hit_test
            _ => false,
        }
    }

    pub(crate) fn absolute_coords(&self, p: Point2D<f32>) -> Area {
        match *self {
            Area::Rectangle {
                top_left,
                bottom_right,
            } => Area::Rectangle {
                top_left: (top_left.0 + p.x, top_left.1 + p.y),
                bottom_right: (bottom_right.0 + p.x, bottom_right.1 + p.y),
            },
            Area::Circle { left, top, radius } => Area::Circle {
                left: (left + p.x),
                top: (top + p.y),
                radius,
            },
            Area::Polygon { ref points } => {
                //                let new_points = Vec::new();
                let iter = points
                    .iter()
                    .enumerate()
                    .map(|(index, point)| match index % 2 {
                        0 => point + p.x,
                        _ => point + p.y,
                    });
                Area::Polygon {
                    points: iter.collect::<Vec<_>>(),
                }
            },
        }
    }
}

#[dom_struct]
pub(crate) struct HTMLAreaElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableDom<DOMTokenList>,
    #[no_trace]
    relations: Cell<LinkRelations>,
    #[no_trace]
    url: DomRefCell<Option<ServoUrl>>,
}

impl HTMLAreaElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLAreaElement {
        HTMLAreaElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            rel_list: Default::default(),
            relations: Cell::new(LinkRelations::empty()),
            url: DomRefCell::new(None),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLAreaElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLAreaElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn get_shape_from_coords(&self) -> Option<Area> {
        let elem = self.upcast::<Element>();
        let shape = elem.get_string_attribute(&"shape".into());
        let shp: Shape = match_ignore_ascii_case! { &shape,
           "circle" => Shape::Circle,
           "circ" => Shape::Circle,
           "rectangle" => Shape::Rectangle,
           "rect" => Shape::Rectangle,
           "polygon" => Shape::Rectangle,
           "poly" => Shape::Polygon,
           _ => return None,
        };
        if elem.has_attribute(&"coords".into()) {
            let attribute = elem.get_string_attribute(&"coords".into());
            Area::parse(&attribute, shp)
        } else {
            None
        }
    }
}

impl HyperlinkElement for HTMLAreaElement {
    fn get_url(&self) -> &DomRefCell<Option<ServoUrl>> {
        &self.url
    }
}

impl VirtualMethods for HTMLAreaElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("rel") => AttrValue::from_serialized_tokenlist(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);

        match *attr.local_name() {
            local_name!("rel") | local_name!("rev") => {
                self.relations
                    .set(LinkRelations::for_element(self.upcast()));
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }

        self.relations
            .set(LinkRelations::for_element(self.upcast()));
    }
}

impl HTMLAreaElementMethods<crate::DomTypeHolder> for HTMLAreaElement {
    // https://html.spec.whatwg.org/multipage/#attr-hyperlink-target
    make_getter!(Target, "target");

    // https://html.spec.whatwg.org/multipage/#attr-hyperlink-target
    make_setter!(SetTarget, "target");

    // https://html.spec.whatwg.org/multipage/#dom-a-rel
    make_getter!(Rel, "rel");

    // https://html.spec.whatwg.org/multipage/#dom-a-rel
    fn SetRel(&self, rel: DOMString, can_gc: CanGc) {
        self.upcast::<Element>()
            .set_tokenlist_attribute(&local_name!("rel"), rel, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-area-rellist>
    fn RelList(&self) -> DomRoot<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(
                self.upcast(),
                &local_name!("rel"),
                Some(vec![
                    Atom::from("noopener"),
                    Atom::from("noreferrer"),
                    Atom::from("opener"),
                ]),
                CanGc::note(),
            )
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#attr-iframe-referrerpolicy>
    fn ReferrerPolicy(&self) -> DOMString {
        reflect_referrer_policy_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#attr-iframe-referrerpolicy
    make_setter!(SetReferrerPolicy, "referrerpolicy");

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-href>
    fn Href(&self) -> USVString {
        self.get_href()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-href>
    fn SetHref(&self, value: USVString, can_gc: CanGc) {
        self.set_href(value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-origin>
    fn Origin(&self) -> USVString {
        self.get_origin()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-protocol>
    fn Protocol(&self) -> USVString {
        self.get_protocol()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-protocol>
    fn SetProtocol(&self, value: USVString, can_gc: CanGc) {
        self.set_protocol(value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-password>
    fn Password(&self) -> USVString {
        self.get_password()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-password>
    fn SetPassword(&self, value: USVString, can_gc: CanGc) {
        self.set_password(value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hash>
    fn Hash(&self) -> USVString {
        self.get_hash()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hash>
    fn SetHash(&self, value: USVString, can_gc: CanGc) {
        self.set_hash(value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-host>
    fn Host(&self) -> USVString {
        self.get_host()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-host>
    fn SetHost(&self, value: USVString, can_gc: CanGc) {
        self.set_host(value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hostname>
    fn Hostname(&self) -> USVString {
        self.get_hostname()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hostname>
    fn SetHostname(&self, value: USVString, can_gc: CanGc) {
        self.set_hostname(value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-port>
    fn Port(&self) -> USVString {
        self.get_port()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-port>
    fn SetPort(&self, value: USVString, can_gc: CanGc) {
        self.set_port(value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-pathname>
    fn Pathname(&self) -> USVString {
        self.get_pathname()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-pathname>
    fn SetPathname(&self, value: USVString, can_gc: CanGc) {
        self.set_pathname(value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-search>
    fn Search(&self) -> USVString {
        self.get_search()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-search>
    fn SetSearch(&self, value: USVString, can_gc: CanGc) {
        self.set_search(value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-username>
    fn Username(&self) -> USVString {
        self.get_username()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-username>
    fn SetUsername(&self, value: USVString, can_gc: CanGc) {
        self.set_username(value, can_gc);
    }
}

impl Activatable for HTMLAreaElement {
    // https://html.spec.whatwg.org/multipage/#the-area-element:activation-behaviour
    fn as_element(&self) -> &Element {
        self.upcast::<Element>()
    }

    fn is_instance_activatable(&self) -> bool {
        self.as_element().has_attribute(&local_name!("href"))
    }

    fn activation_behavior(&self, _event: &Event, _target: &EventTarget, _can_gc: CanGc) {
        follow_hyperlink(self.as_element(), self.relations.get(), None);
    }
}
