// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use dom::bindings::codegen::Bindings::HTMLLegendElementBinding;
use dom::bindings::codegen::Bindings::HTMLLegendElementBinding::HTMLLegendElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::document::Document;
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlformelement::{HTMLFormElement, FormControl};
use dom::node::{Node, UnbindContext};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLLegendElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
    form_owner: MutNullableDom<HTMLFormElement<TH>>,
}

impl<TH: TypeHolderTrait> HTMLLegendElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>)
                     -> HTMLLegendElement<TH> {
        HTMLLegendElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>)
               -> DomRoot<HTMLLegendElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLLegendElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLLegendElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLLegendElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.upcast::<Element<TH>>().check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, context: &UnbindContext<TH>) {
        self.super_type().unwrap().unbind_from_tree(context);

        let node = self.upcast::<Node<TH>>();
        let el = self.upcast::<Element<TH>>();
        if node.ancestors().any(|ancestor| ancestor.is::<HTMLFieldSetElement<TH>>()) {
            el.check_ancestors_disabled_state_for_form_control();
        } else {
            el.check_disabled_attribute();
        }
    }
}


impl<TH: TypeHolderTrait> HTMLLegendElementMethods<TH> for HTMLLegendElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-legend-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement<TH>>> {
        let parent = self.upcast::<Node<TH>>().GetParentElement()?;
        if parent.is::<HTMLFieldSetElement<TH>>() {
            return self.form_owner();
        }
        None
    }
}

impl<TH: TypeHolderTrait> FormControl<TH> for HTMLLegendElement<TH> {
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
