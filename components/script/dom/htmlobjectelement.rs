/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use pixels::Image;
use servo_arc::Arc;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLObjectElementBinding::HTMLObjectElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::node::{Node, NodeTraits};
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidityState;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLObjectElement {
    htmlelement: HTMLElement,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    image: DomRefCell<Option<Arc<Image>>>,
    form_owner: MutNullableDom<HTMLFormElement>,
    validity_state: MutNullableDom<ValidityState>,
}

impl HTMLObjectElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLObjectElement {
        HTMLObjectElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            image: DomRefCell::new(None),
            form_owner: Default::default(),
            validity_state: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLObjectElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLObjectElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }
}

trait ProcessDataURL {
    fn process_data_url(&self);
}

impl ProcessDataURL for &HTMLObjectElement {
    // Makes the local `data` member match the status of the `data` attribute and starts
    /// prefetching the image. This method must be called after `data` is changed.
    fn process_data_url(&self) {
        let element = self.upcast::<Element>();

        // TODO: support other values
        if let (None, Some(_uri)) = (
            element.get_attribute(&ns!(), &local_name!("type")),
            element.get_attribute(&ns!(), &local_name!("data")),
        ) {
            // TODO(gw): Prefetch the image here.
        }
    }
}

impl HTMLObjectElementMethods<crate::DomTypeHolder> for HTMLObjectElement {
    // https://html.spec.whatwg.org/multipage/#dom-object-type
    make_getter!(Type, "type");

    // https://html.spec.whatwg.org/multipage/#dom-object-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-willvalidate
    fn WillValidate(&self) -> bool {
        self.is_instance_validatable()
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> DomRoot<ValidityState> {
        self.validity_state()
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-checkvalidity
    fn CheckValidity(&self, can_gc: CanGc) -> bool {
        self.check_validity(can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-reportvalidity
    fn ReportValidity(&self, can_gc: CanGc) -> bool {
        self.report_validity(can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-validationmessage
    fn ValidationMessage(&self) -> DOMString {
        self.validation_message()
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-setcustomvalidity
    fn SetCustomValidity(&self, error: DOMString) {
        self.validity_state().set_custom_error_message(error);
    }
}

impl Validatable for HTMLObjectElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn validity_state(&self) -> DomRoot<ValidityState> {
        self.validity_state
            .or_init(|| ValidityState::new(&self.owner_window(), self.upcast()))
    }

    fn is_instance_validatable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#the-object-element%3Abarred-from-constraint-validation
        false
    }
}

impl VirtualMethods for HTMLObjectElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match *attr.local_name() {
            local_name!("data") => {
                if let AttributeMutation::Set(_) = mutation {
                    self.process_data_url();
                }
            },
            local_name!("form") => {
                self.form_attribute_mutated(mutation);
            },
            _ => {},
        }
    }
}

impl FormControl for HTMLObjectElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_element(&self) -> &Element {
        self.upcast::<Element>()
    }
}
