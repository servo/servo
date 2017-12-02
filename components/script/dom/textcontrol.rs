/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::conversions::DerivedFrom;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::str::DOMString;
use dom::event::{EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::node::{Node, NodeDamage, window_from_node};
use script_traits::ScriptToConstellationChan;
use textinput::{SelectionDirection, TextInput};

pub trait TextControl: DerivedFrom<EventTarget> + DerivedFrom<Node> {
    fn textinput(&self) -> &DomRefCell<TextInput<ScriptToConstellationChan>>;
    fn selection_api_applies(&self) -> bool;

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    fn get_dom_selection_start(&self) -> Option<u32> {
        // Step 1
        if !self.selection_api_applies() {
            return None;
        }

        // Steps 2-3
        Some(self.selection_start())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    fn set_dom_selection_start(&self, start: Option<u32>) -> ErrorResult {
        // Step 1
        if !self.selection_api_applies() {
            return Err(Error::InvalidState);
        }

        // Step 2
        let mut end = self.selection_end();

        // Step 3
        if let Some(s) = start {
            if end < s {
                end = s;
            }
        }

        // Step 4
        self.set_selection_range(start, Some(end), Some(self.selection_direction()));
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn get_dom_selection_end(&self) -> Option<u32> {
        // Step 1
        if !self.selection_api_applies() {
            return None;
        }

        // Steps 2-3
        Some(self.selection_end())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn set_dom_selection_end(&self, end: Option<u32>) -> ErrorResult {
        // Step 1
        if !self.selection_api_applies() {
            return Err(Error::InvalidState);
        }

        // Step 2
        self.set_selection_range(Some(self.selection_start()), end, Some(self.selection_direction()));
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn get_dom_selection_direction(&self) -> Option<DOMString> {
        // Step 1
        if !self.selection_api_applies() {
            return None;
        }

        Some(DOMString::from(self.selection_direction()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn set_dom_selection_direction(&self, direction: Option<DOMString>) -> ErrorResult {
        // Step 1
        if !self.selection_api_applies() {
            return Err(Error::InvalidState);
        }

        // Step 2
        self.set_selection_range(
            Some(self.selection_start()),
            Some(self.selection_end()),
            direction.map(|d| SelectionDirection::from(d))
        );
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setselectionrange
    fn set_dom_selection_range(&self, start: u32, end: u32, direction: Option<DOMString>) -> ErrorResult {
        // Step 1
        if !self.selection_api_applies() {
            return Err(Error::InvalidState);
        }

        // Step 2
        self.set_selection_range(Some(start), Some(end), direction.map(|d| SelectionDirection::from(d)));
        Ok(())
    }

    fn selection_start(&self) -> u32 {
        self.textinput().borrow().get_selection_start()
    }

    fn selection_end(&self) -> u32 {
        self.textinput().borrow().get_absolute_insertion_point() as u32
    }

    fn selection_direction(&self) -> SelectionDirection {
        self.textinput().borrow().selection_direction
    }

    // https://html.spec.whatwg.org/multipage/#set-the-selection-range
    fn set_selection_range(&self, start: Option<u32>, end: Option<u32>, direction: Option<SelectionDirection>) {
        // Step 1
        let start = start.unwrap_or(0);

        // Step 2
        let end = end.unwrap_or(0);

        // Steps 3-5
        self.textinput().borrow_mut().set_selection_range(start, end, direction.unwrap_or(SelectionDirection::None));

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
