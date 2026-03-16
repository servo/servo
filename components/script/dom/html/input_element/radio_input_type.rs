/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use html5ever::local_name;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::domstring::DOMString;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use stylo_atoms::Atom;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::NodeBinding::GetRootNodeOptions;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::element::AttributeMutation;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventComposed};
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::htmlinputelement::input_type::InputType;
use crate::dom::htmlinputelement::text_value_widget::TextValueWidget;
use crate::dom::input_element::input_type::SpecificInputType;
use crate::dom::input_element::{HTMLInputElement, InputActivationState};
use crate::dom::node::{BindContext, Node, ShadowIncluding, UnbindContext};
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidationFlags;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct RadioInputType {
    text_value_widget: DomRefCell<TextValueWidget>,
}

impl SpecificInputType for RadioInputType {
    /// <https://html.spec.whatwg.org/multipage/#radio-button-state-(type=radio):suffering-from-being-missing>
    fn suffers_from_being_missing(&self, input: &HTMLInputElement, _value: &DOMString) -> bool {
        if input.radio_group_name().is_none() {
            return false;
        }
        let mut is_required = input.Required();
        let mut is_checked = input.Checked();
        let root = input
            .upcast::<Node>()
            .GetRootNode(&GetRootNodeOptions::empty());
        let form = input.form_owner();
        for other in radio_group_iter(
            input,
            input.radio_group_name().as_ref(),
            form.as_deref(),
            &root,
        ) {
            is_required = is_required || other.Required();
            is_checked = is_checked || other.Checked();
        }
        is_required && !is_checked
    }

    /// <https://html.spec.whatwg.org/multipage/#radio-button-state-(type=radio):signal-a-type-change>
    fn signal_type_change(&self, input: &HTMLInputElement, can_gc: CanGc) {
        radio_group_updated(input, input.radio_group_name().as_ref(), can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#radio-button-state-(type=radio):input-activation-behavior>
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
        let root = input
            .upcast::<Node>()
            .GetRootNode(&GetRootNodeOptions::empty());
        let form_owner = input.form_owner();
        let checked_member = radio_group_iter(
            input,
            input.radio_group_name().as_ref(),
            form_owner.as_deref(),
            &root,
        )
        .find(|r| r.Checked());
        let was_checked = input.Checked();
        input.SetChecked(true, can_gc);
        Some(InputActivationState {
            checked: was_checked,
            indeterminate: false,
            checked_radio: checked_member.as_deref().map(DomRoot::from_ref),
            was_radio: true,
            was_checkbox: false,
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#the-input-element:legacy-canceled-activation-behavior>
    fn legacy_canceled_activation_behavior(
        &self,
        input: &HTMLInputElement,
        cache: InputActivationState,
        can_gc: CanGc,
    ) {
        if let Some(ref o) = cache.checked_radio {
            let tree_root = input
                .upcast::<Node>()
                .GetRootNode(&GetRootNodeOptions::empty());
            // Avoiding iterating through the whole tree here, instead
            // we can check if the conditions for radio group siblings apply
            if in_same_group(
                o,
                input.form_owner().as_deref(),
                input.radio_group_name().as_ref(),
                Some(&*tree_root),
            ) {
                o.SetChecked(true, can_gc);
            } else {
                input.SetChecked(false, can_gc);
            }
        } else {
            input.SetChecked(false, can_gc);
        }
    }

    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.text_value_widget
            .borrow()
            .update_shadow_tree(cx, input)
    }

    fn attribute_mutated(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
        attr: &Attr,
        mutation: AttributeMutation,
    ) {
        match *attr.local_name() {
            local_name!("name") => radio_group_updated(
                input,
                mutation.new_value(attr).as_ref().map(|name| name.as_atom()),
                CanGc::from_cx(cx),
            ),
            _ => {},
        }
    }

    fn bind_to_tree(&self, cx: &mut JSContext, input: &HTMLInputElement, _context: &BindContext) {
        radio_group_updated(input, input.radio_group_name().as_ref(), CanGc::from_cx(cx));
    }

    fn unbind_from_tree(
        &self,
        input: &HTMLInputElement,
        form_owner: Option<DomRoot<HTMLFormElement>>,
        context: &UnbindContext,
        can_gc: CanGc,
    ) {
        let root = context.parent.GetRootNode(&GetRootNodeOptions::empty());
        for r in radio_group_iter(
            input,
            input.radio_group_name().as_ref(),
            form_owner.as_deref(),
            &root,
        ) {
            r.validity_state(can_gc)
                .perform_validation_and_update(ValidationFlags::all(), can_gc);
        }
    }
}

fn radio_group_updated(input: &HTMLInputElement, group: Option<&Atom>, can_gc: CanGc) {
    if input.Checked() {
        broadcast_radio_checked(input, group, can_gc);
    }
}

pub(crate) fn perform_radio_group_validation(
    elem: &HTMLInputElement,
    group: Option<&Atom>,
    can_gc: CanGc,
) {
    let root = elem
        .upcast::<Node>()
        .GetRootNode(&GetRootNodeOptions::empty());
    let form = elem.form_owner();
    for r in radio_group_iter(elem, group, form.as_deref(), &root) {
        r.validity_state(can_gc)
            .perform_validation_and_update(ValidationFlags::all(), can_gc);
    }
}

pub(crate) fn radio_group_iter<'a>(
    elem: &'a HTMLInputElement,
    group: Option<&'a Atom>,
    form: Option<&'a HTMLFormElement>,
    root: &'a Node,
) -> impl Iterator<Item = DomRoot<HTMLInputElement>> + 'a {
    root.traverse_preorder(ShadowIncluding::No)
        .filter_map(DomRoot::downcast::<HTMLInputElement>)
        .filter(move |r| &**r == elem || in_same_group(r, form, group, Some(root)))
}

pub(crate) fn broadcast_radio_checked(
    broadcaster: &HTMLInputElement,
    group: Option<&Atom>,
    can_gc: CanGc,
) {
    let root = broadcaster
        .upcast::<Node>()
        .GetRootNode(&GetRootNodeOptions::empty());
    let form = broadcaster.form_owner();
    for r in radio_group_iter(broadcaster, group, form.as_deref(), &root) {
        if broadcaster != &*r && r.Checked() {
            r.SetChecked(false, can_gc);
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#radio-button-group>
pub(crate) fn in_same_group(
    other: &HTMLInputElement,
    owner: Option<&HTMLFormElement>,
    group: Option<&Atom>,
    tree_root: Option<&Node>,
) -> bool {
    if group.is_none() {
        // Radio input elements with a missing or empty name are alone in their own group.
        return false;
    }

    if !matches!(*other.input_type(), InputType::Radio(_)) ||
        other.form_owner().as_deref() != owner ||
        other.radio_group_name().as_ref() != group
    {
        return false;
    }

    match tree_root {
        Some(tree_root) => {
            let other_root = other
                .upcast::<Node>()
                .GetRootNode(&GetRootNodeOptions::empty());
            tree_root == &*other_root
        },
        None => {
            // Skip check if the tree root isn't provided.
            true
        },
    }
}
