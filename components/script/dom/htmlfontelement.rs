/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::Bindings::HTMLFontElementBinding;
use dom::bindings::codegen::Bindings::HTMLFontElementBinding::HTMLFontElementMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLFontElementDerived};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use dom::virtualmethods::VirtualMethods;

use style::properties::DeclaredValue::SpecifiedValue;
use style::properties::PropertyDeclaration;
use style::values::specified::CSSRGBA;
use util::str::{self, DOMString};

#[dom_struct]
pub struct HTMLFontElement {
    htmlelement: HTMLElement,
}

impl HTMLFontElementDerived for EventTarget {
    fn is_htmlfontelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFontElement)))
    }
}

impl HTMLFontElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLFontElement {
        HTMLFontElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLFontElement, localName, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLFontElement> {
        let element = HTMLFontElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLFontElementBinding::Wrap)
    }
}

impl<'a> HTMLFontElementMethods for &'a HTMLFontElement {
    make_getter!(Color, "color");
    make_setter!(SetColor, "color");
}

impl<'a> VirtualMethods for &'a HTMLFontElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn presentational_hints(&self, attribute: &Attr) -> Vec<PropertyDeclaration> {
        let mut hints = self.super_type().unwrap().presentational_hints(attribute);

        match attribute.local_name() {
            &atom!("color") => {
                if let Ok(color) = str::parse_legacy_color(&attribute.value()) {
                    hints.push(PropertyDeclaration::Color(SpecifiedValue(CSSRGBA {
                        parsed: color,
                        authored: None,
                    })));
                }
            },
            _ => (),
        }

        hints
    }
}
