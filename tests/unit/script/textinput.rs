// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use embedder_traits::{EditingAction, EditingMotion, ModifySelection};
use script::test::DOMString;
use script::test::text_input::{
    ClipboardProvider, EditingDirection, History, Lines, SelectionDirection, TextInput,
};
use servo_base::text::{Utf8CodeUnits, Utf16CodeUnits};
use servo_base::{RopeIndex, RopeMovement};

pub struct DummyClipboardContext {
    content: String,
}

impl DummyClipboardContext {
    pub fn new(s: &str) -> DummyClipboardContext {
        DummyClipboardContext {
            content: s.to_owned(),
        }
    }
}

impl ClipboardProvider for DummyClipboardContext {
    fn get_text(&mut self) -> Result<String, String> {
        Ok(self.content.clone())
    }
    fn set_text(&mut self, s: String) {
        self.content = s;
    }
}

fn make_text_input(lines: Lines, s: &str) -> TextInput<DummyClipboardContext> {
    TextInput::new(lines, DOMString::from(s), DummyClipboardContext::new(""))
}

#[test]
fn test_set_content_ignores_max_length() {
    let mut text_input = TextInput::new(
        Lines::Single,
        DOMString::from(""),
        DummyClipboardContext::new(""),
    );

    text_input.set_max_length(Some(Utf16CodeUnits(1)));
    text_input.set_content(DOMString::from("mozilla rocks"));
    assert_eq!(text_input.get_content(), DOMString::from("mozilla rocks"));
}

#[test]
fn test_text_input_when_inserting_multiple_lines_over_a_selection_respects_max_length() {
    let mut text_input = TextInput::new(
        Lines::Multiple,
        DOMString::from("hello\nworld"),
        DummyClipboardContext::new(""),
    );

    text_input.set_max_length(Some(Utf16CodeUnits(17)));
    text_input.modify_edit_point(1, RopeMovement::Grapheme);
    text_input.modify_selection(3, RopeMovement::Grapheme);
    text_input.modify_selection(1, RopeMovement::Line);

    // Selection is now "hello\n
    //                    ------
    //                   world"
    //                   ----
    text_input.insert("cruel\nterrible\nbad", History::Nothing);
    assert_eq!(text_input.get_content(), "hcruel\nterrible\nd");
}

#[test]
fn test_text_input_when_inserting_multiple_lines_still_respects_max_length() {
    let mut text_input = TextInput::new(
        Lines::Multiple,
        DOMString::from("hello\nworld"),
        DummyClipboardContext::new(""),
    );

    text_input.set_max_length(Some(Utf16CodeUnits(17)));
    text_input.modify_edit_point(1, RopeMovement::Line);
    text_input.insert("cruel\nterrible", History::Nothing);
    assert_eq!(text_input.get_content(), "hello\ncruel\nworld");
}

#[test]
fn test_text_input_when_content_is_already_longer_than_max_length_and_theres_no_selection_dont_insert_anything()
 {
    let mut text_input = TextInput::new(
        Lines::Single,
        DOMString::from("abc"),
        DummyClipboardContext::new(""),
    );

    text_input.set_max_length(Some(Utf16CodeUnits(1)));
    text_input.insert('a', History::Nothing);
    assert_eq!(text_input.get_content(), "abc");
}

#[test]
fn test_multi_line_text_input_with_maxlength_doesnt_allow_appending_characters_when_input_spans_lines()
 {
    let mut text_input = TextInput::new(
        Lines::Multiple,
        DOMString::from("abc\nd"),
        DummyClipboardContext::new(""),
    );

    text_input.set_max_length(Some(Utf16CodeUnits(5)));
    text_input.insert('a', History::Nothing);
    assert_eq!(text_input.get_content(), "abc\nd");
}

