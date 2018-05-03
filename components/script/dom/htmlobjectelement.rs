/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::HTMLObjectElementBinding;
use dom::bindings::codegen::Bindings::HTMLObjectElementBinding::HTMLObjectElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{DomRoot, MutNullableDom};
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
use html5ever::{LocalName, Prefix};
use net_traits::image::base::Image;
use servo_arc::Arc;
use std::default::Default;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLObjectElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
    #[ignore_malloc_size_of = "Arc"]
    image: DomRefCell<Option<Arc<Image>>>,
    form_owner: MutNullableDom<HTMLFormElement<TH>>,
}

impl<TH: TypeHolderTrait> HTMLObjectElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLObjectElement<TH> {
        HTMLObjectElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            image: DomRefCell::new(None),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLObjectElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLObjectElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLObjectElementBinding::Wrap)
    }
}

trait ProcessDataURL {
    fn process_data_url(&self);
}

impl<'a, TH: TypeHolderTrait> ProcessDataURL for &'a HTMLObjectElement<TH> {
    // Makes the local `data` member match the status of the `data` attribute and starts
    /// prefetching the image. This method must be called after `data` is changed.
    fn process_data_url(&self) {
        let elem = self.upcast::<Element<TH>>();

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

impl<TH: TypeHolderTrait> HTMLObjectElementMethods<TH> for HTMLObjectElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> DomRoot<ValidityState<TH>> {
        let window = window_from_node(self);
        ValidityState::new(&window, self.upcast())
    }

    // https://html.spec.whatwg.org/multipage/#dom-object-type
    make_getter!(Type, "type");

    // https://html.spec.whatwg.org/multipage/#dom-object-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement<TH>>> {
        self.form_owner()
    }
}

impl<TH: TypeHolderTrait> Validatable for HTMLObjectElement<TH> {
    fn is_instance_validatable(&self) -> bool {
        true
    }
    fn validate(&self, validate_flags: ValidationFlags) -> bool {
        if validate_flags.is_empty() {}
        // Need more flag check for different validation types later
        true
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLObjectElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn attribute_mutated(&self, attr: &Attr<TH>, mutation: AttributeMutation<TH>) {
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

impl<TH: TypeHolderTrait> FormControl<TH> for HTMLObjectElement<TH> {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement<TH>>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement<TH>>) {
        self.form_owner.set(form);
    }

    fn to_element<'a>(&'a self) -> &'a Element<TH> {
        self.upcast::<Element<TH>>()
    }
}
