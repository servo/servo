/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::context::{JSContext, NoGC};
use js::rust::HandleObject;
use stylo_dom::ElementState;

use crate::dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding::HTMLFieldSetElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::attributes::storage::AttrRef;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::html::htmlcollection::HTMLCollection;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::html::htmllegendelement::HTMLLegendElement;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::node::{ChildrenMutation, Node, NodeTraits};
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidityState;

#[dom_struct]
pub(crate) struct HTMLFieldSetElement {
    htmlelement: HTMLElement,
    form_owner: MutNullableDom<HTMLFormElement>,
    validity_state: MutNullableDom<ValidityState>,
    /// The first [`HTMLLegendElement`] child as of the last children change. Descendants of it
    /// are not disabled by this fieldset, so they must be updated when it changes.
    first_legend: MutNullableDom<HTMLLegendElement>,
}

impl HTMLFieldSetElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLFieldSetElement {
        HTMLFieldSetElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::ENABLED | ElementState::VALID,
                local_name,
                prefix,
                document,
            ),
            form_owner: Default::default(),
            validity_state: Default::default(),
            first_legend: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLFieldSetElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLFieldSetElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    pub(crate) fn update_validity(&self, cx: &mut JSContext) {
        let has_invalid_child = self
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
            .flat_map(DomRoot::downcast::<Element>)
            .any(|element| element.is_invalid(cx, false));

        self.upcast::<Element>()
            .set_state(cx.no_gc(), ElementState::VALID, !has_invalid_child);
        self.upcast::<Element>()
            .set_state(cx.no_gc(), ElementState::INVALID, has_invalid_child);
    }
}

impl HTMLFieldSetElementMethods<crate::DomTypeHolder> for HTMLFieldSetElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-fieldset-elements>
    fn Elements(&self, cx: &mut js::context::JSContext) -> DomRoot<HTMLCollection> {
        HTMLCollection::new_with_filter_fn(cx, &self.owner_window(), self.upcast(), |element, _| {
            element
                .downcast::<HTMLElement>()
                .is_some_and(HTMLElement::is_listed_element)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-fieldset-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fieldset-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_getter!(Name, "name");

    /// <https://html.spec.whatwg.org/multipage/#dom-fae-form>
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-willvalidate>
    fn WillValidate(&self, no_gc: &NoGC) -> bool {
        self.is_instance_validatable(no_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-validity>
    fn Validity(&self, cx: &mut JSContext) -> DomRoot<ValidityState> {
        self.validity_state(cx)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-checkvalidity>
    fn CheckValidity(&self, cx: &mut JSContext) -> bool {
        self.check_validity(cx)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-reportvalidity>
    fn ReportValidity(&self, cx: &mut JSContext) -> bool {
        self.report_validity(cx)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-validationmessage>
    fn ValidationMessage(&self, cx: &mut JSContext) -> DOMString {
        self.validation_message(cx)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-setcustomvalidity>
    fn SetCustomValidity(&self, cx: &mut JSContext, error: DOMString) {
        self.validity_state(cx).set_custom_error_message(cx, error);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-fieldset-type>
    fn Type(&self) -> DOMString {
        DOMString::from_static("fieldset")
    }
}

impl HTMLFieldSetElement {
    /// <https://html.spec.whatwg.org/multipage/#concept-fieldset-disabled>
    pub(crate) fn is_fieldset_disabled(&self, no_gc: &NoGC) -> bool {
        // > A fieldset element is a disabled fieldset if it matches any of the following
        // > conditions:
        // > - Its disabled attribute is specified
        // > - It is a descendant of another fieldset element whose disabled attribute is
        // >   specified, and is not a descendant of that fieldset element's first legend element
        // >   child, if any.
        let element = self.upcast::<Element>();
        element.has_attribute(&local_name!("disabled")) ||
            element.is_disabled_by_ancestor_fieldset(no_gc)
    }
}

impl VirtualMethods for HTMLFieldSetElement {
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
        match *attr.local_name() {
            local_name!("disabled") => {
                if !matches!(mutation, AttributeMutation::Set(Some(_))) {
                    // The disabledness of descendant optgroup and option elements might have changed.
                    self.upcast::<Element>()
                        .invalidate_descendants_disabled_state(cx);
                }
            },
            local_name!("form") => {
                self.form_attribute_mutated(cx, mutation);
            },
            _ => {},
        }
    }

    fn children_changed(&self, cx: &mut JSContext, mutation: &ChildrenMutation) {
        if let Some(super_type) = self.super_type() {
            super_type.children_changed(cx, mutation);
        }

        let first_legend = self
            .upcast::<Node>()
            .children()
            .find_map(DomRoot::downcast::<HTMLLegendElement>);
        if first_legend == self.first_legend.get() {
            return;
        }
        self.first_legend.set(first_legend.as_deref());

        let element = self.upcast::<Element>();
        if element.has_attribute(&local_name!("disabled")) {
            // A legend element might have been inserted before the first legend element, causing
            // the elements in the now second legend element to become disabled.
            element.invalidate_descendants_disabled_state(cx);
        }
    }
}

impl FormControl for HTMLFieldSetElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, _cx: &mut JSContext, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_html_element(&self) -> &HTMLElement {
        self.upcast::<HTMLElement>()
    }
}

impl Validatable for HTMLFieldSetElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn validity_state(&self, cx: &mut JSContext) -> DomRoot<ValidityState> {
        self.validity_state
            .or_init(|| ValidityState::new(cx, &self.owner_window(), self.upcast()))
    }

    fn satisfies_constraints(&self) -> bool {
        let Some(validity_state) = self.validity_state.get() else {
            return true;
        };
        validity_state.invalid_flags().is_empty()
    }

    fn is_instance_validatable(&self, _no_gc: &NoGC) -> bool {
        // fieldset is not a submittable element (https://html.spec.whatwg.org/multipage/#category-submit)
        false
    }
}
