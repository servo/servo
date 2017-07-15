/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::js::LayoutJS;
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use style::element_state::ElementState;
use style::properties::declaration_block::{Importance, PropertyDeclarationBlock};
use style::properties::{parse_one_declaration_into, SourcePropertyDeclaration};
use style::properties::PropertyId;
use style::shared_lock::Locked;
use style::stylearc::Arc;
use style_traits::PARSING_MODE_ALLOW_UNITLESS_LENGTH;

#[dom_struct]
pub struct SVGElement {
    element: Element,
    #[ignore_heap_size_of = "Arc"]
    presentation_attributes: Arc<Locked<PropertyDeclarationBlock>>
}

impl SVGElement {
    pub fn new_inherited_with_state(state: ElementState,
                                    tag_name: LocalName,
                                    prefix: Option<Prefix>,
                                    document: &Document)
                                    -> SVGElement {

        let shared_lock = document.style_shared_lock();

        SVGElement {
            element: Element::new_inherited_with_state(state,
                                                       tag_name,
                                                       ns!(svg),
                                                       prefix,
                                                       document),
            presentation_attributes: Arc::new(
                shared_lock.wrap(PropertyDeclarationBlock::new())
            )
        }
    }

    pub fn presentation_attributes(&self) -> Arc<Locked<PropertyDeclarationBlock>> {
        self.presentation_attributes.clone()
    }

    pub fn mutate_svg_preshint(&self, attr: &Attr) -> bool {
        let global = self.global();
        let win = global.as_window();
        let document = win.Document();
        let quirks_mode = document.quirks_mode();
        let url = win.get_url();

        let id = PropertyId::parse(DOMString::from(&**attr.local_name()).into())
            .expect("This SVG presentation attribute is not a css property!");
        let mut declarations = SourcePropertyDeclaration::new();
        let result = parse_one_declaration_into(&mut declarations,
                                                id.clone(),
                                                &attr.value(),
                                                &url,
                                                win.css_error_reporter(),
                                                PARSING_MODE_ALLOW_UNITLESS_LENGTH,
                                                quirks_mode);
        let mut write_lock = document.style_shared_lock().write();
        let lock = self.presentation_attributes();
        let pdb = lock.write_with(&mut write_lock);
        match result {
            Ok(()) => pdb.extend_reset(declarations.drain(), Importance::Normal),
            Err(_) => pdb.remove_property(&id)
        }
    }
}

pub trait LayoutSVGElementHelpers {
    fn presentation_attributes(&self) -> Arc<Locked<PropertyDeclarationBlock>>;
}

impl LayoutSVGElementHelpers for LayoutJS<SVGElement> {
    #[allow(unsafe_code)]
    fn presentation_attributes(&self) -> Arc<Locked<PropertyDeclarationBlock>> {
        unsafe {
            (*self.unsafe_get()).presentation_attributes()
        }
    }
}

impl VirtualMethods for SVGElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<Element>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        match attr.local_name() {
            &local_name!("fill") => {
                self.mutate_svg_preshint(attr);
            }
            _ => {}
        }
    }
}
