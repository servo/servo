/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common handling of keyboard input and state management for text input controls

use std::default::Default;
use std::ops::Range;

use base::text::{Utf8CodeUnitLength, Utf16CodeUnitLength};
use base::{Rope, RopeIndex, RopeMovement, RopeSlice};
use bitflags::bitflags;
use keyboard_types::{Key, KeyState, Modifiers, NamedKey, ShortcutMatcher};
use layout_api::wrapper_traits::SelectionDirection;
use script_bindings::codegen::GenericBindings::MouseEventBinding::MouseEventMethods;
use script_bindings::codegen::GenericBindings::UIEventBinding::UIEventMethods;
use script_bindings::match_domstring_ascii;
use script_bindings::trace::CustomTraceable;

use crate::clipboard_provider::ClipboardProvider;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::str::DOMString;
use crate::dom::compositionevent::CompositionEvent;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::inputevent::InputEvent;
use crate::dom::keyboardevent::KeyboardEvent;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::types::{ClipboardEvent, UIEvent};
use crate::drag_data_store::Kind;
use crate::script_runtime::CanGc;

#[derive(Clone, Copy, PartialEq)]
pub enum Selection {
    Selected,
    NotSelected,
}

#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
pub enum Lines {
    Single,
    Multiple,
}

impl Lines {
    fn normalize(&self, contents: impl Into<String>) -> String {
        let contents = contents.into().replace("\r\n", "\n");
        match self {
            Self::Multiple => {
                // https://html.spec.whatwg.org/multipage/#textarea-line-break-normalisation-transformation
                contents.replace("\r", "\n")
            },
            // https://infra.spec.whatwg.org/#strip-newlines
            //
            // Browsers generally seem to convert newlines to spaces, so we do the same.
            Lines::Single => contents.replace(['\r', '\n'], " "),
        }
    }
}

/// Encapsulated state for handling keyboard input in a single or multiline text input control.
#[derive(JSTraceable, MallocSizeOf)]
pub struct TextInput<T: ClipboardProvider> {
    #[no_trace]
    rope: Rope,

    /// The type of [`TextInput`] this is. When in multi-line mode, the [`TextInput`] will
    /// automatically split all inserted text into lines and incorporate them into
    /// the [`Self::rope`]. When in single line mode, the inserted text will be stripped of
    /// newlines.
    mode: Lines,

    /// Current cursor input point
    #[no_trace]
    edit_point: RopeIndex,

    /// The current selection goes from the selection_origin until the edit_point. Note that the
    /// selection_origin may be after the edit_point, in the case of a backward selection.
    #[no_trace]
    selection_origin: Option<RopeIndex>,

    /// The direction that the selection goes in this [`TextInput`]. DOM APIs track this as
    /// a separate field.
    #[no_trace]
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