#[test]
fn test_single_line_text_input_with_max_length_doesnt_allow_appending_characters_when_replacing_a_selection()
 {
    let mut text_input = TextInput::new(
        Lines::Single,
        DOMString::from("abcde"),
        DummyClipboardContext::new(""),
    );

    text_input.set_max_length(Some(Utf16CodeUnits(5)));
    text_input.modify_edit_point(1, RopeMovement::Grapheme);
    text_input.modify_selection(3, RopeMovement::Grapheme);

    // Selection is now "abcde"
    //                    ---

    text_input.replace_selection(&DOMString::from("too long"), History::Nothing);

    assert_eq!(text_input.get_content(), "atooe");
}

#[test]
fn test_single_line_text_input_with_max_length_allows_deletion_when_replacing_a_selection() {
    let mut text_input = TextInput::new(
        Lines::Single,
        DOMString::from("abcde"),
        DummyClipboardContext::new(""),
    );

    text_input.set_max_length(Some(Utf16CodeUnits(1)));
    text_input.modify_edit_point(1, RopeMovement::Grapheme);
    text_input.modify_selection(2, RopeMovement::Grapheme);

    // Selection is now "abcde"
    //                    --

    text_input.replace_selection(
        &DOMString::from("only deletion should be applied"),
        History::Nothing,
    );

    assert_eq!(text_input.get_content(), "ade");
}

#[test]
fn test_single_line_text_input_with_max_length_multibyte() {
    let mut text_input = TextInput::new(
        Lines::Single,
        DOMString::from(""),
        DummyClipboardContext::new(""),
    );

    text_input.set_max_length(Some(Utf16CodeUnits(2)));
    text_input.insert('á', History::Nothing);
    assert_eq!(text_input.get_content(), "á");
    text_input.insert('é', History::Nothing);
    assert_eq!(text_input.get_content(), "áé");
    text_input.insert('i', History::Nothing);
    assert_eq!(text_input.get_content(), "áé");
}

#[test]
fn test_single_line_text_input_with_max_length_multi_code_unit() {
    let mut text_input = TextInput::new(
        Lines::Single,
        DOMString::from(""),
        DummyClipboardContext::new(""),
    );

    text_input.set_max_length(Some(Utf16CodeUnits(3)));
    text_input.insert('\u{10437}', History::Nothing);
    assert_eq!(text_input.get_content(), "\u{10437}");
    text_input.insert('\u{10437}', History::Nothing);
    assert_eq!(text_input.get_content(), "\u{10437}");
    text_input.insert('x', History::Nothing);
    assert_eq!(text_input.get_content(), "\u{10437}x");
    text_input.insert('x', History::Nothing);
    assert_eq!(text_input.get_content(), "\u{10437}x");
}

#[test]
fn test_single_line_text_input_with_max_length_inside_char() {
    let mut text_input = TextInput::new(
        Lines::Single,
        DOMString::from("\u{10437}"),
        DummyClipboardContext::new(""),
    );

    text_input.set_max_length(Some(Utf16CodeUnits(1)));
    text_input.insert('x', History::Nothing);
    assert_eq!(text_input.get_content(), "\u{10437}");
}

#[test]
fn test_single_line_text_input_with_max_length_doesnt_allow_appending_characters_after_max_length_is_reached()
 {
    let mut text_input = TextInput::new(
        Lines::Single,
        DOMString::from("a"),
        DummyClipboardContext::new(""),
    );

    text_input.set_max_length(Some(Utf16CodeUnits(1)));
    text_input.insert('b', History::Nothing);
    assert_eq!(text_input.get_content(), "a");
}

#[test]
fn test_text_input_delete_char() {
    let mut text_input = make_text_input(Lines::Single, "abcdefg");
    text_input.modify_edit_point(2, RopeMovement::Grapheme);
    text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Backward,
        History::Nothing,
    );
    assert_eq!(text_input.get_content(), "acdefg");

    text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Forward,
        History::Nothing,
    );
    assert_eq!(text_input.get_content(), "adefg");

    text_input.modify_selection(2, RopeMovement::Grapheme);
    text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Forward,
        History::Nothing,
    );
    assert_eq!(text_input.get_content(), "afg");

    let mut text_input = make_text_input(Lines::Single, "a🌠b");
    // Same as "Right" key
    text_input.modify_edit_point(1, RopeMovement::Grapheme);
    text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Forward,
        History::Nothing,
    );
    // Not splitting surrogate pairs.
    assert_eq!(text_input.get_content(), "ab");

    let mut text_input = make_text_input(Lines::Single, "abcdefg");
    text_input.set_selection_range_utf8(
        Utf8CodeUnits(2),
        Utf8CodeUnits(2),
        SelectionDirection::None,
    );
    text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Backward,
        History::Nothing,
    );
    assert_eq!(text_input.get_content(), "acdefg");
}

