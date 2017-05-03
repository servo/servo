/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::activation::Activatable;
use dom::bindings::codegen::Bindings::DOMTokenListBinding::DOMTokenListMethods;
use dom::bindings::codegen::Bindings::HTMLAreaElementBinding;
use dom::bindings::codegen::Bindings::HTMLAreaElementBinding::HTMLAreaElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::domtokenlist::DOMTokenList;
use dom::element::Element;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::htmlanchorelement::follow_hyperlink;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, document_from_node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use euclid::point::Point2D;
use html5ever::{LocalName, Prefix};
use net_traits::ReferrerPolicy;
use std::default::Default;
use std::f32;
use style::attr::AttrValue;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Area {
    Circle { left: f32, top: f32, radius: f32 },
    Rectangle { top_left: (f32, f32), bottom_right: (f32, f32) },
    Polygon { points: Vec<f32> },
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
        let points_count =  match target {
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
                b',' | b';' | b' ' | b'\t' | b'\n' | 0x0C | b'\r'  => {},
                _ => break,
            }

            index += 1;
        }

        //This vector will hold all parsed coordinates
        let mut number_list = Vec::new();
        let mut array = Vec::new();
        let ar_ref = &mut array;

        // Step 5: walk till end of string
        while index < size {
            // Step 5.1: walk till we hit a valid char i.e., 0 to 9, dot or dash, e, E
            while index < size {
                let val = num[index];
                match val {
                    b'0'...b'9' | b'.' | b'-' | b'E' | b'e' => break,
                    _ => {},
                }

                index += 1;
            }

            // Step 5.2: collect valid symbols till we hit another delimiter
            while index < size {
                let val = num[index];

                match val {
                    b',' | b';' | b' ' | b'\t' | b'\n' | 0x0C | b'\r' => break,
                    _ => (*ar_ref).push(val),
                }

                index += 1;
            }

            // The input does not consist any valid charecters
            if (*ar_ref).is_empty() {
                break;
            }

            // Convert String to float
            match String::from_utf8((*ar_ref).clone()).unwrap().parse::<f32>() {
                Ok(v) => number_list.push(v),
                Err(_) => number_list.push(0.0),
            };

            (*ar_ref).clear();

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
                                 radius: number_list[2]
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
                             bottom_right: (number_list[2], number_list[3])
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
                    Some(Area::Polygon { points: number_list })
                } else {
                    None
                }
            },
        }
    }

    pub fn hit_test(&self, p: Point2D<f32>) -> bool {
        match *self {
            Area::Circle { left, top, radius } => {
                (p.x - left) * (p.x - left) +
                (p.y - top) * (p.y - top) -
                radius * radius <= 0.0
            },

            Area::Rectangle { top_left, bottom_right } => {
                p.x <= bottom_right.0 && p.x >= top_left.0 &&
                p.y <= bottom_right.1 && p.y >= top_left.1
            },

            //TODO polygon hit_test
            _ => false,
        }
    }

    pub fn absolute_coords(&self, p: Point2D<f32>) -> Area {
        match *self {
            Area::Rectangle { top_left, bottom_right } => {
                Area::Rectangle {
                    top_left: (top_left.0 + p.x, top_left.1 + p.y),
                    bottom_right: (bottom_right.0 + p.x, bottom_right.1 + p.y)
                }
            },
            Area::Circle { left, top, radius } => {
                Area::Circle {
                    left: (left + p.x),
                    top: (top + p.y),
                    radius: radius
                }
            },
            Area::Polygon { ref points } => {
//                let new_points = Vec::new();
                let iter = points.iter().enumerate().map(|(index, point)| {
                    match index % 2 {
                        0 => point + p.x as f32,
                        _ => point + p.y as f32,
                    }
                });
                Area::Polygon { points: iter.collect::<Vec<_>>() }
            },
        }
    }
}

#[dom_struct]
pub struct HTMLAreaElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableJS<DOMTokenList>,
}

impl HTMLAreaElement {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document) -> HTMLAreaElement {
        HTMLAreaElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            rel_list: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document) -> Root<HTMLAreaElement> {
        Node::reflect_node(box HTMLAreaElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLAreaElementBinding::Wrap)
    }

    pub fn get_shape_from_coords(&self) -> Option<Area> {
       let elem = self.upcast::<Element>();
       let shape = elem.get_string_attribute(&"shape".into());
       let shp: Shape = match shape.to_lowercase().as_ref() {
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

impl VirtualMethods for HTMLAreaElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("rel") => AttrValue::from_serialized_tokenlist(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

impl HTMLAreaElementMethods for HTMLAreaElement {
    // https://html.spec.whatwg.org/multipage/#dom-area-rellist
    fn RelList(&self) -> Root<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(self.upcast(), &local_name!("rel"))
        })
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

    fn pre_click_activation(&self) {
    }

    fn canceled_activation(&self) {
    }

    fn implicit_submission(&self, _ctrl_key: bool, _shift_key: bool,
                           _alt_key: bool, _meta_key: bool) {
    }

    fn activation_behavior(&self, _event: &Event, _target: &EventTarget) {
        // Step 1
        let doc = document_from_node(self);
        if !doc.is_fully_active() {
            return;
        }
        // Step 2
        // TODO : We should be choosing a browsing context and navigating to it.
        // Step 3
        let referrer_policy = match self.RelList().Contains("noreferrer".into()) {
            true => Some(ReferrerPolicy::NoReferrer),
            false => None,
        };
        follow_hyperlink(self.upcast::<Element>(), None, referrer_policy);
    }
}
