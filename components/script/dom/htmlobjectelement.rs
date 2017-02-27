/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLObjectElementBinding;
use dom::bindings::codegen::Bindings::HTMLObjectElementBinding::HTMLObjectElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::{FormControl, HTMLFormElement};
use dom::node::{Node, window_from_node};
use dom::validation::Validatable;
use dom::validitystate::{ValidityState, ValidationFlags};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever_atoms::LocalName;
use net_traits::image::base::Image;
use std::default::Default;
use std::sync::Arc;

#[dom_struct]
pub struct HTMLObjectElement {
    htmlelement: HTMLElement,
    #[ignore_heap_size_of = "Arc"]
    image: DOMRefCell<Option<Arc<Image>>>,
    form_owner: MutNullableJS<HTMLFormElement>,
}

impl HTMLObjectElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLObjectElement {
        HTMLObjectElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            image: DOMRefCell::new(None),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLObjectElement> {
        Node::reflect_node(box HTMLObjectElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLObjectElementBinding::Wrap)
    }
}

trait ProcessDataURL {
    fn process_data_url(&self);
}

impl<'a> ProcessDataURL for &'a HTMLObjectElement {
    // Makes the local `data` member match the status of the `data` attribute and starts
    /// prefetching the image. This method must be called after `data` is changed.
    fn process_data_url(&self) {
        let elem = self.upcast::<Element>();

        // TODO: support other values
        match (elem.get_attribute(&ns!(), &local_name!("type")),
               elem.get_attribute(&ns!(), &local_name!("data"))) {
            (None, Some(_uri)) => {
                // TODO(gw): Prefetch the image here.
            }
            _ => { }
        }
    }
}

impl HTMLObjectElementMethods for HTMLObjectElement {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(&window, self.upcast())
    }

    // https://html.spec.whatwg.org/multipage/#dom-object-type
    make_getter!(Type, "type");

    // https://html.spec.whatwg.org/multipage/#dom-object-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }
}

impl Validatable for HTMLObjectElement {
    fn is_instance_validatable(&self) -> bool {
        true
    }
    fn validate(&self, validate_flags: ValidationFlags) -> bool {
        if validate_flags.is_empty() {}
        // Need more flag check for different validation types later
        true
    }
}

impl VirtualMethods for HTMLObjectElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("data") => {
                if let AttributeMutation::Set(_) = mutation {
                    self.process_data_url();
                }
            },
            &local_name!("form") => {
                self.form_attribute_mutated(mutation);
            },
            _ => {},
        }
    }
}

impl FormControl for HTMLObjectElement {
    fn form_owner(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_element<'a>(&'a self) -> &'a Element {
        self.upcast::<Element>()
    }
}