#[test]
fn test_text_input_insert() {
    let mut text_input = make_text_input(Lines::Single, "abcdefg");
    text_input.modify_edit_point(2, RopeMovement::Grapheme);
    text_input.insert('a', History::Nothing);
    assert_eq!(text_input.get_content(), "abacdefg");

    text_input.modify_selection(2, RopeMovement::Grapheme);
    text_input.insert('b', History::Nothing);
    assert_eq!(text_input.get_content(), "ababefg");

    let mut text_input = make_text_input(Lines::Single, "a🌠c");
    // Same as "Right" key
    text_input.modify_edit_point(2, RopeMovement::Grapheme);
    text_input.insert('b', History::Nothing);
    // Not splitting surrogate pairs.
    assert_eq!(text_input.get_content(), "a🌠bc");

    text_input.modify_edit_point(3, RopeMovement::Grapheme);
    text_input.insert("\n1\n2\n3", History::Nothing);
    assert_eq!(
        text_input.get_content(),
        "a🌠bc 1 2 3",
        "Newlines should be stripped"
    );
}

#[test]
fn test_text_input_selection_boundaries() {
    let mut text_input = make_text_input(Lines::Single, "abcdefg");
    text_input.modify_edit_point(2, RopeMovement::Grapheme);
    text_input.modify_selection(2, RopeMovement::Grapheme);
    assert_eq!(text_input.selection_start(), RopeIndex::new(0, 2));
    assert_eq!(text_input.selection_end(), RopeIndex::new(0, 4));

    text_input.clear_selection();
    text_input.modify_selection(-2, RopeMovement::Grapheme);
    assert_eq!(text_input.selection_start(), RopeIndex::new(0, 2));
    assert_eq!(text_input.selection_end(), RopeIndex::new(0, 4));
}

#[test]
fn test_text_input_replace_selection() {
    let mut text_input = make_text_input(Lines::Single, "abcdefg");
    text_input.modify_edit_point(2, RopeMovement::Grapheme);
    text_input.modify_selection(2, RopeMovement::Grapheme);
    text_input.replace_selection(&DOMString::from("xyz"), History::Nothing);
    assert_eq!(text_input.get_content(), "abxyzefg");

    text_input.modify_selection(-3, RopeMovement::Grapheme);
    text_input.replace_selection(&DOMString::from("\n1\n2\r3\r\n4\n"), History::Nothing);
    assert_eq!(
        text_input.get_content(),
        "ab 1 2 3 4 efg",
        "Newlines should be stripped"
    );

    let mut text_input = make_text_input(Lines::Single, "abcdefg");
    text_input.modify_edit_point(2, RopeMovement::Grapheme);
    text_input.modify_selection(0, RopeMovement::Grapheme);
    text_input.replace_selection(&DOMString::from("1\n\n\n\n\n2"), History::Nothing);
    assert_eq!(
        text_input.get_content(),
        "ab1     2cdefg",
        "Consecutive newlines should become spaces"
    );
}

#[test]
fn test_text_input_replace_selection_multibyte_char() {
    let mut text_input = make_text_input(Lines::Single, "é");
    text_input.modify_selection(1, RopeMovement::Grapheme);
    text_input.replace_selection(&DOMString::from("e"), History::Nothing);
    assert_eq!(text_input.get_content(), "e");
}

