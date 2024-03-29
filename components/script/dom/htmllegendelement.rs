// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLLegendElementBinding::HTMLLegendElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::node::{BindContext, Node, UnbindContext};
use crate::dom::virtualmethods::VirtualMethods;

#[dom_struct]
pub struct HTMLLegendElement {
    htmlelement: HTMLElement,
    form_owner: MutNullableDom<HTMLFormElement>,
}

impl HTMLLegendElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLLegendElement {
        HTMLLegendElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            form_owner: Default::default(),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLLegendElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLLegendElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }
}

impl VirtualMethods for HTMLLegendElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }

        self.upcast::<Element>()
            .check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        let node = self.upcast::<Node>();
        let el = self.upcast::<Element>();
        if node
            .ancestors()
            .any(|ancestor| ancestor.is::<HTMLFieldSetElement>())
        {
            el.check_ancestors_disabled_state_for_form_control();
        } else {
            el.check_disabled_attribute();
        }
    }
}

impl HTMLLegendElementMethods for HTMLLegendElement {
    // https://html.spec.whatwg.org/multipage/#dom-legend-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        let parent = self.upcast::<Node>().GetParentElement()?;
        if parent.is::<HTMLFieldSetElement>() {
            return self.form_owner();
        }
        None
    }
}

impl FormControl for HTMLLegendElement {
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
