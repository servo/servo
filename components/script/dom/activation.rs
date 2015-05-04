/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast};
use dom::bindings::js::{JSRef, Temporary, OptionalRootable, Rootable};
use dom::element::{Element, ActivationElementHelpers};
use dom::event::{Event, EventHelpers, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::mouseevent::MouseEvent;
use dom::node::window_from_node;

use std::borrow::ToOwned;

/// Trait for elements with defined activation behavior
pub trait Activatable {
    fn as_element(&self) -> Temporary<Element>;

    // Is this particular instance of the element activatable?
    fn is_instance_activatable(&self) -> bool;

    // https://html.spec.whatwg.org/multipage/#run-pre-click-activation-steps
    fn pre_click_activation(&self);

    // https://html.spec.whatwg.org/multipage/#run-canceled-activation-steps
    fn canceled_activation(&self);

    // https://html.spec.whatwg.org/multipage/#run-post-click-activation-steps
    fn activation_behavior(&self, event: JSRef<Event>, target: JSRef<EventTarget>);

    // https://html.spec.whatwg.org/multipage/#implicit-submission
    fn implicit_submission(&self, ctrlKey: bool, shiftKey: bool, altKey: bool, metaKey: bool);

    // https://html.spec.whatwg.org/multipage/#run-synthetic-click-activation-steps
    fn synthetic_click_activation(&self, ctrlKey: bool, shiftKey: bool, altKey: bool, metaKey: bool) {
        let element = self.as_element().root();
        // Step 1
        if element.r().click_in_progress() {
            return;
        }
        // Step 2
        element.r().set_click_in_progress(true);
        // Step 3
        self.pre_click_activation();

        // Step 4
        // https://html.spec.whatwg.org/multipage/#fire-a-synthetic-mouse-event
        let win = window_from_node(element.r()).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(element.r());
        let mouse = MouseEvent::new(win.r(), "click".to_owned(),
                                    EventBubbles::DoesNotBubble, EventCancelable::NotCancelable, Some(win.r()), 1,
                                    0, 0, 0, 0, ctrlKey, shiftKey, altKey, metaKey,
                                    0, None).root();
        let event: JSRef<Event> = EventCast::from_ref(mouse.r());
        event.fire(target);

        // Step 5
        if event.DefaultPrevented() {
            self.canceled_activation();
        } else {
            // post click activation
            self.activation_behavior(event, target);
        }

        // Step 6
        element.r().set_click_in_progress(false);
    }
}
