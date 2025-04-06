/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::element::Element;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlinputelement::InputActivationState;
use crate::script_runtime::CanGc;

/// Trait for elements with defined activation behavior
pub(crate) trait Activatable {
    fn as_element(&self) -> &Element;

    // Is this particular instance of the element activatable?
    fn is_instance_activatable(&self) -> bool;

    // https://dom.spec.whatwg.org/#eventtarget-legacy-pre-activation-behavior
    fn legacy_pre_activation_behavior(&self, _can_gc: CanGc) -> Option<InputActivationState> {
        None
    }

    // https://dom.spec.whatwg.org/#eventtarget-legacy-canceled-activation-behavior
    fn legacy_canceled_activation_behavior(
        &self,
        _state_before: Option<InputActivationState>,
        _can_gc: CanGc,
    ) {
    }

    // https://dom.spec.whatwg.org/#eventtarget-activation-behavior
    // event and target are used only by HTMLAnchorElement, in the case
    // where the target is an <img ismap> so the href gets coordinates appended
    fn activation_behavior(&self, event: &Event, target: &EventTarget, can_gc: CanGc);

    // https://html.spec.whatwg.org/multipage/#concept-selector-active
    fn enter_formal_activation_state(&self) {
        self.as_element().set_active_state(true);
    }

    fn exit_formal_activation_state(&self) {
        self.as_element().set_active_state(false);
    }
}