#[test]
fn test_text_input_adjust_vertical() {
    let mut text_input = make_text_input(Lines::Multiple, "abc\nde\nf");
    text_input.modify_edit_point(3, RopeMovement::Grapheme);
    text_input.modify_edit_point(1, RopeMovement::Line);
    assert_eq!(text_input.edit_point(), RopeIndex::new(1, 2));

    text_input.modify_edit_point(-1, RopeMovement::Line);
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 2));

    text_input.modify_edit_point(2, RopeMovement::Line);
    assert_eq!(text_input.edit_point(), RopeIndex::new(2, 1));

    text_input.modify_edit_point(-1, RopeMovement::Line);
    assert_eq!(text_input.edit_point(), RopeIndex::new(1, 1));
}

#[test]
fn test_text_input_adjust_vertical_multibyte() {
    let mut text_input = make_text_input(Lines::Multiple, "áé\nae");
    text_input.modify_edit_point(1, RopeMovement::Grapheme);
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 2));

    text_input.modify_edit_point(1, RopeMovement::Line);
    assert_eq!(text_input.edit_point(), RopeIndex::new(1, 1));
}

#[test]
fn test_text_input_adjust_horizontal() {
    let mut text_input = make_text_input(Lines::Multiple, "abc\nde\nf");
    text_input.modify_edit_point(4, RopeMovement::Grapheme);
    assert_eq!(text_input.edit_point(), RopeIndex::new(1, 0));

    text_input.modify_edit_point(1, RopeMovement::Grapheme);
    assert_eq!(text_input.edit_point(), RopeIndex::new(1, 1));

    text_input.modify_edit_point(2, RopeMovement::Grapheme);
    assert_eq!(text_input.edit_point(), RopeIndex::new(2, 0));

    text_input.modify_edit_point(-1, RopeMovement::Grapheme);
    assert_eq!(text_input.edit_point(), RopeIndex::new(1, 2));
}

#[test]
fn test_text_input_adjust_horizontal_by_word() {
    // Test basic case of movement word by word based on UAX#29 rules
    let mut text_input = make_text_input(Lines::Single, "abc def");
    text_input.modify_edit_point(2, RopeMovement::Word);
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 7));
    text_input.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 4));
    text_input.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 0));

    // Test new line case of movement word by word based on UAX#29 rules
    let mut text_input_2 = make_text_input(Lines::Multiple, "abc\ndef");
    text_input_2.modify_edit_point(2, RopeMovement::Word);
    assert_eq!(text_input_2.edit_point(), RopeIndex::new(1, 3));
    text_input_2.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(text_input_2.edit_point(), RopeIndex::new(1, 0));
    text_input_2.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(text_input_2.edit_point(), RopeIndex::new(0, 0));

    // Test non-standard sized characters case of movement word by word based on UAX#29 rules
    let mut text_input_3 = make_text_input(Lines::Single, "áéc d🌠bc");
    text_input_3.modify_edit_point(1, RopeMovement::Word);
    assert_eq!(text_input_3.edit_point(), RopeIndex::new(0, 5));
    text_input_3.modify_edit_point(1, RopeMovement::Word);
    assert_eq!(text_input_3.edit_point(), RopeIndex::new(0, 7));
    text_input_3.modify_edit_point(1, RopeMovement::Word);
    assert_eq!(text_input_3.edit_point(), RopeIndex::new(0, 13));
    text_input_3.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(text_input_3.edit_point(), RopeIndex::new(0, 11));
    text_input_3.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(text_input_3.edit_point(), RopeIndex::new(0, 6));
}

#[test]
fn test_text_input_adjust_horizontal_to_line_end() {
    // Test standard case of movement to end based on UAX#29 rules
    let mut text_input = make_text_input(Lines::Single, "abc def");
    text_input.modify_edit_point(1, RopeMovement::LineStartOrEnd);
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 7));

    // Test new line case of movement to end based on UAX#29 rules
    let mut text_input_2 = make_text_input(Lines::Multiple, "abc\ndef");
    text_input_2.modify_edit_point(1, RopeMovement::LineStartOrEnd);
    assert_eq!(text_input_2.edit_point(), RopeIndex::new(0, 3));
    text_input_2.modify_edit_point(1, RopeMovement::LineStartOrEnd);
    assert_eq!(text_input_2.edit_point(), RopeIndex::new(0, 3));
    text_input_2.modify_edit_point(-1, RopeMovement::LineStartOrEnd);
    assert_eq!(text_input_2.edit_point(), RopeIndex::new(0, 0));

    // Test non-standard sized characters case of movement to end based on UAX#29 rules
    let mut text_input_3 = make_text_input(Lines::Single, "áéc d🌠bc");
    text_input_3.modify_edit_point(1, RopeMovement::LineStartOrEnd);
    assert_eq!(text_input_3.edit_point(), RopeIndex::new(0, 13));
    text_input_3.modify_edit_point(-1, RopeMovement::LineStartOrEnd);
    assert_eq!(text_input_3.edit_point(), RopeIndex::new(0, 0));
}

