/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementMethods;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementConstants;
use dom::bindings::inheritance::Castable;
use dom::document::Document;
use dom::element::AttributeMutation;
use dom::htmlelement::HTMLElement;
use dom::virtualmethods::VirtualMethods;
use std::cell::Cell;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLMediaElement {
    htmlelement: HTMLElement,
    network_state: Cell<u16>,
    ready_state: Cell<u16>,
    current_src: DOMRefCell<String>,
}

impl HTMLMediaElement {
    pub fn new_inherited(tag_name: DOMString,
                         prefix: Option<DOMString>, document: &Document)
                         -> HTMLMediaElement {
        HTMLMediaElement {
            htmlelement:
                HTMLElement::new_inherited(tag_name, prefix, document),
            network_state: Cell::new(HTMLMediaElementConstants::NETWORK_EMPTY),
            ready_state: Cell::new(HTMLMediaElementConstants::HAVE_NOTHING),
            current_src: DOMRefCell::new("".to_owned()),
        }
    }

    #[inline]
    pub fn htmlelement(&self) -> &HTMLElement {
        &self.htmlelement
    }

    fn media_element_load_algorithm(&self, _src: &str) {
    }
}

impl HTMLMediaElementMethods for HTMLMediaElement {
    fn NetworkState(&self) -> u16 {
        self.network_state.get()
    }

    fn ReadyState(&self) -> u16 {
        self.ready_state.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_getter!(Src, "src");
    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-media-currentsrc
    fn CurrentSrc(&self) -> DOMString {
        DOMString(self.current_src.borrow().clone())
    }
}

impl VirtualMethods for HTMLMediaElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        match attr.local_name() {
            &atom!(src) => {
                if let Some(value) = mutation.new_value(attr) {
                    self.media_element_load_algorithm(&value);
                }
            }
            _ => (),
        };
    }
}
