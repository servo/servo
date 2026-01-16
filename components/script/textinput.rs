/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common handling of keyboard input and state management for text input controls

use std::default::Default;
use std::ops::Range;

use base::text::{Utf8CodeUnitLength, Utf16CodeUnitLength};
use base::{Lines, Rope, RopeIndex, RopeMovement, RopeSlice};
use bitflags::bitflags;
use keyboard_types::{Key, KeyState, Modifiers, NamedKey, ShortcutMatcher};
use script_bindings::codegen::GenericBindings::HTMLFormElementBinding::SelectionMode;
use script_bindings::match_domstring_ascii;
use script_bindings::root::DomRoot;
use script_bindings::trace::CustomTraceable;

use crate::clipboard_provider::ClipboardProvider;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::str::DOMString;
use crate::dom::compositionevent::CompositionEvent;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::inputevent::InputEvent;
use crate::dom::keyboardevent::KeyboardEvent;
use crate::dom::node::{Node, NodeDamage, NodeTraits};
use crate::dom::types::{ClipboardEvent, Element, HTMLInputElement, HTMLTextAreaElement};
use crate::drag_data_store::Kind;
use crate::script_runtime::CanGc;

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum TextInputElement {
    /// An `<input>` element.
    Input(DomRoot<HTMLInputElement>),
    /// A `<textarea>` element.
    TextArea(DomRoot<HTMLTextAreaElement>),
    /// A mock element owner for a [`TextInput`] used for unit tests.
    Mock,
}

impl TextInputElement {
    fn set_dirty_value_flag(&self, value: bool) {
        match self {
            TextInputElement::Input(input_element) => input_element.set_dirty_value_flag(value),
            TextInputElement::TextArea(textarea_element) => {
                textarea_element.set_dirty_value_flag(value)
            },
            TextInputElement::Mock => {},
        }
    }