#[test]
fn test_perform_editing_action() {
    let mut text_input = make_text_input(Lines::Multiple, "hello áéc");

    // Test moving to the end of the current line.
    text_input.perform_editing_action(EditingAction::MoveCursor(
        EditingDirection::Forward,
        EditingMotion::LineStartOrEnd,
        ModifySelection::Yes,
    ));
    assert_eq!(text_input.edit_point().code_point, 11);

    // Test moving to the start of the current line.
    text_input.perform_editing_action(EditingAction::MoveCursor(
        EditingDirection::Backward,
        EditingMotion::LineStartOrEnd,
        ModifySelection::Yes,
    ));
    assert_eq!(text_input.edit_point().code_point, 0);

    // Test moving to the end of the word.
    text_input.perform_editing_action(EditingAction::MoveCursor(
        EditingDirection::Forward,
        EditingMotion::Word,
        ModifySelection::Yes,
    ));
    assert_eq!(text_input.edit_point().code_point, 5);

    text_input.perform_editing_action(EditingAction::MoveCursor(
        EditingDirection::Forward,
        EditingMotion::Word,
        ModifySelection::Yes,
    ));
    assert_eq!(text_input.edit_point().code_point, 11);

    // Test moving to the start of the word.
    text_input.perform_editing_action(EditingAction::MoveCursor(
        EditingDirection::Backward,
        EditingMotion::Word,
        ModifySelection::Yes,
    ));
    assert_eq!(text_input.edit_point().code_point, 6);

    text_input.perform_editing_action(EditingAction::MoveCursor(
        EditingDirection::Backward,
        EditingMotion::Word,
        ModifySelection::Yes,
    ));
    assert_eq!(text_input.edit_point().code_point, 0);
}

#[test]
fn test_text_input_handle_return() {
    let mut single_line_text_input = make_text_input(Lines::Single, "abcdef");
    single_line_text_input.modify_edit_point(3, RopeMovement::Grapheme);
    single_line_text_input.handle_return();
    assert_eq!(single_line_text_input.get_content(), "abcdef");

    let mut multi_line_text_input = make_text_input(Lines::Multiple, "abcdef");
    multi_line_text_input.modify_edit_point(3, RopeMovement::Grapheme);
    multi_line_text_input.handle_return();
    assert_eq!(multi_line_text_input.get_content(), "abc\ndef");
}

#[test]
fn test_text_input_select_all() {
    let mut text_input = make_text_input(Lines::Multiple, "abc\nde\nf");
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 0));

    text_input.select_all();
    assert_eq!(text_input.edit_point(), RopeIndex::new(2, 1));
}

#[test]
fn test_text_input_get_content() {
    let single_line_text_input = make_text_input(Lines::Single, "abcdefg");
    assert_eq!(single_line_text_input.get_content(), "abcdefg");

    let multi_line_text_input = make_text_input(Lines::Multiple, "abc\nde\nf");
    assert_eq!(multi_line_text_input.get_content(), "abc\nde\nf");
}

#[test]
fn test_text_input_set_content() {
    let mut text_input = make_text_input(Lines::Multiple, "abc\nde\nf");
    assert_eq!(text_input.get_content(), "abc\nde\nf");

    text_input.set_content(DOMString::from("abc\nf"));
    assert_eq!(text_input.get_content(), "abc\nf");
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 0));

    text_input.modify_edit_point(3, RopeMovement::Grapheme);
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 3));

    text_input.set_content(DOMString::from("de"));
    assert_eq!(text_input.get_content(), "de");
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 2));
}

