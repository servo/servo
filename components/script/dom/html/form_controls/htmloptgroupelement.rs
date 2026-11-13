/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::str::DOMString;
use stylo_dom::ElementState;

use crate::dom::MoveContext;
use crate::dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding::HTMLOptGroupElementMethods;
use crate::dom::bindings::codegen::GenericBindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::element::attributes::storage::AttrRef;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmloptionelement::HTMLOptionElement;
use crate::dom::html::htmlselectelement::HTMLSelectElement;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::node::{BindContext, Node, UnbindContext};
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidationFlags;

/// <https://html.spec.whatwg.org/multipage/#htmloptgroupelement>
#[dom_struct]
pub(crate) struct HTMLOptGroupElement {
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

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLOptGroupElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLOptGroupElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    fn update_select_validity(&self, cx: &mut JSContext) {
        if let Some(select) = self.owner_select_element() {
            select
                .validity_state(cx)
                .perform_validation_and_update(cx, ValidationFlags::all());
        }
    }

    /// Like the [option element nearest ancestor select](https://html.spec.whatwg.org/multipage/#option-element-nearest-ancestor-select),
    /// but an ancestor optgroup also means there is none.
    pub(crate) fn nearest_ancestor_select(&self) -> Option<DomRoot<HTMLSelectElement>> {
        for ancestor in self.upcast::<Node>().ancestors() {
            let Some(element) = ancestor.downcast::<Element>() else {
                continue;
            };
            if matches!(
                element.local_name(),
                &local_name!("datalist") |
                    &local_name!("hr") |
                    &local_name!("option") |
                    &local_name!("optgroup")
            ) {
                return None;
            }
            if let Some(select) = DomRoot::downcast::<HTMLSelectElement>(ancestor) {
                return Some(select);
            }
        }
        None
    }

    fn owner_select_element(&self) -> Option<DomRoot<HTMLSelectElement>> {
        self.upcast::<Node>()
            .GetParentNode()
            .and_then(DomRoot::downcast)
    }
}

impl HTMLOptGroupElementMethods<crate::DomTypeHolder> for HTMLOptGroupElement {
    // https://html.spec.whatwg.org/multipage/#dom-optgroup-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-optgroup-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-optgroup-label
    make_getter!(Label, "label");

    // https://html.spec.whatwg.org/multipage/#dom-optgroup-label
    make_setter!(SetLabel, "label");
}

impl VirtualMethods for HTMLOptGroupElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: AttrRef<'_>,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);
        if attr.local_name() == &local_name!("disabled") {
            let options = self
                .upcast::<Node>()
                .traverse_preorder(ShadowIncluding::No)
                .filter_map(DomRoot::downcast::<HTMLOptionElement>);
            // The disabledness of descendant option elements might have changed.
            for option in options {
                option.upcast::<Element>().invalidate_disabled_state(cx);
            }
        }
    }

    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        if let Some(super_type) = self.super_type() {
            super_type.bind_to_tree(cx, context);
        }
        self.update_select_validity(cx);
    }

    fn moving_steps(&self, cx: &mut JSContext, context: &MoveContext) {
        if let Some(super_type) = self.super_type() {
            super_type.moving_steps(cx, context);
        }
        self.update_select_validity(cx);
    }

    fn unbind_from_tree(&self, cx: &mut js::context::JSContext, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(cx, context);

        if let Some(select) = context.parent.downcast::<HTMLSelectElement>() {
            select
                .validity_state(cx)
                .perform_validation_and_update(cx, ValidationFlags::all());
        }
    }
}