    /// Whether or not we are currently dragging in this [`TextInput`].
    currently_dragging: bool,
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
    pub fn new(lines: Lines, initial: DOMString, clipboard_provider: T) -> TextInput<T> {
        Self {
            rope: Rope::new(initial),
            mode: lines,
            edit_point: Default::default(),
            selection_origin: None,
            clipboard_provider,
            max_length: Default::default(),
            min_length: Default::default(),
            selection_direction: SelectionDirection::None,
            was_last_change_by_set_content: true,
            currently_dragging: Default::default(),
        }
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

    pub fn set_max_length(&mut self, length: Option<Utf16CodeUnitLength>) {
        self.max_length = length;
    }

    pub fn set_min_length(&mut self, length: Option<Utf16CodeUnitLength>) {
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
        if !self.has_uncollapsed_selection() {
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

    /// Insert a string at the current editing point or replace the selection if
    /// one exists.
    pub fn insert<S: Into<String>>(&mut self, string: S) {
        if self.selection_origin.is_none() {
            self.selection_origin = Some(self.edit_point);
        }
        self.replace_selection(&DOMString::from(string.into()));
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

    /// Whether or not there is an active uncollapsed selection. This means that the
    /// selection origin is set and it differs from the edit point.
    #[inline]
    pub(crate) fn has_uncollapsed_selection(&self) -> bool {
        self.selection_origin
            .is_some_and(|selection_origin| selection_origin != self.edit_point)
    }

    /// Return the selection range as byte offsets from the start of the content.
    ///
    /// If there is no selection, returns an empty range at the edit point.
    pub(crate) fn sorted_selection_offsets_range(&self) -> Range<Utf8CodeUnitLength> {
        self.selection_start_offset()..self.selection_end_offset()
    }

    /// Return the selection range as character offsets from the start of the content.
    ///
    /// If there is no selection, returns an empty range at the edit point.
    pub(crate) fn sorted_selection_character_offsets_range(&self) -> Range<usize> {
        self.rope.index_to_character_offset(self.selection_start())..
            self.rope.index_to_character_offset(self.selection_end())
    }

    // Check that the selection is valid.
    fn assert_ok_selection(&self) {
        debug!(
            "edit_point: {:?}, selection_origin: {:?}, direction: {:?}",
            self.edit_point, self.selection_origin, self.selection_direction
        );

        debug_assert_eq!(self.edit_point, self.rope.normalize_index(self.edit_point));
        if let Some(selection_origin) = self.selection_origin {
            debug_assert_eq!(
                selection_origin,
                self.rope.normalize_index(selection_origin)
            );
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

    /// Replace the current selection with the given [`DOMString`]. If the [`Rope`] is in
    /// single line mode this *will* strip newlines, as opposed to [`Self::set_content`],
    /// which does not.
    pub fn replace_selection(&mut self, insert: &DOMString) {
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
        let string_to_insert = self.mode.normalize(string_to_insert);

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
        if matches!(movement, RopeMovement::Line) || !self.has_uncollapsed_selection() {
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
        match self.mode {
            Lines::Multiple => {
                self.insert('\n');
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
                    self.insert(&text_content);
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
                if let Key::Character(ref character) = key {
                    self.insert(character);
                    return KeyReaction::DispatchInput(
                        Some(character.to_string()),
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
        let insertion = event.data().str();
        if insertion.is_empty() {
            self.clear_selection();
            return KeyReaction::RedrawSelection;
        }

        self.insert(insertion.to_string());
        KeyReaction::DispatchInput(
            Some(insertion.to_string()),
            IsComposing::NotComposing,
            InputType::InsertCompositionText,
        )
    }

    pub(crate) fn handle_compositionupdate(&mut self, event: &CompositionEvent) -> KeyReaction {
        let insertion = event.data().str();
        if insertion.is_empty() {
            return KeyReaction::Nothing;
        }

        let start = self.selection_start_offset();
        let insertion = insertion.to_string();
        self.insert(insertion.clone());
        self.set_selection_range_utf8(
            start,
            start + event.data().len_utf8(),
            SelectionDirection::Forward,
        );
        KeyReaction::DispatchInput(
            Some(insertion),
            IsComposing::Composing,
            InputType::InsertCompositionText,
        )
    }

    fn edit_point_for_mouse_event(&self, node: &Node, event: &MouseEvent) -> RopeIndex {
        node.owner_window()
            .text_index_query_on_node_for_event(node, event)
            .map(|grapheme_index| {
                self.rope.move_by(
                    Default::default(),
                    RopeMovement::Character,
                    grapheme_index as isize,
                )
            })
            .unwrap_or_else(|| self.rope.last_index())
    }

    /// Handle a mouse even that has happened in this [`TextInput`]. Returns `true` if the selection
    /// in the input may have changed and `false` otherwise.
    pub(crate) fn handle_mouse_event(&mut self, node: &Node, mouse_event: &MouseEvent) -> bool {
        // Cancel any ongoing drags if we see a mouseup of any kind or notice
        // that a button other than the primary button is pressed.
        let event_type = mouse_event.upcast::<Event>().type_();
        if event_type == atom!("mouseup") || mouse_event.Buttons() & 1 != 1 {
            self.currently_dragging = false;
        }

        if event_type == atom!("mousedown") {
            return self.handle_mousedown(node, mouse_event);
        }

        if event_type == atom!("mousemove") && self.currently_dragging {
            self.edit_point = self.edit_point_for_mouse_event(node, mouse_event);
            self.update_selection_direction();
            return true;
        }

        false
    }

    /// Handle a "mousedown" event that happened on this [`TextInput`], belonging to the
    /// given [`Node`].
    ///
    /// Returns `true` if the [`TextInput`] changed at all or `false` otherwise.
    fn handle_mousedown(&mut self, node: &Node, mouse_event: &MouseEvent) -> bool {
        assert_eq!(mouse_event.upcast::<Event>().type_(), atom!("mousedown"));

        // Only update the cursor in text fields when the primary buton is pressed.
        //
        // From <https://w3c.github.io/uievents/#dom-mouseevent-button>:
        // > 0 MUST indicate the primary button of the device (in general, the left button
        // > or the only button on single-button devices, used to activate a user interface
        // > control or select text) or the un-initialized value.
        if mouse_event.Button() != 0 {
            return false;
        }

        self.currently_dragging = true;
        match mouse_event.upcast::<UIEvent>().Detail() {
            3 => {
                let word_boundaries = self.rope.line_boundaries(self.edit_point);
                self.edit_point = word_boundaries.end;
                self.selection_origin = Some(word_boundaries.start);
                self.update_selection_direction();
                true
            },
            2 => {
                let word_boundaries = self.rope.relevant_word_boundaries(self.edit_point);
                self.edit_point = word_boundaries.end;
                self.selection_origin = Some(word_boundaries.start);
                self.update_selection_direction();
                true
            },
            1 => {
                self.clear_selection();
                self.edit_point = self.edit_point_for_mouse_event(node, mouse_event);
                self.selection_origin = Some(self.edit_point);
                self.update_selection_direction();
                true
            },
            _ => {
                // We currently don't do anything for higher click counts, but some platforms do.
                // We should re-examine this when implementing support for platform-specific editing
                // behaviors.
                false
            },
        }
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
    ///
    /// Note that when the [`Rope`] is in single line mode, this will **not** strip newlines.
    /// Newline stripping only happens for incremental updates to the [`Rope`] as `<input>`
    /// elements currently need to store unsanitized values while being created.
    pub fn set_content(&mut self, content: DOMString) {
        self.rope = Rope::new(
            content
                .str()
                .to_string()
                .replace("\r\n", "\n")
                .replace("\r", "\n"),
        );
        self.was_last_change_by_set_content = true;

        self.edit_point = self.rope.normalize_index(self.edit_point());
        self.selection_origin = self
            .selection_origin
            .map(|selection_origin| self.rope.normalize_index(selection_origin));
    }

    pub fn set_selection_range_utf16(
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

                self.insert(&text_content);

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
}