#[test]
fn test_text_input_cursor_position_correct_after_clearing_selection() {
    let mut text_input = make_text_input(Lines::Single, "abcdef");

    // Single line - Forward
    text_input.modify_selection(3, RopeMovement::Grapheme);
    text_input.modify_edit_point(1, RopeMovement::Grapheme);
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 3));

    // Single line - Backward
    text_input.modify_edit_point(-3, RopeMovement::Grapheme);
    text_input.modify_selection(3, RopeMovement::Grapheme);
    text_input.modify_edit_point(-1, RopeMovement::Grapheme);
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 0));

    let mut text_input = make_text_input(Lines::Multiple, "abc\nde\nf");

    // Multiline - Forward
    text_input.modify_selection(4, RopeMovement::Grapheme);
    text_input.modify_edit_point(1, RopeMovement::Grapheme);
    assert_eq!(text_input.edit_point(), RopeIndex::new(1, 0));

    // Multiline - Backward
    text_input.modify_edit_point(-4, RopeMovement::Grapheme);
    text_input.modify_selection(4, RopeMovement::Grapheme);
    text_input.modify_edit_point(-1, RopeMovement::Grapheme);
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 0));
}

#[test]
fn test_text_input_set_selection_with_direction() {
    let mut text_input = make_text_input(Lines::Single, "abcdef");
    text_input.set_selection_range_utf8(
        Utf8CodeUnits(2),
        Utf8CodeUnits(6),
        SelectionDirection::Forward,
    );
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 6));
    assert_eq!(
        text_input.selection_direction(),
        SelectionDirection::Forward
    );
    assert!(text_input.selection_origin().is_some());
    assert_eq!(text_input.selection_origin().unwrap(), RopeIndex::new(0, 2));

    text_input.set_selection_range_utf8(
        Utf8CodeUnits(2),
        Utf8CodeUnits(6),
        SelectionDirection::Backward,
    );
    assert_eq!(text_input.edit_point(), RopeIndex::new(0, 2));
    assert_eq!(
        text_input.selection_direction(),
        SelectionDirection::Backward
    );
    assert!(text_input.selection_origin().is_some());
    assert_eq!(text_input.selection_origin().unwrap(), RopeIndex::new(0, 6));

    text_input = make_text_input(Lines::Multiple, "\n\n");
    text_input.set_selection_range_utf8(
        Utf8CodeUnits(0),
        Utf8CodeUnits(1),
        SelectionDirection::Forward,
    );
    assert_eq!(text_input.edit_point(), RopeIndex::new(1, 0));
    assert_eq!(
        text_input.selection_direction(),
        SelectionDirection::Forward
    );
    assert!(text_input.selection_origin().is_some());
    assert_eq!(text_input.selection_origin().unwrap(), RopeIndex::new(0, 0));

    text_input = make_text_input(Lines::Multiple, "\n");
    text_input.set_selection_range_utf8(
        Utf8CodeUnits(0),
        Utf8CodeUnits(1),
        SelectionDirection::Forward,
    );

    assert_eq!(text_input.edit_point(), RopeIndex::new(1, 0));
    assert_eq!(
        text_input.selection_direction(),
        SelectionDirection::Forward
    );
    assert!(text_input.selection_origin().is_some());
    assert_eq!(text_input.selection_origin().unwrap(), RopeIndex::new(0, 0));
}

