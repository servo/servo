/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLOutputElementBinding;
use dom::bindings::codegen::Bindings::HTMLOutputElementBinding::HTMLOutputElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::{FormControl, HTMLFormElement};
use dom::node::{Node, window_from_node};
use dom::nodelist::NodeList;
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLOutputElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
    form_owner: MutNullableDom<HTMLFormElement<TH>>,
}

impl<TH: TypeHolderTrait> HTMLOutputElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLOutputElement<TH> {
        HTMLOutputElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLOutputElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLOutputElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLOutputElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLOutputElementMethods<TH> for HTMLOutputElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> DomRoot<ValidityState<TH>> {
        let window = window_from_node(self);
        ValidityState::new(&window, self.upcast())
    }

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement<TH>>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> DomRoot<NodeList<TH>> {
        self.upcast::<HTMLElement<TH>>().labels()
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLOutputElement<TH> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn attribute_mutated(&self, attr: &Attr<TH>, mutation: AttributeMutation<TH>) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("form") => {
                self.form_attribute_mutated(mutation);
            },
            _ => {},
        }
    }
}

impl<TH: TypeHolderTrait> FormControl<TH> for HTMLOutputElement<TH> {
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
