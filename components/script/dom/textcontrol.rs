/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This is an abstraction used by `HTMLInputElement` and `HTMLTextAreaElement` to implement the
//! text control selection DOM API.
//!
//! <https://html.spec.whatwg.org/multipage/#textFieldSelection>

use base::text::Utf16CodeUnitLength;
use layout_api::wrapper_traits::SelectionDirection;

use crate::clipboard_provider::EmbedderClipboardProvider;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLFormElementBinding::SelectionMode;
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::node::Node;
use crate::textinput::TextInput;

pub(crate) trait TextControlElement: DerivedFrom<EventTarget> + DerivedFrom<Node> {
    fn selection_api_applies(&self) -> bool;
    fn has_selectable_text(&self) -> bool;
    fn has_uncollapsed_selection(&self) -> bool;
    fn set_dirty_value_flag(&self, value: bool);
    fn select_all(&self);
    fn maybe_update_shared_selection(&self);
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

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-select>
    pub(crate) fn dom_select(&self) {
        // Step 1: If this element is an input element, and either select() does not apply
        // to this element or the corresponding control has no selectable text, return.
        if !self.element.has_selectable_text() {
            return;
        }

        // Step 2 : Set the selection range with 0 and infinity.
        self.set_range(
            Some(Utf16CodeUnitLength::zero()),
            Some(Utf16CodeUnitLength(usize::MAX)),
            None,
        );
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    pub(crate) fn dom_start(&self) -> Option<Utf16CodeUnitLength> {
        // Step 1
        if !self.element.selection_api_applies() {
            return None;
        }

        // Steps 2-3
        Some(self.start())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    pub(crate) fn set_dom_start(&self, start: Option<Utf16CodeUnitLength>) -> ErrorResult {
        // Step 1: If this element is an input element, and selectionStart does not apply
        // to this element, throw an "InvalidStateError" DOMException.
        if !self.element.selection_api_applies() {
            return Err(Error::InvalidState(None));
        }

        // Step 2: Let end be the value of this element's selectionEnd attribute.
        let mut end = self.end();

        // Step 3: If end is less than the given value, set end to the given value.
        match start {
            Some(start) if end < start => end = start,
            _ => {},
        }

        // Step 4: Set the selection range with the given value, end, and the value of
        // this element's selectionDirection attribute.
        self.set_range(start, Some(end), Some(self.direction()));
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    pub(crate) fn dom_end(&self) -> Option<Utf16CodeUnitLength> {
        // Step 1: If this element is an input element, and selectionEnd does not apply to
        // this element, return null.
        if !self.element.selection_api_applies() {
            return None;
        }

        // Step 2: If there is no selection, return the code unit offset within the
        // relevant value to the character that immediately follows the text entry cursor.
        // Step 3: Return the code unit offset within the relevant value to the character
        // that immediately follows the end of the selection.
        Some(self.end())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    pub(crate) fn set_dom_end(&self, end: Option<Utf16CodeUnitLength>) -> ErrorResult {
        // Step 1: If this element is an input element, and selectionEnd does not apply to
        // this element, throw an "InvalidStateError" DOMException.
        if !self.element.selection_api_applies() {
            return Err(Error::InvalidState(None));
        }

        // Step 2: Set the selection range with the value of this element's selectionStart
        // attribute, the given value, and the value of this element's selectionDirection
        // attribute.
        self.set_range(Some(self.start()), end, Some(self.direction()));
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    pub(crate) fn dom_direction(&self) -> Option<DOMString> {
        // Step 1
        if !self.element.selection_api_applies() {
            return None;
        }

        Some(DOMString::from(match self.direction() {
            SelectionDirection::Forward => "forward",
            SelectionDirection::Backward => "backward",
            SelectionDirection::None => "none",
        }))
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    pub(crate) fn set_dom_direction(&self, direction: Option<DOMString>) -> ErrorResult {
        // Step 1
        if !self.element.selection_api_applies() {
            return Err(Error::InvalidState(None));
        }

        // Step 2
        self.set_range(
            Some(self.start()),
            Some(self.end()),
            direction.map(|direction| direction.str().as_ref().into()),
        );
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setselectionrange
    pub(crate) fn set_dom_range(
        &self,
        start: Utf16CodeUnitLength,
        end: Utf16CodeUnitLength,
        direction: Option<DOMString>,
    ) -> ErrorResult {
        // Step 1
        if !self.element.selection_api_applies() {
            return Err(Error::InvalidState(None));
        }

        // Step 2
        self.set_range(
            Some(start),
            Some(end),
            direction.map(|direction| direction.str().as_ref().into()),
        );
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setrangetext
    pub(crate) fn set_dom_range_text(
        &self,
        replacement: DOMString,
        start: Option<Utf16CodeUnitLength>,
        end: Option<Utf16CodeUnitLength>,
        selection_mode: SelectionMode,
    ) -> ErrorResult {
        // Step 1: If this element is an input element, and setRangeText() does not apply
        // to this element, throw an "InvalidStateError" DOMException.
        if !self.element.selection_api_applies() {
            return Err(Error::InvalidState(None));
        }

        // Step 2: Set this element's dirty value flag to true.
        self.element.set_dirty_value_flag(true);

        // Step 3: If the method has only one argument, then let start and end have the
        // values of the selectionStart attribute and the selectionEnd attribute
        // respectively.
        //
        // Otherwise, let start, end have the values of the second and third arguments
        // respectively.
        let mut selection_start = self.start();
        let mut selection_end = self.end();
        let mut start = start.unwrap_or(selection_start);
        let mut end = end.unwrap_or(selection_end);

        // Step 4: If start is greater than end, then throw an "IndexSizeError"
        // DOMException.
        if start > end {
            return Err(Error::IndexSize(None));
        }

        // Step 5: If start is greater than the length of the relevant value of the text
        // control, then set it to the length of the relevant value of the text control.
        let content_length = self.textinput.borrow().len_utf16();
        if start > content_length {
            start = content_length;
        }

        // Step 6: If end is greater than the length of the relevant value of the text
        // control, then set it to the length of the relevant value of the text controlV
        if end > content_length {
            end = content_length;
        }

        // Step 7: Let selection start be the current value of the selectionStart
        // attribute.
        // Step 8: Let selection end be the current value of the selectionEnd attribute.
        //
        // NOTE: These were assigned above.

        {
            // Step 9: If start is less than end, delete the sequence of code units within
            // the element's relevant value starting with the code unit at the startth
            // position and ending with the code unit at the (end-1)th position.
            //
            // Step: 10: Insert the value of the first argument into the text of the
            // relevant value of the text control, immediately before the startth code
            // unit.
            let mut textinput = self.textinput.borrow_mut();
            textinput.set_selection_range_utf16(start, end, SelectionDirection::None);
            textinput.replace_selection(&replacement);
        }

        // Step 11: Let *new length* be the length of the value of the first argument.
        //
        // Must come before the textinput.replace_selection() call, as replacement gets moved in
        // that call.
        let new_length = replacement.len_utf16();

        // Step 12: Let new end be the sum of start and new length.
        let new_end = start + new_length;

        // Step 13: Run the appropriate set of substeps from the following list:
        match selection_mode {
            // ↪ If the fourth argument's value is "select"
            //     Let selection start be start.
            //     Let selection end be new end.
            SelectionMode::Select => {
                selection_start = start;
                selection_end = new_end;
            },

            // ↪ If the fourth argument's value is "start"
            //     Let selection start and selection end be start.
            SelectionMode::Start => {
                selection_start = start;
                selection_end = start;
            },

            // ↪ If the fourth argument's value is "end"
            //     Let selection start and selection end be new end
            SelectionMode::End => {
                selection_start = new_end;
                selection_end = new_end;
            },

            //  ↪ If the fourth argument's value is "preserve"
            // If the method has only one argument
            SelectionMode::Preserve => {
                // Sub-step 1: Let old length be end minus start.
                let old_length = end.saturating_sub(start);

                // Sub-step 2: Let delta be new length minus old length.
                let delta = (new_length.0 as isize) - (old_length.0 as isize);

                // Sub-step 3: If selection start is greater than end, then increment it
                // by delta. (If delta is negative, i.e. the new text is shorter than the
                // old text, then this will decrease the value of selection start.)
                //
                // Otherwise: if selection start is greater than start, then set it to
                // start. (This snaps the start of the selection to the start of the new
                // text if it was in the middle of the text that it replaced.)
                if selection_start > end {
                    selection_start =
                        Utf16CodeUnitLength::from((selection_start.0 as isize) + delta);
                } else if selection_start > start {
                    selection_start = start;
                }

                // Sub-step 4: If selection end is greater than end, then increment it by
                // delta in the same way.
                //
                // Otherwise: if selection end is greater than start, then set it to new
                // end. (This snaps the end of the selection to the end of the new text if
                // it was in the middle of the text that it replaced.)
                if selection_end > end {
                    selection_end = Utf16CodeUnitLength::from((selection_end.0 as isize) + delta);
                } else if selection_end > start {
                    selection_end = new_end;
                }
            },
        }

        // Step 14: Set the selection range with selection start and selection end.
        self.set_range(Some(selection_start), Some(selection_end), None);
        Ok(())
    }

    fn start(&self) -> Utf16CodeUnitLength {
        self.textinput.borrow().selection_start_utf16()
    }

    fn end(&self) -> Utf16CodeUnitLength {
        self.textinput.borrow().selection_end_utf16()
    }

    fn direction(&self) -> SelectionDirection {
        self.textinput.borrow().selection_direction()
    }

    /// <https://html.spec.whatwg.org/multipage/#set-the-selection-range>
    fn set_range(
        &self,
        start: Option<Utf16CodeUnitLength>,
        end: Option<Utf16CodeUnitLength>,
        direction: Option<SelectionDirection>,
    ) {
        // To set the selection range with an integer or null start, an integer or null or
        // the special value infinity end, and optionally a string direction, run the
        // following steps:
        //
        // Step 1: If start is null, let start be 0.
        let start = start.unwrap_or_default();

        // Step 2: If end is null, let end be 0.
        let end = end.unwrap_or_default();

        // Step 3: Set the selection of the text control to the sequence of code units
        // within the relevant value starting with the code unit at the startth position
        // (in logical order) and ending with the code unit at the (end-1)th position.
        // Arguments greater than the length of the relevant value of the text control
        // (including the special value infinity) must be treated as pointing at the end
        // of the text control. If end is less than or equal to start, then the start of
        // the selection and the end of the selection must both be placed immediately
        // before the character with offset end. In UAs where there is no concept of an
        // empty selection, this must set the cursor to be just before the character with
        // offset end.
        //
        // Step 4: If direction is not identical to either "backward" or "forward", or if
        // the direction argument was not given, set direction to "none".
        //
        // Step 5: Set the selection direction of the text control to direction.
        self.textinput.borrow_mut().set_selection_range_utf16(
            start,
            end,
            direction.unwrap_or(SelectionDirection::None),
        );

        // Step 6: If the previous steps caused the selection of the text control to be
        // modified (in either extent or direction), then queue an element task on the
        // user interaction task source given the element to fire an event named select at
        // the element, with the bubbles attribute initialized to true.
        //
        // Note: This is handled inside this method call.
        self.element.maybe_update_shared_selection();
    }
}
