/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This is an abstraction used by `HTMLInputElement` and `HTMLTextAreaElement` to implement the
//! text control selection DOM API.
//!
//! <https://html.spec.whatwg.org/multipage/#textFieldSelection>

use utf16string::Utf16String;

use crate::clipboard_provider::EmbedderClipboardProvider;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLFormElementBinding::SelectionMode;
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::node::{Node, NodeDamage, NodeTraits};
use crate::textinput::{SelectionDirection, SelectionState, TextInput};

pub(crate) trait TextControlElement: DerivedFrom<EventTarget> + DerivedFrom<Node> {
    fn selection_api_applies(&self) -> bool;
    fn has_selectable_text(&self) -> bool;
    fn set_dirty_value_flag(&self, value: bool);
}

pub(crate) struct TextControlSelection<'a, E: TextControlElement> {
    element: &'a E,
    textinput: &'a DomRefCell<TextInput<EmbedderClipboardProvider>>,
}

impl<'a, E: TextControlElement> TextControlSelection<'a, E> {
    pub(crate) fn new(
        element: &'a E,
        textinput: &'a DomRefCell<TextInput<EmbedderClipboardProvider>>,
    ) -> Self {
        TextControlSelection { element, textinput }
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-select
    pub(crate) fn dom_select(&self) {
        // Step 1
        if !self.element.has_selectable_text() {
            return;
        }

        // Step 2
        self.set_range(Some(0), Some(u32::MAX), None, None);
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    pub(crate) fn dom_start(&self) -> Option<u32> {
        // Step 1
        if !self.element.selection_api_applies() {
            return None;
        }

        // Steps 2-3
        Some(self.start())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    pub(crate) fn set_dom_start(&self, start: Option<u32>) -> ErrorResult {
        // Step 1
        if !self.element.selection_api_applies() {
            return Err(Error::InvalidState);
        }

        // Step 2
        let mut end = self.end();

        // Step 3
        if let Some(s) = start {
            if end < s {
                end = s;
            }
        }

        // Step 4
        self.set_range(start, Some(end), Some(self.direction()), None);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    pub(crate) fn dom_end(&self) -> Option<u32> {
        // Step 1
        if !self.element.selection_api_applies() {
            return None;
        }

        // Steps 2-3
        Some(self.end())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    pub(crate) fn set_dom_end(&self, end: Option<u32>) -> ErrorResult {
        // Step 1
        if !self.element.selection_api_applies() {
            return Err(Error::InvalidState);
        }

        // Step 2
        self.set_range(Some(self.start()), end, Some(self.direction()), None);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    pub(crate) fn dom_direction(&self) -> Option<DOMString> {
        // Step 1
        if !self.element.selection_api_applies() {
            return None;
        }

        Some(DOMString::from(self.direction()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    pub(crate) fn set_dom_direction(&self, direction: Option<DOMString>) -> ErrorResult {
        // Step 1
        if !self.element.selection_api_applies() {
            return Err(Error::InvalidState);
        }

        // Step 2
        self.set_range(
            Some(self.start()),
            Some(self.end()),
            direction.map(SelectionDirection::from),
            None,
        );
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setselectionrange
    pub(crate) fn set_dom_range(
        &self,
        start: u32,
        end: u32,
        direction: Option<DOMString>,
    ) -> ErrorResult {
        // Step 1
        if !self.element.selection_api_applies() {
            return Err(Error::InvalidState);
        }

        // Step 2
        self.set_range(
            Some(start),
            Some(end),
            direction.map(SelectionDirection::from),
            None,
        );
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-setrangetext>
    pub(crate) fn set_dom_range_text(
        &self,
        replacement: Utf16String,
        start: Option<u32>,
        end: Option<u32>,
        selection_mode: SelectionMode,
    ) -> ErrorResult {
        // Step 1. If this element is an input element, and setRangeText() does not apply
        // to this element, throw an "InvalidStateError" DOMException.
        if !self.element.selection_api_applies() {
            return Err(Error::InvalidState);
        }

        // Step 2. Set this element's dirty value flag to true.
        self.element.set_dirty_value_flag(true);

        // Step 3. If the method has only one argument, then let start and end have the values of the selectionStart
        // attribute and the selectionEnd attribute respectively.
        // Otherwise, let start, end have the values of the second and third arguments respectively.
        let mut start = start.unwrap_or_else(|| self.start());
        let mut end = end.unwrap_or_else(|| self.end());

        // Step 4. If start is greater than end, then throw an "IndexSizeError" DOMException.
        if start > end {
            return Err(Error::IndexSize);
        }

        // Save the original selection state to later pass to set_selection_range, because we will
        // change the selection state in order to replace the text in the range.
        let original_selection_state = self.textinput.borrow().selection_state();

        let content_length = self.textinput.borrow().utf16_len().0 as u32;

        // Step 5. If start is greater than the length of the relevant value of the text control,
        // then set it to the length of the relevant value of the text control.
        if start > content_length {
            start = content_length;
        }

        // Step 6. If end is greater than the length of the relevant value of the text control,
        // then set it to the length of the relevant value of the text control.
        if end > content_length {
            end = content_length;
        }

        // Step 7. Let selection start be the current value of the selectionStart attribute.
        let mut selection_start = self.start();

        // Step 8. Let selection end be the current value of the selectionEnd attribute.
        let mut selection_end = self.end();

        // Step 11. Let new length be the length of the value of the first argument.
        // NOTE: Must come before the textinput.replace_selection() call, as replacement gets moved in
        // that call.
        let new_length = replacement.number_of_code_units() as u32;

        {
            let mut textinput = self.textinput.borrow_mut();

            // Step 9. If start is less than end, delete the sequence of code units within the element's relevant value starting with the code unit
            // at the startth position and ending with the code unit at the (end-1)th position.
            // Step 10. Insert the value of the first argument into the text of the relevant value of the text control,
            // immediately before the startth code unit.
            textinput.set_selection_range(start, end, SelectionDirection::None);
            textinput.replace_selection(replacement);
        }

        // Step 12. Let new end be the sum of start and new length.
        let new_end = start + new_length;

        // Step 13. Run the appropriate set of substeps from the following list:
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

        // Step 14. Set the selection range with selection start and selection end.
        self.set_range(
            Some(selection_start),
            Some(selection_end),
            None,
            Some(original_selection_state),
        );
        Ok(())
    }

    fn start(&self) -> u32 {
        // FIXME: We need to convert from byte offsets to code unit offsets here
        self.textinput.borrow().selection_start_offset() as u32
    }

    fn end(&self) -> u32 {
        // FIXME: We need to convert from byte offsets to code unit offsets here
        self.textinput.borrow().selection_end_offset() as u32
    }

    fn direction(&self) -> SelectionDirection {
        self.textinput.borrow().selection_direction()
    }

    /// <https://html.spec.whatwg.org/multipage/#set-the-selection-range>
    fn set_range(
        &self,
        start: Option<u32>,
        end: Option<u32>,
        direction: Option<SelectionDirection>,
        original_selection_state: Option<SelectionState>,
    ) {
        let mut textinput = self.textinput.borrow_mut();
        let original_selection_state =
            original_selection_state.unwrap_or_else(|| textinput.selection_state());

        // Step 1. If start is null, let start be zero.
        let start = start.unwrap_or(0);

        // Step 2. If end is null, let end be zero.
        let end = end.unwrap_or(0);

        // Steps 3-5
        textinput.set_selection_range(start, end, direction.unwrap_or(SelectionDirection::None));

        // Step 6
        if textinput.selection_state() != original_selection_state {
            self.element
                .owner_global()
                .task_manager()
                .user_interaction_task_source()
                .queue_event(
                    self.element.upcast::<EventTarget>(),
                    atom!("select"),
                    EventBubbles::Bubbles,
                    EventCancelable::NotCancelable,
                );
        }

        self.element
            .upcast::<Node>()
            .dirty(NodeDamage::OtherNodeDamage);
    }
}
