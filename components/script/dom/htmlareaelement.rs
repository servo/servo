/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLAreaElementBinding;
use dom::bindings::codegen::Bindings::HTMLAreaElementBinding::HTMLAreaElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::domtokenlist::DOMTokenList;
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use euclid::point::Point2D;
use html5ever_atoms::LocalName;
use std::default::Default;
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
impl Area {
    pub fn parse(coord: &str, target: Shape) -> Option<Area> {
        let mut array;
        let size = coord.len();
        let num = coord.as_bytes();
        let mut index = 0;
        let mut number_list = Vec::new();
        let points_count;

        match target {
            Shape::Circle => points_count = 3,
            Shape::Rectangle => points_count = 4,
            Shape::Polygon => points_count = 0,
        }

        // Step 4: Walk till char is not a delimiter
        while index < size {
            let val = num[index];
            match val {
                0x2C | 0x3B | 0x20 | 0x09 | 0x0A | 0x0C | 0x0D  => {},
                _ => break,
            }

            index += 1;
        }

        // Step 5: walk till end of string
        while index < size {
            array = Vec::new();

            // Step 5.1: walk till we hit a valid char i.e., 0 to 9, dot or dash, e, E
            while index < size {
                let val = num[index];
                match val {
                    0x30...0x39 | 0x2E | 0x2D | 0x45 | 0x65 => break,
                    _ => {},
                }

                index += 1;
            }

            // Step 5.2: collect valid symbols till we hit another delimiter
            while index < size {
                let val = num[index];

                match val {
                    0x30...0x39 | 0x2E | 0x2D | 0x45 | 0x65 => array.push(num[index]),
                    _ => break,
                }

                index += 1;
            }

            // The input does not consist any valid charecters
            if array.is_empty() {
                continue;
            }

            // Convert String to float
            match String::from_utf8(array).unwrap().parse::<f32>() {
                Ok(v) => number_list.push(v),
                Err(_) => number_list.push(0.0),
            };

            // For rectangle and circle, stop parsing once we have three and four coordinates respectively
            if points_count > 0 && number_list.len() == points_count {
                break;
            }
        }

        let final_size = number_list.len();

        match target {
            Shape::Circle =>
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
                },

            Shape::Rectangle =>
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
                },

            Shape::Polygon =>
                if final_size >= 6 {
                    if final_size % 2 != 0 {
                        // Drop last element if there are odd number of coordinates
                        number_list.remove(final_size - 1);
                    }
                    Some(Area::Polygon { points: number_list })
                } else {
                    None
                },

            //Need not check for _ as we have already done that at the begining of the function
        }
    }

    pub fn hit_test(&self, p: Point2D<f32>) -> bool {
        match *self {
            Area::Circle { left, top, radius } =>
                (p.x - left)*(p.x - left) +
                (p.y - top)*(p.y - top) -
                radius * radius <= 0.0,

            Area::Rectangle { top_left, bottom_right } =>
                p.x <= bottom_right.0 && p.x >= top_left.0 &&
                p.y <= bottom_right.1 && p.y >= top_left.1,

            _ => false,
        }
    }
}

#[dom_struct]
pub struct HTMLAreaElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableHeap<JS<DOMTokenList>>,
}

impl HTMLAreaElement {
    fn new_inherited(local_name: LocalName, prefix: Option<DOMString>, document: &Document) -> HTMLAreaElement {
        HTMLAreaElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            rel_list: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLAreaElement> {
        Node::reflect_node(box HTMLAreaElement::new_inherited(local_name, prefix, document),
               document,
               HTMLAreaElementBinding::Wrap)
    }

    #[warn(dead_code)]
    fn get_shape_from_coords(&self) -> Option<Area> {
       let elem = self.upcast::<Element>();
       let shp: Shape;
       let shape = elem.get_string_attribute(&LocalName::from("shape"));

       match shape.as_ref() {
           "Circle" => shp = Shape::Circle,
           "Rectangle" => shp = Shape::Rectangle,
           "Polygon" => shp = Shape::Polygon,
           _ => return None,
}
        if elem.has_attribute(&LocalName::from("coords")) {
            let attribute = elem.get_string_attribute(&LocalName::from("coords"));
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