#[test]
fn test_selection_bounds() {
    let mut text_input = make_text_input(Lines::Single, "abcdef");

    assert_eq!(
        RopeIndex::new(0, 0),
        text_input.selection_origin_or_edit_point()
    );
    assert_eq!(RopeIndex::new(0, 0), text_input.selection_start());
    assert_eq!(RopeIndex::new(0, 0), text_input.selection_end());

    text_input.set_selection_range_utf8(
        Utf8CodeUnits(2),
        Utf8CodeUnits(5),
        SelectionDirection::Forward,
    );
    assert_eq!(
        RopeIndex::new(0, 2),
        text_input.selection_origin_or_edit_point()
    );
    assert_eq!(RopeIndex::new(0, 2), text_input.selection_start());
    assert_eq!(RopeIndex::new(0, 5), text_input.selection_end());

    text_input.set_selection_range_utf8(
        Utf8CodeUnits(3),
        Utf8CodeUnits(6),
        SelectionDirection::Backward,
    );
    assert_eq!(
        RopeIndex::new(0, 6),
        text_input.selection_origin_or_edit_point()
    );
    assert_eq!(RopeIndex::new(0, 3), text_input.selection_start());
    assert_eq!(RopeIndex::new(0, 6), text_input.selection_end());

    text_input = make_text_input(Lines::Multiple, "\n\n");
    text_input.set_selection_range_utf8(
        Utf8CodeUnits(0),
        Utf8CodeUnits(1),
        SelectionDirection::Forward,
    );
    assert_eq!(
        RopeIndex::new(0, 0),
        text_input.selection_origin_or_edit_point()
    );
    assert_eq!(RopeIndex::new(0, 0), text_input.selection_start());
    assert_eq!(RopeIndex::new(1, 0), text_input.selection_end());
}

#[test]
fn test_select_all() {
    let mut text_input = make_text_input(Lines::Single, "abc");
    text_input.set_selection_range_utf8(
        Utf8CodeUnits(2),
        Utf8CodeUnits(3),
        SelectionDirection::Backward,
    );
    text_input.select_all();
    assert_eq!(
        text_input.selection_direction(),
        SelectionDirection::Forward
    );
    assert_eq!(RopeIndex::new(0, 0), text_input.selection_start());
    assert_eq!(RopeIndex::new(0, 3), text_input.selection_end());
}

#[test]
fn test_backspace_in_textarea_at_beginning_of_line() {
    let mut text_input = make_text_input(Lines::Multiple, "first line\n");
    text_input.perform_editing_action(EditingAction::MoveCursor(
        EditingDirection::Forward,
        EditingMotion::Line,
        ModifySelection::No,
    ));
    text_input.perform_editing_action(EditingAction::Backspace(EditingMotion::Grapheme));
    assert_eq!(text_input.get_content(), DOMString::from("first line"));
}

#[test]
fn test_insert_undo_redo() {
    let mut text_input = make_text_input(Lines::Single, "abc");

    // Insert single character
    text_input.insert('d', History::Push);
    assert_eq!(text_input.get_content(), DOMString::from("dabc"));
    assert!(text_input.undo());
    assert_eq!(text_input.get_content(), DOMString::from("abc"));
    assert!(text_input.redo());
    assert_eq!(text_input.get_content(), DOMString::from("dabc"));

    // Insertion with edit point moving way from modified text
    text_input.modify_edit_point(1, RopeMovement::Grapheme);
    text_input.insert('0', History::Push);
    assert_eq!(text_input.get_content(), DOMString::from("da0bc"));
    text_input.modify_edit_point(1, RopeMovement::Word);
    assert!(text_input.undo());
    assert_eq!(text_input.get_content(), DOMString::from("dabc"));
    assert_eq!(
        RopeIndex::new(0, 2),
        text_input.selection_origin_or_edit_point()
    );

    // Reset back to start
    assert!(text_input.undo());
    assert_eq!(text_input.get_content(), DOMString::from("abc"));
    // Shouldn't be able to undo any more
    assert!(!text_input.undo());

    // Insert multi-character string
    text_input.insert("ggg", History::Push);
    assert!(text_input.undo());
    assert_eq!(text_input.get_content(), DOMString::from("abc"));
    assert!(!text_input.undo());
    assert!(text_input.redo());
    // Shouldn't be able to redo any more
    assert!(!text_input.redo());
    assert_eq!(text_input.get_content(), DOMString::from("gggabc"));

    // Insertion with replacement
    text_input.modify_selection(1, RopeMovement::Grapheme);
    text_input.insert("new stuff", History::Push);
    assert!(text_input.undo());
    assert_eq!(text_input.get_content(), DOMString::from("gggabc"));
    assert!(text_input.redo());
    assert!(!text_input.redo());
    assert_eq!(text_input.get_content(), DOMString::from("gggnew stuffbc"));
}

