/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::window_from_node;
use crate::dom::window::ReflowReason;
use script_layout_interface::message::ReflowGoal;

/// Trait for elements with defined activation behavior
pub trait Activatable {
    fn as_element(&self) -> &Element;

    // Is this particular instance of the element activatable?
    fn is_instance_activatable(&self) -> bool;

    // https://html.spec.whatwg.org/multipage/#run-pre-click-activation-steps
    fn pre_click_activation(&self);

    // https://html.spec.whatwg.org/multipage/#run-canceled-activation-steps
    fn canceled_activation(&self);

    // https://html.spec.whatwg.org/multipage/#run-post-click-activation-steps
    fn activation_behavior(&self, event: &Event, target: &EventTarget);

    // https://html.spec.whatwg.org/multipage/#implicit-submission
    fn implicit_submission(&self, ctrl_key: bool, shift_key: bool, alt_key: bool, meta_key: bool);

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

//https://html.spec.whatwg.org/multipage/#fire-a-synthetic-mouse-event
pub fn synthetic_click_activation(
    element: &Element,
    ctrl_key: bool,
    shift_key: bool,
    alt_key: bool,
    meta_key: bool,
    not_trusted: bool,
) {
    // Step 12 of https://dom.spec.whatwg.org/#concept-event-dispatch
    let activatable = element.as_maybe_activatable();
    if let Some(a) = activatable {
        a.pre_click_activation();
    }

    // https://html.spec.whatwg.org/multipage/#fire-a-synthetic-mouse-event
    let win = window_from_node(element);
    let target = element.upcast::<EventTarget>();
    //Step 1
    let mouse = MouseEvent::new(
        &win,
        //Step 2
        DOMString::from("click"),
        //Step 3 & 4
        EventBubbles::Bubbles,
        EventCancelable::Cancelable,
        Some(&win),
        1,
        0,
        0,
        0,
        0,
        ctrl_key,
        shift_key,
        alt_key,
        meta_key,
        0,
        None,
        None,
    );
    //Step 5
    let event = mouse.upcast::<Event>();
    if not_trusted {
        event.set_trusted(false);
    }
    //Step 9
    target.dispatch_event(event);
    event.dispatching();
    if let Some(a) = activatable {
        if event.DefaultPrevented() {
            a.canceled_activation();
        } else {
            // post click activation
            a.activation_behavior(event, target);
        }
    }
    element.set_click_in_progress(false);
}
