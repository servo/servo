/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLOutputElementBinding;
use crate::dom::bindings::codegen::Bindings::HTMLOutputElementBinding::HTMLOutputElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::node::{window_from_node, Node};
use crate::dom::nodelist::NodeList;
use crate::dom::validitystate::ValidityState;
use crate::dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};

#[dom_struct]
pub struct HTMLOutputElement {
    htmlelement: HTMLElement,
    form_owner: MutNullableDom<HTMLFormElement>,
    labels_node_list: MutNullableDom<NodeList>,
    default_value_override: DomRefCell<Option<DOMString>>,
}

impl HTMLOutputElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLOutputElement {
        HTMLOutputElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            form_owner: Default::default(),
            labels_node_list: Default::default(),
            default_value_override: DomRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLOutputElement> {
        Node::reflect_node(
            Box::new(HTMLOutputElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            HTMLOutputElementBinding::Wrap,
        )
    }

    pub fn reset(&self) {
        Node::string_replace_all(self.DefaultValue(), self.upcast::<Node>());
        *self.default_value_override.borrow_mut() = None;
    }
}

impl HTMLOutputElementMethods for HTMLOutputElement {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> DomRoot<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(&window, self.upcast())
    }

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    make_labels_getter!(Labels, labels_node_list);

    // https://html.spec.whatwg.org/multipage/#dom-output-defaultvaleu
    fn DefaultValue(&self) -> DOMString {
        let dvo = self.default_value_override.borrow();
        if let Some(ref dv) = *dvo {
            dv.clone()
        } else {
            self.upcast::<Node>().descendant_text_content()
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-output-defaultvalue
    fn SetDefaultValue(&self, value: DOMString) {
        if self.default_value_override.borrow().is_none() {
            // Step 1 ("and return")
            Node::string_replace_all(value.clone(), self.upcast::<Node>());
        } else {
            // Step 2, if not returned from step 1
            *self.default_value_override.borrow_mut() = Some(value);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-output-value
    fn Value(&self) -> DOMString {
        self.upcast::<Node>().descendant_text_content()
    }

    // https://html.spec.whatwg.org/multipage/#dom-output-value
    fn SetValue(&self, value: DOMString) {
        *self.default_value_override.borrow_mut() = Some(self.DefaultValue());
        Node::string_replace_all(value, self.upcast::<Node>());
    }

    // https://html.spec.whatwg.org/multipage/#dom-output-type
    fn Type(&self) -> DOMString {
        return DOMString::from("output");
    }
}

impl VirtualMethods for HTMLOutputElement {
    fn super_type<'b>(&'b self) -> Option<&'b dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("form") => {
                self.form_attribute_mutated(mutation);
            },
            _ => {},
        }
    }
}

impl FormControl for HTMLOutputElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_element<'a>(&'a self) -> &'a Element {
        self.upcast::<Element>()
    }
}
