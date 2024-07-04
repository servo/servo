/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::match_ignore_ascii_case;
use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use servo_atoms::Atom;
use style::attr::AttrValue;
use style::color::AbsoluteColor;
use style::str::{read_numbers, HTML_SPACE_CHARACTERS};
use style::values::computed::font::{
    FamilyName, FontFamilyNameSyntax, GenericFontFamily, SingleFontFamily,
};

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLFontElementBinding::HTMLFontElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{Element, LayoutElementHelpers};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::dom::virtualmethods::VirtualMethods;

#[dom_struct]
pub struct HTMLFontElement {
    htmlelement: HTMLElement,
}

impl HTMLFontElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLFontElement {
        HTMLFontElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLFontElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLFontElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }

    pub(crate) fn parse_face_attribute(face_value: Atom) -> Vec<SingleFontFamily> {
        face_value
            .split(',')
            .map(|string| Self::parse_single_face_value_from_string(string.trim()))
            .collect()
    }

    fn parse_single_face_value_from_string(string: &str) -> SingleFontFamily {
        match_ignore_ascii_case! { string,
            "serif" => return SingleFontFamily::Generic(GenericFontFamily::Serif),
            "sans-serif" => return SingleFontFamily::Generic(GenericFontFamily::SansSerif),
            "cursive" => return SingleFontFamily::Generic(GenericFontFamily::Cursive),
            "fantasy" => return SingleFontFamily::Generic(GenericFontFamily::Fantasy),
            "monospace" => return SingleFontFamily::Generic(GenericFontFamily::Monospace),
            "system-ui" => return SingleFontFamily::Generic(GenericFontFamily::SystemUi),
            _ => {}
        }

        let name = string.to_owned().replace(['\'', '"'], "");
        let syntax = if name == string {
            FontFamilyNameSyntax::Identifiers
        } else {
            FontFamilyNameSyntax::Quoted
        };

        SingleFontFamily::FamilyName(FamilyName {
            name: name.into(),
            syntax,
        })
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
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_affects_presentational_hints(&self, attr: &Attr) -> bool {
        if attr.local_name() == &local_name!("color") ||
            attr.local_name() == &local_name!("size") ||
            attr.local_name() == &local_name!("face")
        {
            return true;
        }

        self.super_type()
            .unwrap()
            .attribute_affects_presentational_hints(attr)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("face") => AttrValue::from_atomic(value.into()),
            local_name!("color") => AttrValue::from_legacy_color(value.into()),
            local_name!("size") => parse_size(&value),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }
}

pub trait HTMLFontElementLayoutHelpers {
    fn get_color(self) -> Option<AbsoluteColor>;
    fn get_face(self) -> Option<Atom>;
    fn get_size(self) -> Option<u32>;
}

impl HTMLFontElementLayoutHelpers for LayoutDom<'_, HTMLFontElement> {
    fn get_color(self) -> Option<AbsoluteColor> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("color"))
            .and_then(AttrValue::as_color)
            .cloned()
    }

    fn get_face(self) -> Option<Atom> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("face"))
            .map(AttrValue::as_atom)
            .cloned()
    }

    fn get_size(self) -> Option<u32> {
        let size = self
            .upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("size"));
        match size {
            Some(&AttrValue::UInt(_, s)) => Some(s),
            _ => None,
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-a-legacy-font-size>
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
            let _ = input_chars.next(); // consume the '+'
            ParseMode::RelativePlus
        },
        Some(&'-') => {
            let _ = input_chars.next(); // consume the '-'
            ParseMode::RelativeMinus
        },
        Some(_) => ParseMode::Absolute,
    };

    // Steps 6, 7, 8
    let mut value = match read_numbers(input_chars) {
        (Some(v), _) if v >= 0 => v,
        _ => return AttrValue::String(original_input.into()),
    };

    // Step 9
    match parse_mode {
        ParseMode::RelativePlus => value += 3,
        ParseMode::RelativeMinus => value = 3 - value,
        ParseMode::Absolute => (),
    }

    // Steps 10, 11, 12
    AttrValue::UInt(original_input.into(), value as u32)
}
