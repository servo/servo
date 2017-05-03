/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::bindings::codegen::Bindings::HTMLFontElementBinding;
use dom::bindings::codegen::Bindings::HTMLFontElementBinding::HTMLFontElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{LayoutJS, Root};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{Element, RawLayoutElementHelpers};
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::LocalName;
use servo_atoms::Atom;
use style::attr::AttrValue;
use style::str::{HTML_SPACE_CHARACTERS, read_numbers};

#[dom_struct]
pub struct HTMLFontElement {
    htmlelement: HTMLElement,
}


impl HTMLFontElement {
    fn new_inherited(local_name: LocalName, prefix: Option<DOMString>, document: &Document) -> HTMLFontElement {
        HTMLFontElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLFontElement> {
        Node::reflect_node(box HTMLFontElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLFontElementBinding::Wrap)
    }
}

impl HTMLFontElementMethods for HTMLFontElement {
    // https://html.spec.whatwg.org/multipage/#dom-font-color
    make_getter!(Color, "color");

    // https://html.spec.whatwg.org/multipage/#dom-font-color
    make_legacy_color_setter!(SetColor, "color");

    // https://html.spec.whatwg.org/multipage/#dom-font-face
    make_getter!(Face, "face");

    // https://html.spec.whatwg.org/multipage/#dom-font-face
    make_atomic_setter!(SetFace, "face");

    // https://html.spec.whatwg.org/multipage/#dom-font-size
    make_getter!(Size, "size");

    // https://html.spec.whatwg.org/multipage/#dom-font-size
    fn SetSize(&self, value: DOMString) {
        let element = self.upcast::<Element>();
        element.set_attribute(&local_name!("size"), parse_size(&value));
    }
}

impl VirtualMethods for HTMLFontElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("face") => AttrValue::from_atomic(value.into()),
            &local_name!("color") => AttrValue::from_legacy_color(value.into()),
            &local_name!("size") => parse_size(&value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

pub trait HTMLFontElementLayoutHelpers {
    fn get_color(&self) -> Option<RGBA>;
    fn get_face(&self) -> Option<Atom>;
    fn get_size(&self) -> Option<u32>;
}

impl HTMLFontElementLayoutHelpers for LayoutJS<HTMLFontElement> {
    #[allow(unsafe_code)]
    fn get_color(&self) -> Option<RGBA> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("color"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }

    #[allow(unsafe_code)]
    fn get_face(&self) -> Option<Atom> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("face"))
                .map(AttrValue::as_atom)
                .cloned()
        }
    }

    #[allow(unsafe_code)]
    fn get_size(&self) -> Option<u32> {
        let size = unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("size"))
        };
        match size {
            Some(&AttrValue::UInt(_, s)) => Some(s),
            _ => None,
        }
    }
}

/// https://html.spec.whatwg.org/multipage/#rules-for-parsing-a-legacy-font-size
fn parse_size(mut input: &str) -> AttrValue {
    let original_input = input;
    // Steps 1 & 2 are not relevant

    // Step 3
    input = input.trim_matches(HTML_SPACE_CHARACTERS);

    enum ParseMode {
        RelativePlus,
        RelativeMinus,
        Absolute,
    }
    let mut input_chars = input.chars().peekable();
    let parse_mode = match input_chars.peek() {
        // Step 4
        None => return AttrValue::String(original_input.into()),

        // Step 5
        Some(&'+') => {
            let _ = input_chars.next();  // consume the '+'
            ParseMode::RelativePlus
        }
        Some(&'-') => {
            let _ = input_chars.next();  // consume the '-'
            ParseMode::RelativeMinus
        }
        Some(_) => ParseMode::Absolute,
    };

    // Steps 6, 7, 8
    let mut value = match read_numbers(input_chars) {
        (Some(v), _) if v >= 0 => v,
        _ => return AttrValue::String(original_input.into()),
    };

    // Step 9
    match parse_mode {
        ParseMode::RelativePlus => value = 3 + value,
        ParseMode::RelativeMinus => value = 3 - value,
        ParseMode::Absolute => (),
    }

    // Steps 10, 11, 12
    AttrValue::UInt(original_input.into(), value as u32)
}
