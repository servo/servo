/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_layout_interface::ReflowGoal;

use crate::dom::element::Element;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlinputelement::InputActivationState;
use crate::dom::node::window_from_node;
use crate::dom::window::ReflowReason;

/// Trait for elements with defined activation behavior
pub trait Activatable {
    fn as_element(&self) -> &Element;

    // Is this particular instance of the element activatable?
    fn is_instance_activatable(&self) -> bool;

    // https://dom.spec.whatwg.org/#eventtarget-legacy-pre-activation-behavior
    fn legacy_pre_activation_behavior(&self) -> Option<InputActivationState> {
        None
    }

    // https://dom.spec.whatwg.org/#eventtarget-legacy-canceled-activation-behavior
    fn legacy_canceled_activation_behavior(&self, _state_before: Option<InputActivationState>) {}

    // https://dom.spec.whatwg.org/#eventtarget-activation-behavior
    // event and target are used only by HTMLAnchorElement, in the case
    // where the target is an <img ismap> so the href gets coordinates appended
    fn activation_behavior(&self, event: &Event, target: &EventTarget);

    // https://html.spec.whatwg.org/multipage/#concept-selector-active
    fn enter_formal_activation_state(&self) {
        self.as_element().set_active_state(true);

        let win = window_from_node(self.as_element());
        win.reflow(ReflowGoal::Full, ReflowReason::ElementStateChanged);
    }

    fn exit_formal_activation_state(&self) {
        self.as_element().set_active_state(false);

        let win = window_from_node(self.as_element());
        win.reflow(ReflowGoal::Full, ReflowReason::ElementStateChanged);
    }
}
