/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use js::context::JSContext;
use script_bindings::domstring::DOMString;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlformelement::{FormControl, FormSubmitterElement, SubmittedFrom};
use crate::dom::htmlinputelement::text_value_widget::TextValueWidget;
use crate::dom::input_element::HTMLInputElement;
use crate::dom::input_element::input_type::SpecificInputType;
use crate::dom::node::NodeTraits;

const DEFAULT_SUBMIT_VALUE: &str = "Submit";

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct SubmitInputType {
    text_value_widget: DomRefCell<TextValueWidget>,
}

impl SpecificInputType for SubmitInputType {
    fn value_for_shadow_dom(&self, _input: &HTMLInputElement) -> DOMString {
        DEFAULT_SUBMIT_VALUE.into()
    }

    /// <https://html.spec.whatwg.org/multipage/#submit-button-state-(type=submit):input-activation-behavior>
    fn activation_behavior(
        &self,
        input: &HTMLInputElement,
        _event: &Event,
        _target: &EventTarget,
        can_gc: CanGc,
    ) {
        // Step 1: If the element does not have a form owner, then return.
        if let Some(form_owner) = input.form_owner() {
            let document = input.owner_document();

            // Step 2: If the element's node document is not fully active, then return.
            if !document.is_fully_active() {
                return;
            }

            // Step 3: Submit the element's form owner from the element with userInvolvement
            // set to event's user navigation involvement.
            form_owner.submit(
                SubmittedFrom::NotFromForm,
                FormSubmitterElement::Input(input),
                can_gc,
            )
        }
    }

    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.text_value_widget
            .borrow()
            .update_shadow_tree(cx, input)
    }
}
