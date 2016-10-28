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
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use euclid::point::Point2D;
use std::default::Default;
use string_cache::Atom;
use style::attr::AttrValue;

pub enum Area {
    Circle { left: f32, top: f32, radius: f32 },
    Rectangle { left_l: f32, top_t: f32, left_r: f32, top_b: f32 },
    Polygon { points: Vec<f32> },
    Default,
}

// https://html.spec.whatwg.org/multipage/#rules-for-parsing-a-list-of-floating-point-numbers
impl Area {
    pub fn get_area(coord: &str) -> Area {
        let mut array;
        let size = coord.len();
        let num = coord.as_bytes();
        let mut index = 0;
        let mut number_list = Vec::new();

        // Step 4: Walk till char is not a delimiter
        while index < size
        {
            let val = num[index];
            match val {
                0x2C | 0x3B | 0x20 | 0x09 | 0x0A | 0x0C | 0x0D  => {},
                _ => break,
            }

            index += 1;
        }

        // Step 5: walk till end of string
        while index < size
        {
            array = Vec::new();

            // Step 5.1: walk till we hit a valid char i.e., 0 to 9, dot or dash, e, E
            while index < size
            {
                let val = num[index];
                match val {
                    0x30...0x39 | 0x2E | 0x2D | 0x45 | 0x65 => break,
                    _ => {},
                }

                index += 1;
            }

            // Step 5.2: collect valid symbols till we hit another delimiter
            while index < size
            {
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
            let mut string_val = String::from_utf8(array);
            match string_val.unwrap().parse::<f32>() {
                Ok(v) => number_list.push(v),
                Err(_) => number_list.push(0.0),
            };
        }

        let final_size = number_list.len();

        if final_size == 3 {
            if number_list[2] <= 0.0 {
                Area::Default
            } else {
                Area::Circle { left: number_list[0], top: number_list[1], radius: number_list[2] }
            }
        } else if final_size == 4 {
            if number_list[0] > number_list[2] {
                let temp = number_list[0];
                number_list[0] = number_list[2];
                number_list[2] = temp;
            }

            if number_list[1] > number_list[3] {
                let temp = number_list[1];
                number_list[1] = number_list[3];
                number_list[3] = temp;
            }

            Area::Rectangle { left_l: number_list[0], top_t: number_list[1], left_r: number_list[2],
                              top_b: number_list[3] }
        } else if final_size >= 6 {
            if final_size % 2 != 0 {
                // Drop last element if there are odd number of coordinates
                number_list.remove(final_size - 1);
            }
            Area::Polygon { points: number_list }
        } else {
            Area::Default
        }
    }

    pub fn hit_test(&self, p: Point2D<f32>) -> bool {
        match *self {
            Area::Circle { left, top, radius } => (p.x - left)*(p.x - left) +
                                                  (p.y - top)*(p.y - top) -
                                                  radius * radius <= 0.0,

            Area::Rectangle { left_l, top_t, left_r, top_b } => p.x <= left_r && p.x >= left_l &&
                                                                p.y <= top_b && p.y >= top_t,

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
    fn new_inherited(local_name: Atom, prefix: Option<DOMString>, document: &Document) -> HTMLAreaElement {
        HTMLAreaElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            rel_list: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLAreaElement> {
        Node::reflect_node(box HTMLAreaElement::new_inherited(local_name, prefix, document),
               document,
               HTMLAreaElementBinding::Wrap)
    }

    fn get_shape_from_coords(&self) -> Area {
        // This will be updated in second half of image maps feature
        let attribute: &str = "10.0, 10.5, 5.3, 6.4";
        Area::get_area(attribute)
    }

}

impl VirtualMethods for HTMLAreaElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("rel") => AttrValue::from_serialized_tokenlist(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

impl HTMLAreaElementMethods for HTMLAreaElement {
    // https://html.spec.whatwg.org/multipage/#dom-area-rellist
    fn RelList(&self) -> Root<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(self.upcast(), &atom!("rel"))
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-coords
    make_getter!(Coords, "coords");
    // https://html.spec.whatwg.org/multipage/#dom-a-coords
    make_setter!(SetCoords, "coords");
}
