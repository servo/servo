/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::domstring::DOMString;
use script_bindings::inheritance::Castable;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventComposed};
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlinputelement::text_value_widget::TextValueWidget;
use crate::dom::input_element::input_type::SpecificInputType;
use crate::dom::input_element::{HTMLInputElement, InputActivationState};
use crate::dom::node::Node;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct CheckboxInputType {
    text_value_widget: DomRefCell<TextValueWidget>,
}

impl SpecificInputType for CheckboxInputType {
    /// <https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox):suffering-from-being-missing>
    fn suffers_from_being_missing(&self, input: &HTMLInputElement, _value: &DOMString) -> bool {
        input.Required() && !input.Checked()
    }

    /// <https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox):input-activation-behavior>
    fn activation_behavior(
        &self,
        input: &HTMLInputElement,
        _event: &Event,
        _target: &EventTarget,
        can_gc: CanGc,
    ) {
        // Step 1: If the element is not connected, then return.
        if !input.upcast::<Node>().is_connected() {
            return;
        }

        let target = input.upcast::<EventTarget>();

        // Step 2: Fire an event named input at the element with the bubbles and composed
        // attributes initialized to true.
        target.fire_event_with_params(
            atom!("input"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            EventComposed::Composed,
            can_gc,
        );

        // Step 3: Fire an event named change at the element with the bubbles attribute
        // initialized to true.
        target.fire_bubbling_event(atom!("change"), can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-input-element:legacy-pre-activation-behavior>
    fn legacy_pre_activation_behavior(
        &self,
        input: &HTMLInputElement,
        can_gc: CanGc,
    ) -> Option<InputActivationState> {
        let was_checked = input.Checked();
        let was_indeterminate = input.Indeterminate();
        input.SetIndeterminate(false);
        input.SetChecked(!was_checked, can_gc);
        Some(InputActivationState {
            checked: was_checked,
            indeterminate: was_indeterminate,
            checked_radio: None,
            was_radio: false,
            was_checkbox: true,
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#the-input-element:legacy-canceled-activation-behavior>
    fn legacy_canceled_activation_behavior(
        &self,
        input: &HTMLInputElement,
        cache: InputActivationState,
        can_gc: CanGc,
    ) {
        input.SetIndeterminate(cache.indeterminate);
        input.SetChecked(cache.checked, can_gc);
    }

    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.text_value_widget
            .borrow()
            .update_shadow_tree(cx, input)
    }
}
