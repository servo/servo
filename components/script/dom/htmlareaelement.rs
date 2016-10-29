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

impl Area {
    pub fn get_area (coord: String) -> Area
    {
        let num;
        let size;
        let mut stringval;
        let mut array;
        let mut index;
        let mut numberlist;

        size    = coord.len();
        num     = coord.into_bytes();
        index   = 0;
        numberlist = Vec::new();

        //Step 4: Walk till char is not a delimiter
        while index < size {
            let val = num[index];

            if val != 0x2c && val != 0x3B && val != 0x20 && val != 0x09 && val != 0x0A && val != 0x0C && val != 0x0D {
                break;
            }

            index = index + 1;
        }

        //Step 5: walk till end of string
        while index < size {
            array = Vec::new();

            //Step 5.1: walk till we hit a valid char i.e., 0 to 9, dot or dash, e, E
            while index < size {
                let val = num[index];

                if val >= 0x30 && val <= 0x39 || val == 0x2E || val == 0x2D || val == 0x45 || val == 0x65 {
                    break;
                }

                index = index + 1;
            }

            //Step 5.2: collect valid symbols till we hit another delimiter
            while index < size {
                let val = num[index];

                if val >= 0x30 && val <= 0x39 || val == 0x2E || val == 0x2D || val == 0x45 || val == 0x65 {
                    array.push(num[index]);
                } else {
                    break;
                }

                index = index + 1;
            }

            //only junk exist
            if array.len() == 0 {
                continue;
            }

            //Convert String to float
            stringval = String::from_utf8(array);
            match stringval.unwrap().parse::<f32>() {
                Ok(v) => numberlist.push(v),
                Err(_) => numberlist.push(0.0),
            };
        }
        let final_size = numberlist.len();

        if final_size == 3 {
            if numberlist[2] <= 0.0 {
                Area::Circle { left: 0.0, top: 0.0, radius: 0.0 }
            } else {
                Area::Circle { left: numberlist[0], top: numberlist[1], radius: numberlist[2] }
            }
        } else if final_size == 4 {
            if numberlist[0] > numberlist[2] {
                let temp = numberlist[0];
                numberlist[0] = numberlist[2];
                numberlist[2] = temp;
            }

            if numberlist[1] > numberlist[3] {
                let temp = numberlist[1];
                numberlist[1] = numberlist[3];
                numberlist[3] = temp;
            }

            Area::Rectangle { left_l: numberlist[0], top_t: numberlist[1], left_r: numberlist[2], top_b: numberlist[3] }
        } else if final_size >= 6 {
            //check if even
            if final_size % 2 != 0 {
                //drop last element
                numberlist.remove (final_size - 1);
            }
            Area::Polygon { points: numberlist }
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
        //This will be updated in second half of image maps feature
        let attribute: String = "10.0, 10.5, 5.3, 6.4".to_string();
        Area::get_area(attribute);
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
