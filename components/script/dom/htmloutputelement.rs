/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLOutputElementBinding;
use dom::bindings::codegen::Bindings::HTMLOutputElementBinding::HTMLOutputElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::{FormControl, HTMLFormElement};
use dom::node::{Node, UnbindContext, window_from_node};
use dom::nodelist::NodeList;
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;
use html5ever_atoms::LocalName;

#[dom_struct]
pub struct HTMLOutputElement {
    htmlelement: HTMLElement,
    form_owner: MutNullableJS<HTMLFormElement>,
}

impl HTMLOutputElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLOutputElement {
        HTMLOutputElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLOutputElement> {
        Node::reflect_node(box HTMLOutputElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLOutputElementBinding::Wrap)
    }
}

impl HTMLOutputElementMethods for HTMLOutputElement {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(&window, self.upcast())
    }

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> Root<NodeList> {
        self.upcast::<HTMLElement>().labels()
    }
}

impl VirtualMethods for HTMLOutputElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("form") => {
                match mutation {
                    AttributeMutation::Set(_) => {
                        self.form_attribute_set();
                    },
                    AttributeMutation::Removed => {
                         self.form_attribute_removed();
                    },
                }
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.bind_form_control_to_tree();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);
        self.unbind_form_control_from_tree();
    }
}

impl FormControl for HTMLOutputElement {
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
