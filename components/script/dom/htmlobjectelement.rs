/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLObjectElementBinding;
use dom::bindings::codegen::Bindings::HTMLObjectElementBinding::HTMLObjectElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLObjectElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::AttributeHandlers;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, NodeHelpers, window_from_node};
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;

use net_traits::image::base::Image;
use std::sync::Arc;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLObjectElement {
    htmlelement: HTMLElement,
    image: DOMRefCell<Option<Arc<Image>>>,
}

impl HTMLObjectElementDerived for EventTarget {
    fn is_htmlobjectelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)))
    }
}

impl HTMLObjectElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLObjectElement {
        HTMLObjectElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLObjectElement, localName, prefix, document),
            image: DOMRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLObjectElement> {
        let element = HTMLObjectElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLObjectElementBinding::Wrap)
    }
}

trait ProcessDataURL {
    fn process_data_url(&self);
}

impl<'a> ProcessDataURL for &'a HTMLObjectElement {
    // Makes the local `data` member match the status of the `data` attribute and starts
    /// prefetching the image. This method must be called after `data` is changed.
    fn process_data_url(&self) {
        let elem = ElementCast::from_ref(*self);

        // TODO: support other values
        match (elem.get_attribute(&ns!(""), &atom!("type")).map(|x| x.r().Value()),
               elem.get_attribute(&ns!(""), &atom!("data")).map(|x| x.r().Value())) {
            (None, Some(_uri)) => {
                // TODO(gw): Prefetch the image here.
            }
            _ => { }
        }
    }
}

pub fn is_image_data(uri: &str) -> bool {
    static TYPES: &'static [&'static str] = &["data:image/png", "data:image/gif", "data:image/jpeg"];
    TYPES.iter().any(|&type_| uri.starts_with(type_))
}

impl<'a> HTMLObjectElementMethods for &'a HTMLObjectElement {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(window.r())
    }

    // https://html.spec.whatwg.org/multipage/#dom-object-type
    make_getter!(Type);

    // https://html.spec.whatwg.org/multipage/#dom-object-type
    make_setter!(SetType, "type");
}

impl VirtualMethods for HTMLObjectElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("data") => {
                self.process_data_url();
            },
            _ => ()
        }
    }
}
