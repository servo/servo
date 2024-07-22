/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, LocalName, Prefix};
use js::rust::HandleObject;
use style_dom::ElementState;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding::HTMLOptGroupElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmloptionelement::HTMLOptionElement;
use crate::dom::htmlselectelement::HTMLSelectElement;
use crate::dom::node::{BindContext, Node, ShadowIncluding, UnbindContext};
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidationFlags;
use crate::dom::virtualmethods::VirtualMethods;

#[dom_struct]
pub struct HTMLOptGroupElement {
    htmlelement: HTMLElement,
}

impl HTMLOptGroupElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLOptGroupElement {
        HTMLOptGroupElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::ENABLED,
                local_name,
                prefix,
                document,
            ),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLOptGroupElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLOptGroupElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    fn update_select_validity(&self) {
        if let Some(select) = self
            .upcast::<Node>()
            .ancestors()
            .filter_map(DomRoot::downcast::<HTMLSelectElement>)
            .next()
        {
            select
                .validity_state()
                .perform_validation_and_update(ValidationFlags::all());
        }
    }
}

impl HTMLOptGroupElementMethods for HTMLOptGroupElement {
    // https://html.spec.whatwg.org/multipage/#dom-optgroup-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-optgroup-disabled
    make_bool_setter!(SetDisabled, "disabled");
}

impl VirtualMethods for HTMLOptGroupElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        if attr.local_name() == &local_name!("disabled") {
            let disabled_state = match mutation {
                AttributeMutation::Set(None) => true,
                AttributeMutation::Set(Some(_)) => {
                    // Option group was already disabled.
                    return;
                },
                AttributeMutation::Removed => false,
            };
            let el = self.upcast::<Element>();
            el.set_disabled_state(disabled_state);
            el.set_enabled_state(!disabled_state);
            let options = el
                .upcast::<Node>()
                .children()
                .filter(|child| child.is::<HTMLOptionElement>())
                .map(|child| DomRoot::from_ref(child.downcast::<HTMLOptionElement>().unwrap()));
            if disabled_state {
                for option in options {
                    let el = option.upcast::<Element>();
                    el.set_disabled_state(true);
                    el.set_enabled_state(false);
                }
            } else {
                for option in options {
                    let el = option.upcast::<Element>();
                    el.check_disabled_attribute();
                }
            }
        }
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }

        self.update_select_validity();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        if let Some(select) = context
            .parent
            .inclusive_ancestors(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<HTMLSelectElement>)
            .next()
        {
            select
                .validity_state()
                .perform_validation_and_update(ValidationFlags::all());
        }
    }
}
