/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast};
use dom::bindings::js::{JSRef, Temporary, OptionalRootable};
use dom::element::{Element, ActivationElementHelpers};
use dom::event::{Event, EventHelpers};
use dom::eventtarget::{EventTarget, EventTargetHelpers};
use dom::mouseevent::MouseEvent;
use dom::node::window_from_node;

use std::borrow::ToOwned;

/// Trait for elements with defined activation behavior
pub trait Activatable : Copy {
    fn as_element(&self) -> Temporary<Element>;

    // https://html.spec.whatwg.org/multipage/interaction.html#run-pre-click-activation-steps
    fn pre_click_activation(&self);

    // https://html.spec.whatwg.org/multipage/interaction.html#run-canceled-activation-steps
    fn canceled_activation(&self);

    // https://html.spec.whatwg.org/multipage/interaction.html#run-post-click-activation-steps
    fn activation_behavior(&self);

    // https://html.spec.whatwg.org/multipage/forms.html#implicit-submission
    fn implicit_submission(&self, ctrlKey: bool, shiftKey: bool, altKey: bool, metaKey: bool);

    // https://html.spec.whatwg.org/multipage/interaction.html#run-synthetic-click-activation-steps
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
        // https://html.spec.whatwg.org/multipage/webappapis.html#fire-a-synthetic-mouse-event
        let win = window_from_node(element.r()).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(element.r());
        let mouse = MouseEvent::new(win.r(), "click".to_owned(),
                                    false, false, Some(win.r()), 1,
                                    0, 0, 0, 0, ctrlKey, shiftKey, altKey, metaKey,
                                    0, None).root();
        let event: JSRef<Event> = EventCast::from_ref(mouse.r());
        event.set_trusted(true);
        target.dispatch_event(event);

        // Step 5
        if event.DefaultPrevented() {
            self.canceled_activation();
        } else {
            // post click activation
            self.activation_behavior();
        }

        // Step 6
        element.r().set_click_in_progress(false);
    }
}
