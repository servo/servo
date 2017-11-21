/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::conversions::DerivedFrom;
use dom::bindings::str::DOMString;
use dom::event::{EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::node::{Node, NodeDamage, window_from_node};
use script_traits::ScriptToConstellationChan;
use textinput::{SelectionDirection, TextInput};

pub trait TextControl: DerivedFrom<EventTarget> + DerivedFrom<Node> {
    fn textinput(&self) -> &DomRefCell<TextInput<ScriptToConstellationChan>>;

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    fn dom_selection_start(&self) -> u32 {
        self.textinput().borrow().get_selection_start()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    fn set_dom_selection_start(&self, start: u32) {
        self.set_selection_range(start, self.dom_selection_end(), self.selection_direction());
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn dom_selection_end(&self) -> u32 {
        self.textinput().borrow().get_absolute_insertion_point() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn set_dom_selection_end(&self, end: u32) {
        self.set_selection_range(self.dom_selection_start(), end, self.selection_direction());
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn dom_selection_direction(&self) -> DOMString {
        DOMString::from(self.selection_direction())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn set_dom_selection_direction(&self, direction: DOMString) {
        self.textinput().borrow_mut().selection_direction = SelectionDirection::from(direction);
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setselectionrange
    fn set_dom_selection_range(&self, start: u32, end: u32, direction: Option<DOMString>) {
        // Step 4
        let direction = direction.map_or(SelectionDirection::None, |d| SelectionDirection::from(d));

        self.set_selection_range(start, end, direction);
    }

    fn selection_direction(&self) -> SelectionDirection {
        self.textinput().borrow().selection_direction
    }

    // https://html.spec.whatwg.org/multipage/#set-the-selection-range
    fn set_selection_range(&self, start: u32, end: u32, direction: SelectionDirection) {
        // Step 5
        self.textinput().borrow_mut().selection_direction = direction;

        // Step 3
        self.textinput().borrow_mut().set_selection_range(start, end);

        // Step 6
        let window = window_from_node(self);
        let _ = window.user_interaction_task_source().queue_event(
            &self.upcast::<EventTarget>(),
            atom!("select"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            &window);

        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }
}