#[test]
fn test_insert_undo_redo_with_merging() {
    let mut text_input = make_text_input(Lines::Single, "");

    // Basic merging
    text_input.insert('a', History::Push);
    text_input.insert('b', History::Push);
    text_input.insert("cd", History::Push);
    assert!(text_input.undo());
    assert!(!text_input.undo(), "Shouldn't be able to undo any ore");
    assert!(
        text_input.get_content().is_empty(),
        "All edits should be undone"
    );
    assert!(text_input.redo());
    assert!(!text_input.redo(), "Shouldn't be able to redo any more");
    assert_eq!(text_input.get_content(), DOMString::from("abcd"));

    // Non-contiguous edits should block merging
    text_input.modify_edit_point(-2, RopeMovement::Grapheme);
    text_input.insert('x', History::Push);
    text_input.insert('y', History::Push);
    assert!(text_input.undo());
    assert_eq!(
        text_input.get_content(),
        DOMString::from("abcd"),
        "Non-contiguous insertion should block merge"
    );
    assert!(text_input.redo());
    assert!(!text_input.redo(), "Shouldn't be able to redo any more");
    assert_eq!(text_input.get_content(), DOMString::from("abxycd"));

    // Reset
    assert!(text_input.undo());
    assert!(text_input.undo());
    assert!(!text_input.undo());
    assert!(text_input.get_content().is_empty());
}

#[test]
fn test_delete_undo_redo() {
    let mut text_input = make_text_input(Lines::Single, "abcd");

    // Single character deletion
    text_input.modify_edit_point(1, RopeMovement::Word);
    assert!(text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Backward,
        History::Push
    ));
    assert!(text_input.undo());
    assert!(!text_input.undo(), "Shouldn't be able to undo any more");
    assert_eq!(text_input.get_content(), DOMString::from("abcd"));
    assert!(text_input.redo());
    assert!(!text_input.redo(), "Shouldn't be able to redo any more");
    assert_eq!(text_input.get_content(), DOMString::from("abc"));

    // Delete three times and merge
    assert!(text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Backward,
        History::Push
    ));
    assert!(text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Backward,
        History::Push
    ));
    assert!(text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Backward,
        History::Push
    ));
    assert!(text_input.undo());
    assert!(!text_input.undo(), "Shouldn't be able to undo any more");
    assert_eq!(
        text_input.get_content(),
        DOMString::from("abcd"),
        "Edits should be merged"
    );

    // Delete selection
    text_input.modify_selection(-3, RopeMovement::Grapheme);
    assert!(text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Backward,
        History::Push
    ));
    assert!(text_input.undo());
    assert_eq!(text_input.get_content(), DOMString::from("abcd"));
    assert_eq!(
        text_input.selection_start(),
        RopeIndex::new(0, 1),
        "Undoing a deletion of a selection should re-make that selection"
    );
    assert_eq!(
        text_input.selection_end(),
        RopeIndex::new(0, 4),
        "Undoing a deletion of a selection should re-make that selection"
    );
    text_input.clear_selection();
    assert!(text_input.redo());
    // Delete one more character
    assert!(text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Backward,
        History::Push
    ));
    assert!(
        text_input.undo(),
        "Should be able to undo the selection deletion"
    );
    assert!(
        text_input.undo(),
        "Selection deletion should block merge, so we should have one more undo"
    );
    assert!(!text_input.undo());
    assert_eq!(text_input.get_content(), DOMString::from("abcd"));
    text_input.modify_selection(-1, RopeMovement::Word);

    // Forward deletion
    assert!(text_input.delete_unit_or_selection(
        RopeMovement::Grapheme,
        EditingDirection::Forward,
        History::Push
    ));
    assert!(text_input.undo());
    assert_eq!(text_input.get_content(), DOMString::from("abcd"));
}