    fn as_element(&self) -> &Element {
        match self {
            TextInputElement::Input(input_element) => input_element.upcast(),
            TextInputElement::TextArea(textarea_element) => textarea_element.upcast(),
            TextInputElement::Mock => unreachable!("Should not call DOM methods from unit tests"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Selection {
    Selected,
    NotSelected,
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub enum SelectionDirection {
    Forward,
    Backward,
    None,
}

impl From<DOMString> for SelectionDirection {
    fn from(direction: DOMString) -> SelectionDirection {
        match_domstring_ascii!(direction,
            "forward" => SelectionDirection::Forward,
            "backward" => SelectionDirection::Backward,
            _ => SelectionDirection::None,
        )
    }
}

impl From<SelectionDirection> for DOMString {
    fn from(direction: SelectionDirection) -> DOMString {
        match direction {
            SelectionDirection::Forward => DOMString::from("forward"),
            SelectionDirection::Backward => DOMString::from("backward"),
            SelectionDirection::None => DOMString::from("none"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct SelectionState {
    start: RopeIndex,
    end: RopeIndex,
    direction: SelectionDirection,
}

/// Encapsulated state for handling keyboard input in a single or multiline text input control.
#[derive(JSTraceable, MallocSizeOf)]
pub struct TextInput<T: ClipboardProvider> {
    /// Storage for the contents of this [`TextInput`]. This string contents are stored
    /// in the [`Rope`] as UTF-8 strings, one per line.
    #[no_trace]
    rope: Rope,

    /// The element that this [`TextInput`] belongs to. This can be `None` when this
    /// [`TextInput`] is being used in unit tests.
    element: TextInputElement,

    /// Current cursor input point
    #[no_trace]
    edit_point: RopeIndex,

    /// The current selection goes from the selection_origin until the edit_point. Note that the
    /// selection_origin may be after the edit_point, in the case of a backward selection.
    #[no_trace]
    selection_origin: Option<RopeIndex>,
    selection_direction: SelectionDirection,

    #[ignore_malloc_size_of = "Can't easily measure this generic type"]
    clipboard_provider: T,

    /// The maximum number of UTF-16 code units this text input is allowed to hold.
    ///
    /// <https://html.spec.whatwg.org/multipage/#attr-fe-maxlength>
    max_length: Option<Utf16CodeUnitLength>,
    min_length: Option<Utf16CodeUnitLength>,

    /// Was last change made by set_content?
    was_last_change_by_set_content: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IsComposing {
    Composing,
    NotComposing,
}

impl From<IsComposing> for bool {
    fn from(is_composing: IsComposing) -> Self {
        match is_composing {
            IsComposing::Composing => true,
            IsComposing::NotComposing => false,
        }
    }
}

/// <https://www.w3.org/TR/input-events-2/#interface-InputEvent-Attributes>
#[derive(Clone, Copy, PartialEq)]
pub enum InputType {
    InsertText,
    InsertLineBreak,
    InsertFromPaste,
    InsertCompositionText,
    DeleteByCut,
    DeleteContentBackward,
    DeleteContentForward,
    Nothing,
}

impl InputType {
    fn as_str(&self) -> &str {
        match *self {
            InputType::InsertText => "insertText",
            InputType::InsertLineBreak => "insertLineBreak",
            InputType::InsertFromPaste => "insertFromPaste",
            InputType::InsertCompositionText => "insertCompositionText",
            InputType::DeleteByCut => "deleteByCut",
            InputType::DeleteContentBackward => "deleteContentBackward",
            InputType::DeleteContentForward => "deleteContentForward",
            InputType::Nothing => "",
        }
    }
}

/// Resulting action to be taken by the owner of a text input that is handling an event.
pub enum KeyReaction {
    TriggerDefaultAction,
    DispatchInput(Option<String>, IsComposing, InputType),
    RedrawSelection,
    Nothing,
}

bitflags! {
    /// Resulting action to be taken by the owner of a text input that is handling a clipboard
    /// event.
    #[derive(Clone, Copy)]
    pub struct ClipboardEventFlags: u8 {
        const QueueInputEvent = 1 << 0;
        const FireClipboardChangedEvent = 1 << 1;
    }
}

pub struct ClipboardEventReaction {
    pub flags: ClipboardEventFlags,
    pub text: Option<String>,
    pub input_type: InputType,
}

impl ClipboardEventReaction {
    fn new(flags: ClipboardEventFlags) -> Self {
        Self {
            flags,
            text: None,
            input_type: InputType::Nothing,
        }
    }

    fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    fn with_input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }

    fn empty() -> Self {
        Self::new(ClipboardEventFlags::empty())
    }
}

/// The direction in which to delete a character.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

// Some shortcuts use Cmd on Mac and Control on other systems.
#[cfg(target_os = "macos")]
pub(crate) const CMD_OR_CONTROL: Modifiers = Modifiers::META;
#[cfg(not(target_os = "macos"))]
pub(crate) const CMD_OR_CONTROL: Modifiers = Modifiers::CONTROL;

/// The length in bytes of the first n code units in a string when encoded in UTF-16.
///
/// If the string is fewer than n code units, returns the length of the whole string.
fn len_of_first_n_code_units(text: &DOMString, n: Utf16CodeUnitLength) -> Utf8CodeUnitLength {
    let mut utf8_len = Utf8CodeUnitLength::zero();
    let mut utf16_len = Utf16CodeUnitLength::zero();
    for c in text.str().chars() {
        utf16_len += Utf16CodeUnitLength(c.len_utf16());
        if utf16_len > n {
            break;
        }
        utf8_len += Utf8CodeUnitLength(c.len_utf8());
    }
    utf8_len
}

impl<T: ClipboardProvider> TextInput<T> {
    /// Instantiate a new text input control
    pub fn new(
        lines: Lines,
        initial: DOMString,
        clipboard_provider: T,
        max_length: Option<Utf16CodeUnitLength>,
        min_length: Option<Utf16CodeUnitLength>,
        selection_direction: SelectionDirection,
    ) -> TextInput<T> {
        Self {
            rope: Rope::new(initial, lines),
            element: TextInputElement::Mock,
            edit_point: Default::default(),
            selection_origin: None,
            clipboard_provider,
            max_length,
            min_length,
            selection_direction,
            was_last_change_by_set_content: true,
        }
    }

    pub(crate) fn set_element(&mut self, element: TextInputElement) {
        self.element = element;
    }

    pub fn edit_point(&self) -> RopeIndex {
        self.edit_point
    }

    pub fn selection_origin(&self) -> Option<RopeIndex> {
        self.selection_origin
    }

    /// The selection origin, or the edit point if there is no selection. Note that the selection
    /// origin may be after the edit point, in the case of a backward selection.
    pub fn selection_origin_or_edit_point(&self) -> RopeIndex {
        self.selection_origin.unwrap_or(self.edit_point)
    }

    pub fn selection_direction(&self) -> SelectionDirection {
        self.selection_direction
    }

    pub(crate) fn set_max_length(&mut self, length: Option<Utf16CodeUnitLength>) {
        self.max_length = length;
    }

    pub(crate) fn set_min_length(&mut self, length: Option<Utf16CodeUnitLength>) {
        self.min_length = length;
    }

    /// Was last edit made by set_content?
    pub(crate) fn was_last_change_by_set_content(&self) -> bool {
        self.was_last_change_by_set_content
    }

    /// Remove a character at the current editing point
    ///
    /// Returns true if any character was deleted
    pub fn delete_char(&mut self, direction: Direction) -> bool {
        if self.selection_origin.is_none() || self.selection_origin == Some(self.edit_point) {
            let amount = match direction {
                Direction::Forward => 1,
                Direction::Backward => -1,
            };
            self.modify_selection(amount, RopeMovement::Grapheme);
        }

        if self.selection_start() == self.selection_end() {
            return false;
        }

        self.replace_selection(&DOMString::new());
        true
    }

    /// Insert a character at the current editing point
    pub fn insert_char(&mut self, ch: char) {
        self.insert_string(ch.to_string());
    }

    /// Insert a string at the current editing point or replace the selection if
    /// one exists.
    pub fn insert_string<S: Into<String>>(&mut self, s: S) {
        if self.selection_origin.is_none() {
            self.selection_origin = Some(self.edit_point);
        }
        self.replace_selection(&DOMString::from(s.into()));
    }

    /// The start of the selection (or the edit point, if there is no selection). Always less than
    /// or equal to selection_end(), regardless of the selection direction.
    pub fn selection_start(&self) -> RopeIndex {
        match self.selection_direction {
            SelectionDirection::None | SelectionDirection::Forward => {
                self.selection_origin_or_edit_point()
            },
            SelectionDirection::Backward => self.edit_point,
        }
    }

    pub(crate) fn selection_start_utf16(&self) -> Utf16CodeUnitLength {
        self.rope.index_to_utf16_offset(self.selection_start())
    }

    /// The byte offset of the selection_start()
    fn selection_start_offset(&self) -> Utf8CodeUnitLength {
        self.rope.index_to_utf8_offset(self.selection_start())
    }

    /// The end of the selection (or the edit point, if there is no selection). Always greater
    /// than or equal to selection_start(), regardless of the selection direction.
    pub fn selection_end(&self) -> RopeIndex {
        match self.selection_direction {
            SelectionDirection::None | SelectionDirection::Forward => self.edit_point,
            SelectionDirection::Backward => self.selection_origin_or_edit_point(),
        }
    }

    pub(crate) fn selection_end_utf16(&self) -> Utf16CodeUnitLength {
        self.rope.index_to_utf16_offset(self.selection_end())
    }

    /// The byte offset of the selection_end()
    pub fn selection_end_offset(&self) -> Utf8CodeUnitLength {
        self.rope.index_to_utf8_offset(self.selection_end())
    }

    /// Whether or not there is an active selection (the selection may be zero-length)
    #[inline]
    pub(crate) fn has_selection(&self) -> bool {
        self.selection_origin.is_some()
    }

    /// Return the selection range as byte offsets from the start of the content.
    ///
    /// If there is no selection, returns an empty range at the edit point.
    pub(crate) fn sorted_selection_offsets_range(&self) -> Range<Utf8CodeUnitLength> {
        self.selection_start_offset()..self.selection_end_offset()
    }

    /// The state of the current selection. Can be used to compare whether selection state has changed.
    pub(crate) fn selection_state(&self) -> SelectionState {
        SelectionState {
            start: self.selection_start(),
            end: self.selection_end(),
            direction: self.selection_direction,
        }
    }

    // Check that the selection is valid.
    fn assert_ok_selection(&self) {
        debug!(
            "edit_point: {:?}, selection_origin: {:?}, direction: {:?}",
            self.edit_point, self.selection_origin, self.selection_direction
        );

        debug_assert_eq!(self.edit_point, self.rope.clamp_index(self.edit_point));
        if let Some(selection_origin) = self.selection_origin {
            debug_assert_eq!(selection_origin, self.rope.clamp_index(selection_origin));
            match self.selection_direction {
                SelectionDirection::None | SelectionDirection::Forward => {
                    debug_assert!(selection_origin <= self.edit_point)
                },
                SelectionDirection::Backward => debug_assert!(self.edit_point <= selection_origin),
            }
        }
    }

    fn selection_slice(&self) -> RopeSlice<'_> {
        self.rope
            .slice(Some(self.selection_start()), Some(self.selection_end()))
    }

    pub(crate) fn get_selection_text(&self) -> Option<String> {
        let text: String = self.selection_slice().into();
        if text.is_empty() {
            return None;
        }
        Some(text)
    }

    /// The length of the selected text in UTF-16 code units.
    fn selection_utf16_len(&self) -> Utf16CodeUnitLength {
        Utf16CodeUnitLength(
            self.selection_slice()
                .chars()
                .map(char::len_utf16)
                .sum::<usize>(),
        )
    }

    pub fn replace_selection(&mut self, insert: &DOMString) {
        if !self.has_selection() {
            return;
        }

        let string_to_insert = if let Some(max_length) = self.max_length {
            let utf16_length_without_selection =
                self.len_utf16().saturating_sub(self.selection_utf16_len());
            let utf16_length_that_can_be_inserted =
                max_length.saturating_sub(utf16_length_without_selection);
            let Utf8CodeUnitLength(last_char_index) =
                len_of_first_n_code_units(insert, utf16_length_that_can_be_inserted);
            &insert.str()[..last_char_index]
        } else {
            &insert.str()
        };

        let start = self.selection_start();
        let end = self.selection_end();
        let end_index_of_insertion = self.rope.replace_range(start..end, string_to_insert);

        self.was_last_change_by_set_content = false;
        self.clear_selection();
        self.edit_point = end_index_of_insertion;
    }

    pub fn modify_edit_point(&mut self, amount: isize, movement: RopeMovement) {
        if amount == 0 {
            return;
        }

        // When moving by lines or if we do not have a selection, we do actually move
        // the edit point from its position.
        if matches!(movement, RopeMovement::Line) || !self.has_selection() {
            self.clear_selection();
            self.edit_point = self.rope.move_by(self.edit_point, movement, amount);
            return;
        }

        // If there's a selection and we are moving by words or characters, we just collapse
        // the selection in the direction of the motion.
        let new_edit_point = if amount > 0 {
            self.selection_end()
        } else {
            self.selection_start()
        };
        self.clear_selection();
        self.edit_point = new_edit_point;
    }

    pub fn modify_selection(&mut self, amount: isize, movement: RopeMovement) {
        let old_edit_point = self.edit_point;
        self.edit_point = self.rope.move_by(old_edit_point, movement, amount);

        if self.selection_origin.is_none() {
            self.selection_origin = Some(old_edit_point);
        }
        self.update_selection_direction();
    }

    pub fn modify_selection_or_edit_point(
        &mut self,
        amount: isize,
        movement: RopeMovement,
        select: Selection,
    ) {
        match select {
            Selection::Selected => self.modify_selection(amount, movement),
            Selection::NotSelected => self.modify_edit_point(amount, movement),
        }
        self.assert_ok_selection();
    }

    /// Update the field selection_direction.
    ///
    /// When the edit_point (or focus) is before the selection_origin (or anchor)
    /// you have a backward selection. Otherwise you have a forward selection.
    fn update_selection_direction(&mut self) {
        debug!(
            "edit_point: {:?}, selection_origin: {:?}",
            self.edit_point, self.selection_origin
        );
        self.selection_direction = if Some(self.edit_point) < self.selection_origin {
            SelectionDirection::Backward
        } else {
            SelectionDirection::Forward
        }
    }

    /// Deal with a newline input.
    pub fn handle_return(&mut self) -> KeyReaction {
        match self.rope.mode() {
            Lines::Multiple => {
                self.insert_char('\n');
                KeyReaction::DispatchInput(
                    None,
                    IsComposing::NotComposing,
                    InputType::InsertLineBreak,
                )
            },
            Lines::Single => KeyReaction::TriggerDefaultAction,
        }
    }

    /// Select all text in the input control.
    pub fn select_all(&mut self) {
        self.selection_origin = Some(RopeIndex::default());
        self.edit_point = self.rope.last_index();
        self.selection_direction = SelectionDirection::Forward;
        self.assert_ok_selection();
    }

    /// Remove the current selection.
    pub fn clear_selection(&mut self) {
        self.selection_origin = None;
        self.selection_direction = SelectionDirection::None;
    }

    /// Remove the current selection and set the edit point to the end of the content.
    pub(crate) fn clear_selection_to_end(&mut self) {
        self.clear_selection();
        self.edit_point = self.rope.last_index();
    }

    pub(crate) fn clear_selection_to_start(&mut self) {
        self.clear_selection();
        self.edit_point = Default::default();
    }

    /// Process a given `KeyboardEvent` and return an action for the caller to execute.
    pub(crate) fn handle_keydown(&mut self, event: &KeyboardEvent) -> KeyReaction {
        let key = event.key();
        let mods = event.modifiers();
        self.handle_keydown_aux(key, mods, cfg!(target_os = "macos"))
    }

    // This function exists for easy unit testing.
    // To test Mac OS shortcuts on other systems a flag is passed.
    pub fn handle_keydown_aux(
        &mut self,
        key: Key,
        mut mods: Modifiers,
        macos: bool,
    ) -> KeyReaction {
        let maybe_select = if mods.contains(Modifiers::SHIFT) {
            Selection::Selected
        } else {
            Selection::NotSelected
        };
        mods.remove(Modifiers::SHIFT);
        ShortcutMatcher::new(KeyState::Down, key.clone(), mods)
            .shortcut(Modifiers::CONTROL | Modifiers::ALT, 'B', || {
                self.modify_selection_or_edit_point(-1, RopeMovement::Word, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::CONTROL | Modifiers::ALT, 'F', || {
                self.modify_selection_or_edit_point(1, RopeMovement::Word, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::CONTROL | Modifiers::ALT, 'A', || {
                self.modify_selection_or_edit_point(-1, RopeMovement::LineStartOrEnd, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::CONTROL | Modifiers::ALT, 'E', || {
                self.modify_selection_or_edit_point(1, RopeMovement::LineStartOrEnd, maybe_select);
                KeyReaction::RedrawSelection
            })
            .optional_shortcut(macos, Modifiers::CONTROL, 'A', || {
                self.modify_selection_or_edit_point(-1, RopeMovement::LineStartOrEnd, maybe_select);
                KeyReaction::RedrawSelection
            })
            .optional_shortcut(macos, Modifiers::CONTROL, 'E', || {
                self.modify_selection_or_edit_point(1, RopeMovement::LineStartOrEnd, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(CMD_OR_CONTROL, 'A', || {
                self.select_all();
                KeyReaction::RedrawSelection
            })
            .shortcut(CMD_OR_CONTROL, 'X', || {
                if let Some(text) = self.get_selection_text() {
                    self.clipboard_provider.set_text(text);
                    self.delete_char(Direction::Backward);
                }
                KeyReaction::DispatchInput(None, IsComposing::NotComposing, InputType::DeleteByCut)
            })
            .shortcut(CMD_OR_CONTROL, 'C', || {
                // TODO(stevennovaryo): we should not provide text to clipboard for type=password
                if let Some(text) = self.get_selection_text() {
                    self.clipboard_provider.set_text(text);
                }
                KeyReaction::DispatchInput(None, IsComposing::NotComposing, InputType::Nothing)
            })
            .shortcut(CMD_OR_CONTROL, 'V', || {
                if let Ok(text_content) = self.clipboard_provider.get_text() {
                    self.insert_string(&text_content);
                    KeyReaction::DispatchInput(
                        Some(text_content),
                        IsComposing::NotComposing,
                        InputType::InsertFromPaste,
                    )
                } else {
                    KeyReaction::DispatchInput(
                        Some("".to_string()),
                        IsComposing::NotComposing,
                        InputType::InsertFromPaste,
                    )
                }
            })
            .shortcut(Modifiers::empty(), Key::Named(NamedKey::Delete), || {
                if self.delete_char(Direction::Forward) {
                    KeyReaction::DispatchInput(
                        None,
                        IsComposing::NotComposing,
                        InputType::DeleteContentForward,
                    )
                } else {
                    KeyReaction::Nothing
                }
            })
            .shortcut(Modifiers::empty(), Key::Named(NamedKey::Backspace), || {
                if self.delete_char(Direction::Backward) {
                    KeyReaction::DispatchInput(
                        None,
                        IsComposing::NotComposing,
                        InputType::DeleteContentBackward,
                    )
                } else {
                    KeyReaction::Nothing
                }
            })
            .optional_shortcut(
                macos,
                Modifiers::META,
                Key::Named(NamedKey::ArrowLeft),
                || {
                    self.modify_selection_or_edit_point(
                        -1,
                        RopeMovement::LineStartOrEnd,
                        maybe_select,
                    );
                    KeyReaction::RedrawSelection
                },
            )
            .optional_shortcut(
                macos,
                Modifiers::META,
                Key::Named(NamedKey::ArrowRight),
                || {
                    self.modify_selection_or_edit_point(
                        1,
                        RopeMovement::LineStartOrEnd,
                        maybe_select,
                    );
                    KeyReaction::RedrawSelection
                },
            )
            .optional_shortcut(
                macos,
                Modifiers::META,
                Key::Named(NamedKey::ArrowUp),
                || {
                    self.modify_selection_or_edit_point(
                        -1,
                        RopeMovement::RopeStartOrEnd,
                        maybe_select,
                    );
                    KeyReaction::RedrawSelection
                },
            )
            .optional_shortcut(
                macos,
                Modifiers::META,
                Key::Named(NamedKey::ArrowDown),
                || {
                    self.modify_selection_or_edit_point(
                        1,
                        RopeMovement::RopeStartOrEnd,
                        maybe_select,
                    );
                    KeyReaction::RedrawSelection
                },
            )
            .shortcut(Modifiers::ALT, Key::Named(NamedKey::ArrowLeft), || {
                self.modify_selection_or_edit_point(-1, RopeMovement::Word, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::ALT, Key::Named(NamedKey::ArrowRight), || {
                self.modify_selection_or_edit_point(1, RopeMovement::Word, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::Named(NamedKey::ArrowLeft), || {
                self.modify_selection_or_edit_point(-1, RopeMovement::Grapheme, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::Named(NamedKey::ArrowRight), || {
                self.modify_selection_or_edit_point(1, RopeMovement::Grapheme, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::Named(NamedKey::ArrowUp), || {
                self.modify_selection_or_edit_point(-1, RopeMovement::Line, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::Named(NamedKey::ArrowDown), || {
                self.modify_selection_or_edit_point(1, RopeMovement::Line, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::Named(NamedKey::Enter), || {
                self.handle_return()
            })
            .optional_shortcut(
                macos,
                Modifiers::empty(),
                Key::Named(NamedKey::Home),
                || {
                    self.modify_selection_or_edit_point(
                        -1,
                        RopeMovement::RopeStartOrEnd,
                        maybe_select,
                    );
                    KeyReaction::RedrawSelection
                },
            )
            .optional_shortcut(macos, Modifiers::empty(), Key::Named(NamedKey::End), || {
                self.modify_selection_or_edit_point(1, RopeMovement::RopeStartOrEnd, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::Named(NamedKey::PageUp), || {
                self.modify_selection_or_edit_point(-28, RopeMovement::Line, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::Named(NamedKey::PageDown), || {
                self.modify_selection_or_edit_point(28, RopeMovement::Line, maybe_select);
                KeyReaction::RedrawSelection
            })
            .otherwise(|| {
                if let Key::Character(ref c) = key {
                    self.insert_string(c.as_str());
                    return KeyReaction::DispatchInput(
                        Some(c.to_string()),
                        IsComposing::NotComposing,
                        InputType::InsertText,
                    );
                }
                if matches!(key, Key::Named(NamedKey::Process)) {
                    return KeyReaction::DispatchInput(
                        None,
                        IsComposing::Composing,
                        InputType::Nothing,
                    );
                }
                KeyReaction::Nothing
            })
            .unwrap()
    }

    pub(crate) fn handle_compositionend(&mut self, event: &CompositionEvent) -> KeyReaction {
        let ch = event.data().str();
        self.insert_string(ch.as_ref());
        KeyReaction::DispatchInput(
            Some(ch.to_string()),
            IsComposing::NotComposing,
            InputType::InsertCompositionText,
        )
    }

    pub(crate) fn handle_compositionupdate(&mut self, event: &CompositionEvent) -> KeyReaction {
        let insertion = event.data().str();
        let start = self.selection_start_offset();
        self.insert_string(insertion.as_ref());
        self.set_selection_range_utf8(
            start,
            start + event.data().len_utf8(),
            SelectionDirection::Forward,
        );
        KeyReaction::DispatchInput(
            Some(insertion.to_string()),
            IsComposing::Composing,
            InputType::InsertCompositionText,
        )
    }

    /// Whether the content is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.rope.is_empty()
    }

    /// The total number of code units required to encode the content in utf16.
    pub(crate) fn len_utf16(&self) -> Utf16CodeUnitLength {
        self.rope.len_utf16()
    }

    /// Get the current contents of the text input. Multiple lines are joined by \n.
    pub fn get_content(&self) -> DOMString {
        self.rope.contents().into()
    }

    /// Set the current contents of the text input. If this is control supports multiple lines,
    /// any \n encountered will be stripped and force a new logical line.
    pub fn set_content(&mut self, content: DOMString) {
        self.rope = Rope::new(content, self.rope.mode());
        self.was_last_change_by_set_content = true;

        self.edit_point = self.rope.clamp_index(self.edit_point());
        self.selection_origin = self
            .selection_origin
            .map(|selection_origin| self.rope.clamp_index(selection_origin));
    }

    pub(crate) fn set_selection_range_utf16(
        &mut self,
        start: Utf16CodeUnitLength,
        end: Utf16CodeUnitLength,
        direction: SelectionDirection,
    ) {
        self.set_selection_range_utf8(
            self.rope.utf16_offset_to_utf8_offset(start),
            self.rope.utf16_offset_to_utf8_offset(end),
            direction,
        );
    }

    pub fn set_selection_range_utf8(
        &mut self,
        mut start: Utf8CodeUnitLength,
        mut end: Utf8CodeUnitLength,
        direction: SelectionDirection,
    ) {
        let text_end = self.get_content().len_utf8();
        if end > text_end {
            end = text_end;
        }
        if start > end {
            start = end;
        }

        self.selection_direction = direction;

        match direction {
            SelectionDirection::None | SelectionDirection::Forward => {
                self.selection_origin = Some(self.rope.utf8_offset_to_rope_index(start));
                self.edit_point = self.rope.utf8_offset_to_rope_index(end);
            },
            SelectionDirection::Backward => {
                self.selection_origin = Some(self.rope.utf8_offset_to_rope_index(end));
                self.edit_point = self.rope.utf8_offset_to_rope_index(start);
            },
        }
        self.assert_ok_selection();
    }

    /// This implements step 3 onward from:
    ///
    ///  - <https://www.w3.org/TR/clipboard-apis/#copy-action>
    ///  - <https://www.w3.org/TR/clipboard-apis/#cut-action>
    ///  - <https://www.w3.org/TR/clipboard-apis/#paste-action>
    ///
    /// Earlier steps should have already been run by the callers.
    pub(crate) fn handle_clipboard_event(
        &mut self,
        clipboard_event: &ClipboardEvent,
    ) -> ClipboardEventReaction {
        let event = clipboard_event.upcast::<Event>();
        if !event.IsTrusted() {
            return ClipboardEventReaction::empty();
        }

        // This step is common to all event types in the specification.
        // Step 3: If the event was not canceled, then
        if event.DefaultPrevented() {
            // Step 4: Else, if the event was canceled
            // Step 4.1: Return false.
            return ClipboardEventReaction::empty();
        }

        let event_type = event.Type();
        match_domstring_ascii!(event_type,
            "copy" => {
                // These steps are from <https://www.w3.org/TR/clipboard-apis/#copy-action>:
                let selection = self.get_selection_text();

                // Step 3.1 Copy the selected contents, if any, to the clipboard
                if let Some(text) = selection {
                    self.clipboard_provider.set_text(text);
                }

                // Step 3.2 Fire a clipboard event named clipboardchange
                ClipboardEventReaction::new(ClipboardEventFlags::FireClipboardChangedEvent)
            },
            "cut" => {
                // These steps are from <https://www.w3.org/TR/clipboard-apis/#cut-action>:
                let selection = self.get_selection_text();

                // Step 3.1 If there is a selection in an editable context where cutting is enabled, then
                let Some(text) = selection else {
                    // Step 3.2 Else, if there is no selection or the context is not editable, then
                    return ClipboardEventReaction::empty();
                };

                // Step 3.1.1 Copy the selected contents, if any, to the clipboard
                self.clipboard_provider.set_text(text);

                // Step 3.1.2 Remove the contents of the selection from the document and collapse the selection.
                self.delete_char(Direction::Backward);

                // Step 3.1.3 Fire a clipboard event named clipboardchange
                // Step 3.1.4 Queue tasks to fire any events that should fire due to the modification.
                ClipboardEventReaction::new(
                    ClipboardEventFlags::FireClipboardChangedEvent |
                        ClipboardEventFlags::QueueInputEvent,
                )
                .with_input_type(InputType::DeleteByCut)
            },
            "paste" => {
                // These steps are from <https://www.w3.org/TR/clipboard-apis/#paste-action>:
                let Some(data_transfer) = clipboard_event.get_clipboard_data() else {
                    return ClipboardEventReaction::empty();
                };
                let Some(drag_data_store) = data_transfer.data_store() else {
                    return ClipboardEventReaction::empty();
                };

                // Step 3.1: If there is a selection or cursor in an editable context where pasting is
                // enabled, then:
                // TODO: Our TextInput always has a selection or an input point. It's likely that this
                // shouldn't be the case when the entry loses the cursor.

                // Step 3.1.1: Insert the most suitable content found on the clipboard, if any, into the
                // context.
                // TODO: Only text content is currently supported, but other data types should be supported
                // in the future.
                let Some(text_content) =
                    drag_data_store
                        .iter_item_list()
                        .find_map(|item| match item {
                            Kind::Text { data, .. } => Some(data.to_string()),
                            _ => None,
                        })
                else {
                    return ClipboardEventReaction::empty();
                };
                if text_content.is_empty() {
                    return ClipboardEventReaction::empty();
                }

                self.insert_string(&text_content);

                // Step 3.1.2: Queue tasks to fire any events that should fire due to the
                // modification, see § 5.3 Integration with other scripts and events for details.
                ClipboardEventReaction::new(ClipboardEventFlags::QueueInputEvent)
                    .with_text(text_content)
                    .with_input_type(InputType::InsertFromPaste)
            },
        _ => ClipboardEventReaction::empty(),)
    }

    /// <https://w3c.github.io/uievents/#event-type-input>
    pub(crate) fn queue_input_event(
        &self,
        target: &EventTarget,
        data: Option<String>,
        is_composing: IsComposing,
        input_type: InputType,
    ) {
        let global = target.global();
        let target = Trusted::new(target);
        global.task_manager().user_interaction_task_source().queue(
            task!(fire_input_event: move || {
                let target = target.root();
                let global = target.global();
                let window = global.as_window();
                let event = InputEvent::new(
                    window,
                    None,
                    DOMString::from("input"),
                    true,
                    false,
                    Some(window),
                    0,
                    data.map(DOMString::from),
                    is_composing.into(),
                    input_type.as_str().into(),
                    CanGc::note(),
                );
                let event = event.upcast::<Event>();
                event.set_composed(true);
                event.fire(&target, CanGc::note());
            }),
        );
    }

    /// An implementation of
    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-setselectionrange>,
    /// which is used by both `<textarea>` and `<input>`.
    pub(crate) fn dom_set_selection_range(
        &mut self,
        start: Utf16CodeUnitLength,
        end: Utf16CodeUnitLength,
        direction: Option<DOMString>,
    ) -> ErrorResult {
        // Step 2: Set the selection range with the value of this element's selectionStart
        // attribute, the value of this element's selectionEnd attribute, and the given
        // value.
        self.dom_set_selection_range_inner(
            Some(start),
            Some(end),
            direction.map(SelectionDirection::from),
            None,
        );
        Ok(())
    }

    /// An implementation of <https://html.spec.whatwg.org/multipage/#set-the-selection-range>,
    /// which is used by `setSelectionRange()` as well as other parts of the DOM implementation.
    pub(crate) fn dom_set_selection_range_inner(
        &mut self,
        start: Option<Utf16CodeUnitLength>,
        end: Option<Utf16CodeUnitLength>,
        direction: Option<SelectionDirection>,
        original_selection_state: Option<SelectionState>,
    ) {
        let original_selection_state =
            original_selection_state.unwrap_or_else(|| self.selection_state());

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
        self.set_selection_range_utf16(start, end, direction.unwrap_or(SelectionDirection::None));

        // Step 6: If the previous steps caused the selection of the text control to be
        // modified (in either extent or direction), then queue an element task on the
        // user interaction task source given the element to fire an event named select at
        // the element, with the bubbles attribute initialized to true.
        let html_element = self.element.as_element();
        if self.selection_state() != original_selection_state {
            html_element
                .owner_global()
                .task_manager()
                .user_interaction_task_source()
                .queue_event(
                    html_element.upcast(),
                    atom!("select"),
                    EventBubbles::Bubbles,
                    EventCancelable::NotCancelable,
                );
            html_element.upcast::<Node>().dirty(NodeDamage::Other);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-select>
    pub(crate) fn dom_select(&mut self) {
        // Step 2 : Set the selection range with 0 and infinity.
        self.dom_set_selection_range_inner(
            Some(Utf16CodeUnitLength::zero()),
            Some(Utf16CodeUnitLength(usize::MAX)),
            None,
            None,
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart>
    pub(crate) fn dom_get_selection_start(&self) -> Option<Utf16CodeUnitLength> {
        // Step 2: If there is no selection, return the code unit offset within the
        // relevant value to the character that immediately follows the text entry cursor.
        // Step 3: Return the code unit offset within the relevant value to the character
        // that immediately follows the start of the selection.
        Some(self.selection_start_utf16())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart>
    pub(crate) fn dom_set_selection_start(
        &mut self,
        start: Option<Utf16CodeUnitLength>,
    ) -> ErrorResult {
        // Step 2: Let end be the value of this element's selectionEnd attribute.
        let mut end = self.selection_end_utf16();

        // Step 3: If end is less than the given value, set end to the given value.
        match start {
            Some(start) if end < start => end = start,
            _ => {},
        }

        // Step 4: Set the selection range with the given value, end, and the value of
        // this element's selectionDirection attribute.
        self.dom_set_selection_range_inner(
            start,
            Some(end),
            Some(self.selection_direction()),
            None,
        );
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend>
    pub(crate) fn dom_get_selection_end(&self) -> Option<Utf16CodeUnitLength> {
        // Step 2: If there is no selection, return the code unit offset within the
        // relevant value to the character that immediately follows the text entry cursor.
        // Step 3: Return the code unit offset within the relevant value to the character
        // that immediately follows the end of the selection.
        Some(self.selection_end_utf16())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend>
    pub(crate) fn dom_set_selection_end(
        &mut self,
        end: Option<Utf16CodeUnitLength>,
    ) -> ErrorResult {
        // Step 2: Set the selection range with the value of this element's selectionStart
        // attribute, the given value, and the value of this element's selectionDirection
        // attribute.
        self.dom_set_selection_range_inner(
            Some(self.selection_start_utf16()),
            end,
            Some(self.selection_direction()),
            None,
        );
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection>
    pub(crate) fn dom_get_selection_direction(&self) -> Option<DOMString> {
        // Step 2: Return this element's selection direction.
        Some(DOMString::from(self.selection_direction()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection>
    pub(crate) fn dom_set_selection_direction(
        &mut self,
        direction: Option<DOMString>,
    ) -> ErrorResult {
        // Step 2: Set the selection range with the value of this element's selectionStart
        // attribute, the value of this element's selectionEnd attribute, and the given
        // valu
        self.dom_set_selection_range_inner(
            Some(self.selection_start_utf16()),
            Some(self.selection_end_utf16()),
            direction.map(SelectionDirection::from),
            None,
        );
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-setrangetext>
    pub(crate) fn dom_set_range_text(
        &mut self,
        replacement: DOMString,
        start: Option<Utf16CodeUnitLength>,
        end: Option<Utf16CodeUnitLength>,
        selection_mode: SelectionMode,
    ) -> ErrorResult {
        // Step 2: Set this element's dirty value flag to true.
        self.element.set_dirty_value_flag(true);

        // Step 3: If the method has only one argument, then let start and end have the
        // values of the selectionStart attribute and the selectionEnd attribute
        // respectively.
        //
        // Otherwise, let start, end have the values of the second and third arguments
        // respectively.
        let mut selection_start = self.selection_end_utf16();
        let mut selection_end = self.selection_end_utf16();
        let mut start = start.unwrap_or(selection_start);
        let mut end = end.unwrap_or(selection_end);

        // Step 4: If start is greater than end, then throw an "IndexSizeError"
        // DOMException.
        if start > end {
            return Err(Error::IndexSize(None));
        }

        // Save the original selection state to later pass to set_selection_range, because we will
        // change the selection state in order to replace the text in the range.
        let original_selection_state = self.selection_state();

        // Step 5: If start is greater than the length of the relevant value of the text
        // control, then set it to the length of the relevant value of the text control.
        let content_length = self.len_utf16();
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
            self.set_selection_range_utf16(start, end, SelectionDirection::None);
            self.replace_selection(&replacement);
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
        self.dom_set_selection_range_inner(
            Some(selection_start),
            Some(selection_end),
            None,
            Some(original_selection_state),
        );
        Ok(())
    }
}
