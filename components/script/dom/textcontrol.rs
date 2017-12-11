/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding::SelectionMode;
use dom::bindings::conversions::DerivedFrom;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::str::DOMString;
use dom::event::{EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::node::{Node, NodeDamage, window_from_node};
use script_traits::ScriptToConstellationChan;
use textinput::{SelectionDirection, SelectionState, TextInput};

pub trait TextControl: DerivedFrom<EventTarget> + DerivedFrom<Node> {
    fn textinput(&self) -> &DomRefCell<TextInput<ScriptToConstellationChan>>;
    fn selection_api_applies(&self) -> bool;
    fn has_selectable_text(&self) -> bool;
    fn set_dirty_value_flag(&self, value: bool);

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-select
    fn dom_select(&self) {
        // Step 1
        if !self.has_selectable_text() {
            return;
        }

        // Step 2
        self.set_selection_range(Some(0), Some(u32::max_value()), None, None);
    }

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
        self.set_selection_range(start, Some(end), Some(self.selection_direction()), None);
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
        self.set_selection_range(Some(self.selection_start()), end, Some(self.selection_direction()), None);
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
            direction.map(|d| SelectionDirection::from(d)),
            None
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
        self.set_selection_range(Some(start), Some(end), direction.map(|d| SelectionDirection::from(d)), None);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setrangetext
    fn set_dom_range_text(&self, replacement: DOMString, start: Option<u32>, end: Option<u32>,
                          selection_mode: SelectionMode) -> ErrorResult {
        // Step 1
        if !self.selection_api_applies() {
            return Err(Error::InvalidState);
        }

        // Step 2
        self.set_dirty_value_flag(true);

        // Step 3
        let mut start = start.unwrap_or_else(|| self.selection_start());
        let mut end = end.unwrap_or_else(|| self.selection_end());

        // Step 4
        if start > end {
            return Err(Error::IndexSize);
        }

        // Save the original selection state to later pass to set_selection_range, because we will
        // change the selection state in order to replace the text in the range.
        let original_selection_state = self.textinput().borrow().selection_state();

        let content_length = self.textinput().borrow().len() as u32;

        // Step 5
        if start > content_length {
            start = content_length;
        }

        // Step 6
        if end > content_length {
            end = content_length;
        }

        // Step 7
        let mut selection_start = self.selection_start();

        // Step 8
        let mut selection_end = self.selection_end();

        // Step 11
        // Must come before the textinput.replace_selection() call, as replacement gets moved in
        // that call.
        let new_length = replacement.len() as u32;

        {
            let mut textinput = self.textinput().borrow_mut();

            // Steps 9-10
            textinput.set_selection_range(start, end, SelectionDirection::None);
            textinput.replace_selection(replacement);
        }

        // Step 12
        let new_end = start + new_length;

        // Step 13
        match selection_mode {
            SelectionMode::Select => {
                selection_start = start;
                selection_end = new_end;
            },

            SelectionMode::Start => {
                selection_start = start;
                selection_end = start;
            },

            SelectionMode::End => {
                selection_start = new_end;
                selection_end = new_end;
            },

            SelectionMode::Preserve => {
                // Sub-step 1
                let old_length = end - start;

                // Sub-step 2
                let delta = (new_length as isize) - (old_length as isize);

                // Sub-step 3
                if selection_start > end {
                    selection_start = ((selection_start as isize) + delta) as u32;
                } else if selection_start > start {
                    selection_start = start;
                }

                // Sub-step 4
                if selection_end > end {
                    selection_end = ((selection_end as isize) + delta) as u32;
                } else if selection_end > start {
                    selection_end = new_end;
                }
            },
        }

        // Step 14
        self.set_selection_range(
            Some(selection_start),
            Some(selection_end),
            None,
            Some(original_selection_state)
        );

        Ok(())
    }

    fn selection_start(&self) -> u32 {
        self.textinput().borrow().selection_start_offset() as u32
    }

    fn selection_end(&self) -> u32 {
        self.textinput().borrow().selection_end_offset() as u32
    }

    fn selection_direction(&self) -> SelectionDirection {
        self.textinput().borrow().selection_direction
    }

    // https://html.spec.whatwg.org/multipage/#set-the-selection-range
    fn set_selection_range(&self, start: Option<u32>, end: Option<u32>, direction: Option<SelectionDirection>,
                           original_selection_state: Option<SelectionState>) {
        let mut textinput = self.textinput().borrow_mut();
        let original_selection_state = original_selection_state.unwrap_or_else(|| textinput.selection_state());

        // Step 1
        let start = start.unwrap_or(0);

        // Step 2
        let end = end.unwrap_or(0);

        // Steps 3-5
        textinput.set_selection_range(start, end, direction.unwrap_or(SelectionDirection::None));

        // Step 6
        if textinput.selection_state() != original_selection_state {
            let window = window_from_node(self);
            window.user_interaction_task_source().queue_event(
                &self.upcast::<EventTarget>(),
                atom!("select"),
                EventBubbles::Bubbles,
                EventCancelable::NotCancelable,
                &window);
        }

        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }
}
