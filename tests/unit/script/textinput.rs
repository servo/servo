// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use base::text::{Utf8CodeUnitLength, Utf16CodeUnitLength};
use base::{RopeIndex, RopeMovement};
use keyboard_types::{Key, Modifiers, NamedKey};
use script::test::DOMString;
use script::test::textinput::{ClipboardProvider, Direction, SelectionDirection, TextInput};
use script::textinput::Lines;

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

fn text_input(lines: Lines, s: &str) -> TextInput<DummyClipboardContext> {
    TextInput::new(lines, DOMString::from(s), DummyClipboardContext::new(""))
}

#[test]
fn test_set_content_ignores_max_length() {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from(""),
        DummyClipboardContext::new(""),
    );

    textinput.set_max_length(Some(Utf16CodeUnitLength::one()));
    textinput.set_content(DOMString::from("mozilla rocks"));
    assert_eq!(textinput.get_content(), DOMString::from("mozilla rocks"));
}

#[test]
fn test_textinput_when_inserting_multiple_lines_over_a_selection_respects_max_length() {
    let mut textinput = TextInput::new(
        Lines::Multiple,
        DOMString::from("hello\nworld"),
        DummyClipboardContext::new(""),
    );

    textinput.set_max_length(Some(Utf16CodeUnitLength(17)));
    textinput.modify_edit_point(1, RopeMovement::Grapheme);
    textinput.modify_selection(3, RopeMovement::Grapheme);
    textinput.modify_selection(1, RopeMovement::Line);

    // Selection is now "hello\n
    //                    ------
    //                   world"
    //                   ----
    textinput.insert("cruel\nterrible\nbad");
    assert_eq!(textinput.get_content(), "hcruel\nterrible\nd");
}

#[test]
fn test_textinput_when_inserting_multiple_lines_still_respects_max_length() {
    let mut textinput = TextInput::new(
        Lines::Multiple,
        DOMString::from("hello\nworld"),
        DummyClipboardContext::new(""),
    );

    textinput.set_max_length(Some(Utf16CodeUnitLength(17)));
    textinput.modify_edit_point(1, RopeMovement::Line);
    textinput.insert("cruel\nterrible");
    assert_eq!(textinput.get_content(), "hello\ncruel\nworld");
}

#[test]
fn test_textinput_when_content_is_already_longer_than_max_length_and_theres_no_selection_dont_insert_anything()
 {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from("abc"),
        DummyClipboardContext::new(""),
    );

    textinput.set_max_length(Some(Utf16CodeUnitLength::one()));
    textinput.insert('a');
    assert_eq!(textinput.get_content(), "abc");
}

#[test]
fn test_multi_line_textinput_with_maxlength_doesnt_allow_appending_characters_when_input_spans_lines()
 {
    let mut textinput = TextInput::new(
        Lines::Multiple,
        DOMString::from("abc\nd"),
        DummyClipboardContext::new(""),
    );

    textinput.set_max_length(Some(Utf16CodeUnitLength(5)));
    textinput.insert('a');
    assert_eq!(textinput.get_content(), "abc\nd");
}

#[test]
fn test_single_line_textinput_with_max_length_doesnt_allow_appending_characters_when_replacing_a_selection()
 {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from("abcde"),
        DummyClipboardContext::new(""),
    );

    textinput.set_max_length(Some(Utf16CodeUnitLength(5)));
    textinput.modify_edit_point(1, RopeMovement::Grapheme);
    textinput.modify_selection(3, RopeMovement::Grapheme);

    // Selection is now "abcde"
    //                    ---

    textinput.replace_selection(&DOMString::from("too long"));

    assert_eq!(textinput.get_content(), "atooe");
}

#[test]
fn test_single_line_textinput_with_max_length_allows_deletion_when_replacing_a_selection() {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from("abcde"),
        DummyClipboardContext::new(""),
    );

    textinput.set_max_length(Some(Utf16CodeUnitLength(1)));
    textinput.modify_edit_point(1, RopeMovement::Grapheme);
    textinput.modify_selection(2, RopeMovement::Grapheme);

    // Selection is now "abcde"
    //                    --

    textinput.replace_selection(&DOMString::from("only deletion should be applied"));

    assert_eq!(textinput.get_content(), "ade");
}

#[test]
fn test_single_line_textinput_with_max_length_multibyte() {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from(""),
        DummyClipboardContext::new(""),
    );

    textinput.set_max_length(Some(Utf16CodeUnitLength(2)));
    textinput.insert('á');
    assert_eq!(textinput.get_content(), "á");
    textinput.insert('é');
    assert_eq!(textinput.get_content(), "áé");
    textinput.insert('i');
    assert_eq!(textinput.get_content(), "áé");
}

#[test]
fn test_single_line_textinput_with_max_length_multi_code_unit() {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from(""),
        DummyClipboardContext::new(""),
    );

    textinput.set_max_length(Some(Utf16CodeUnitLength(3)));
    textinput.insert('\u{10437}');
    assert_eq!(textinput.get_content(), "\u{10437}");
    textinput.insert('\u{10437}');
    assert_eq!(textinput.get_content(), "\u{10437}");
    textinput.insert('x');
    assert_eq!(textinput.get_content(), "\u{10437}x");
    textinput.insert('x');
    assert_eq!(textinput.get_content(), "\u{10437}x");
}

#[test]
fn test_single_line_textinput_with_max_length_inside_char() {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from("\u{10437}"),
        DummyClipboardContext::new(""),
    );

    textinput.set_max_length(Some(Utf16CodeUnitLength::one()));
    textinput.insert('x');
    assert_eq!(textinput.get_content(), "\u{10437}");
}

#[test]
fn test_single_line_textinput_with_max_length_doesnt_allow_appending_characters_after_max_length_is_reached()
 {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from("a"),
        DummyClipboardContext::new(""),
    );

    textinput.set_max_length(Some(Utf16CodeUnitLength::one()));
    textinput.insert('b');
    assert_eq!(textinput.get_content(), "a");
}

#[test]
fn test_textinput_delete_char() {
    let mut textinput = text_input(Lines::Single, "abcdefg");
    textinput.modify_edit_point(2, RopeMovement::Grapheme);
    textinput.delete_char(Direction::Backward);
    assert_eq!(textinput.get_content(), "acdefg");

    textinput.delete_char(Direction::Forward);
    assert_eq!(textinput.get_content(), "adefg");

    textinput.modify_selection(2, RopeMovement::Grapheme);
    textinput.delete_char(Direction::Forward);
    assert_eq!(textinput.get_content(), "afg");

    let mut textinput = text_input(Lines::Single, "a🌠b");
    // Same as "Right" key
    textinput.modify_edit_point(1, RopeMovement::Grapheme);
    textinput.delete_char(Direction::Forward);
    // Not splitting surrogate pairs.
    assert_eq!(textinput.get_content(), "ab");

    let mut textinput = text_input(Lines::Single, "abcdefg");
    textinput.set_selection_range_utf8(
        Utf8CodeUnitLength(2),
        Utf8CodeUnitLength(2),
        SelectionDirection::None,
    );
    textinput.delete_char(Direction::Backward);
    assert_eq!(textinput.get_content(), "acdefg");
}

#[test]
fn test_textinput_insert() {
    let mut textinput = text_input(Lines::Single, "abcdefg");
    textinput.modify_edit_point(2, RopeMovement::Grapheme);
    textinput.insert('a');
    assert_eq!(textinput.get_content(), "abacdefg");

    textinput.modify_selection(2, RopeMovement::Grapheme);
    textinput.insert('b');
    assert_eq!(textinput.get_content(), "ababefg");

    let mut textinput = text_input(Lines::Single, "a🌠c");
    // Same as "Right" key
    textinput.modify_edit_point(2, RopeMovement::Grapheme);
    textinput.insert('b');
    // Not splitting surrogate pairs.
    assert_eq!(textinput.get_content(), "a🌠bc");

    textinput.modify_edit_point(3, RopeMovement::Grapheme);
    textinput.insert("\n1\n2\n3");
    assert_eq!(
        textinput.get_content(),
        "a🌠bc 1 2 3",
        "Newlines should be stripped"
    );
}

#[test]
fn test_textinput_selection_boundaries() {
    let mut textinput = text_input(Lines::Single, "abcdefg");
    textinput.modify_edit_point(2, RopeMovement::Grapheme);
    textinput.modify_selection(2, RopeMovement::Grapheme);
    assert_eq!(textinput.selection_start(), RopeIndex::new(0, 2));
    assert_eq!(textinput.selection_end(), RopeIndex::new(0, 4));

    textinput.clear_selection();
    textinput.modify_selection(-2, RopeMovement::Grapheme);
    assert_eq!(textinput.selection_start(), RopeIndex::new(0, 2));
    assert_eq!(textinput.selection_end(), RopeIndex::new(0, 4));
}

#[test]
fn test_textinput_replace_selection() {
    let mut textinput = text_input(Lines::Single, "abcdefg");
    textinput.modify_edit_point(2, RopeMovement::Grapheme);
    textinput.modify_selection(2, RopeMovement::Grapheme);
    textinput.replace_selection(&DOMString::from("xyz"));
    assert_eq!(textinput.get_content(), "abxyzefg");

    textinput.modify_selection(-3, RopeMovement::Grapheme);
    textinput.replace_selection(&DOMString::from("\n1\n2\r3\r\n4\n"));
    assert_eq!(
        textinput.get_content(),
        "ab 1 2 3 4 efg",
        "Newlines should be stripped"
    );

    let mut textinput = text_input(Lines::Single, "abcdefg");
    textinput.modify_edit_point(2, RopeMovement::Grapheme);
    textinput.modify_selection(0, RopeMovement::Grapheme);
    textinput.replace_selection(&DOMString::from("1\n\n\n\n\n2"));
    assert_eq!(
        textinput.get_content(),
        "ab1     2cdefg",
        "Consecutive newlines should become spaces"
    );
}

#[test]
fn test_textinput_replace_selection_multibyte_char() {
    let mut textinput = text_input(Lines::Single, "é");
    textinput.modify_selection(1, RopeMovement::Grapheme);
    textinput.replace_selection(&DOMString::from("e"));
    assert_eq!(textinput.get_content(), "e");
}

#[test]
fn test_textinput_adjust_vertical() {
    let mut textinput = text_input(Lines::Multiple, "abc\nde\nf");
    textinput.modify_edit_point(3, RopeMovement::Grapheme);
    textinput.modify_edit_point(1, RopeMovement::Line);
    assert_eq!(textinput.edit_point(), RopeIndex::new(1, 2));

    textinput.modify_edit_point(-1, RopeMovement::Line);
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 2));

    textinput.modify_edit_point(2, RopeMovement::Line);
    assert_eq!(textinput.edit_point(), RopeIndex::new(2, 1));

    textinput.modify_edit_point(-1, RopeMovement::Line);
    assert_eq!(textinput.edit_point(), RopeIndex::new(1, 1));
}

#[test]
fn test_textinput_adjust_vertical_multibyte() {
    let mut textinput = text_input(Lines::Multiple, "áé\nae");
    textinput.modify_edit_point(1, RopeMovement::Grapheme);
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 2));

    textinput.modify_edit_point(1, RopeMovement::Line);
    assert_eq!(textinput.edit_point(), RopeIndex::new(1, 1));
}

#[test]
fn test_textinput_adjust_horizontal() {
    let mut textinput = text_input(Lines::Multiple, "abc\nde\nf");
    textinput.modify_edit_point(4, RopeMovement::Grapheme);
    assert_eq!(textinput.edit_point(), RopeIndex::new(1, 0));

    textinput.modify_edit_point(1, RopeMovement::Grapheme);
    assert_eq!(textinput.edit_point(), RopeIndex::new(1, 1));

    textinput.modify_edit_point(2, RopeMovement::Grapheme);
    assert_eq!(textinput.edit_point(), RopeIndex::new(2, 0));

    textinput.modify_edit_point(-1, RopeMovement::Grapheme);
    assert_eq!(textinput.edit_point(), RopeIndex::new(1, 2));
}

#[test]
fn test_textinput_adjust_horizontal_by_word() {
    // Test basic case of movement word by word based on UAX#29 rules
    let mut textinput = text_input(Lines::Single, "abc def");
    textinput.modify_edit_point(2, RopeMovement::Word);
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 7));
    textinput.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 4));
    textinput.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 0));

    // Test new line case of movement word by word based on UAX#29 rules
    let mut textinput_2 = text_input(Lines::Multiple, "abc\ndef");
    textinput_2.modify_edit_point(2, RopeMovement::Word);
    assert_eq!(textinput_2.edit_point(), RopeIndex::new(1, 3));
    textinput_2.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(textinput_2.edit_point(), RopeIndex::new(1, 0));
    textinput_2.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(textinput_2.edit_point(), RopeIndex::new(0, 0));

    // Test non-standard sized characters case of movement word by word based on UAX#29 rules
    let mut textinput_3 = text_input(Lines::Single, "áéc d🌠bc");
    textinput_3.modify_edit_point(1, RopeMovement::Word);
    assert_eq!(textinput_3.edit_point(), RopeIndex::new(0, 5));
    textinput_3.modify_edit_point(1, RopeMovement::Word);
    assert_eq!(textinput_3.edit_point(), RopeIndex::new(0, 7));
    textinput_3.modify_edit_point(1, RopeMovement::Word);
    assert_eq!(textinput_3.edit_point(), RopeIndex::new(0, 13));
    textinput_3.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(textinput_3.edit_point(), RopeIndex::new(0, 11));
    textinput_3.modify_edit_point(-1, RopeMovement::Word);
    assert_eq!(textinput_3.edit_point(), RopeIndex::new(0, 6));
}

#[test]
fn test_textinput_adjust_horizontal_to_line_end() {
    // Test standard case of movement to end based on UAX#29 rules
    let mut textinput = text_input(Lines::Single, "abc def");
    textinput.modify_edit_point(1, RopeMovement::LineStartOrEnd);
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 7));

    // Test new line case of movement to end based on UAX#29 rules
    let mut textinput_2 = text_input(Lines::Multiple, "abc\ndef");
    textinput_2.modify_edit_point(1, RopeMovement::LineStartOrEnd);
    assert_eq!(textinput_2.edit_point(), RopeIndex::new(0, 3));
    textinput_2.modify_edit_point(1, RopeMovement::LineStartOrEnd);
    assert_eq!(textinput_2.edit_point(), RopeIndex::new(0, 3));
    textinput_2.modify_edit_point(-1, RopeMovement::LineStartOrEnd);
    assert_eq!(textinput_2.edit_point(), RopeIndex::new(0, 0));

    // Test non-standard sized characters case of movement to end based on UAX#29 rules
    let mut textinput_3 = text_input(Lines::Single, "áéc d🌠bc");
    textinput_3.modify_edit_point(1, RopeMovement::LineStartOrEnd);
    assert_eq!(textinput_3.edit_point(), RopeIndex::new(0, 13));
    textinput_3.modify_edit_point(-1, RopeMovement::LineStartOrEnd);
    assert_eq!(textinput_3.edit_point(), RopeIndex::new(0, 0));
}

#[test]
fn test_navigation_keyboard_shortcuts() {
    let mut textinput = text_input(Lines::Multiple, "hello áéc");

    // Test that CMD + Right moves to the end of the current line.
    textinput.handle_keydown_aux(Key::Named(NamedKey::ArrowRight), Modifiers::META, true);
    assert_eq!(textinput.edit_point().code_point, 11);
    // Test that CMD + Right moves to the beginning of the current line.
    textinput.handle_keydown_aux(Key::Named(NamedKey::ArrowLeft), Modifiers::META, true);
    assert_eq!(textinput.edit_point().code_point, 0);
    // Test that CTRL + ALT + E moves to the end of the current line also.
    textinput.handle_keydown_aux(
        Key::Character("e".to_owned()),
        Modifiers::CONTROL | Modifiers::ALT,
        true,
    );
    assert_eq!(textinput.edit_point().code_point, 11);
    // Test that CTRL + ALT + A moves to the beginning of the current line also.
    textinput.handle_keydown_aux(
        Key::Character("a".to_owned()),
        Modifiers::CONTROL | Modifiers::ALT,
        true,
    );
    assert_eq!(textinput.edit_point().code_point, 0);

    // Test that ALT + Right moves to the end of the word.
    textinput.handle_keydown_aux(Key::Named(NamedKey::ArrowRight), Modifiers::ALT, true);
    assert_eq!(textinput.edit_point().code_point, 5);
    // Test that CTRL + ALT + F moves to the end of the word also.
    textinput.handle_keydown_aux(
        Key::Character("f".to_owned()),
        Modifiers::CONTROL | Modifiers::ALT,
        true,
    );
    assert_eq!(textinput.edit_point().code_point, 11);
    // Test that ALT + Left moves to the end of the word.
    textinput.handle_keydown_aux(Key::Named(NamedKey::ArrowLeft), Modifiers::ALT, true);
    assert_eq!(textinput.edit_point().code_point, 6);
    // Test that CTRL + ALT + B moves to the end of the word also.
    textinput.handle_keydown_aux(
        Key::Character("b".to_owned()),
        Modifiers::CONTROL | Modifiers::ALT,
        true,
    );
    assert_eq!(textinput.edit_point().code_point, 0);
}

#[test]
fn test_textinput_handle_return() {
    let mut single_line_textinput = text_input(Lines::Single, "abcdef");
    single_line_textinput.modify_edit_point(3, RopeMovement::Grapheme);
    single_line_textinput.handle_return();
    assert_eq!(single_line_textinput.get_content(), "abcdef");

    let mut multi_line_textinput = text_input(Lines::Multiple, "abcdef");
    multi_line_textinput.modify_edit_point(3, RopeMovement::Grapheme);
    multi_line_textinput.handle_return();
    assert_eq!(multi_line_textinput.get_content(), "abc\ndef");
}

#[test]
fn test_textinput_select_all() {
    let mut textinput = text_input(Lines::Multiple, "abc\nde\nf");
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 0));

    textinput.select_all();
    assert_eq!(textinput.edit_point(), RopeIndex::new(2, 1));
}

#[test]
fn test_textinput_get_content() {
    let single_line_textinput = text_input(Lines::Single, "abcdefg");
    assert_eq!(single_line_textinput.get_content(), "abcdefg");

    let multi_line_textinput = text_input(Lines::Multiple, "abc\nde\nf");
    assert_eq!(multi_line_textinput.get_content(), "abc\nde\nf");
}

#[test]
fn test_textinput_set_content() {
    let mut textinput = text_input(Lines::Multiple, "abc\nde\nf");
    assert_eq!(textinput.get_content(), "abc\nde\nf");

    textinput.set_content(DOMString::from("abc\nf"));
    assert_eq!(textinput.get_content(), "abc\nf");
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 0));

    textinput.modify_edit_point(3, RopeMovement::Grapheme);
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 3));

    textinput.set_content(DOMString::from("de"));
    assert_eq!(textinput.get_content(), "de");
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 2));
}

#[test]
fn test_clipboard_paste() {
    #[cfg(target_os = "macos")]
    const MODIFIERS: Modifiers = Modifiers::META;
    #[cfg(not(target_os = "macos"))]
    const MODIFIERS: Modifiers = Modifiers::CONTROL;

    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from("defg"),
        DummyClipboardContext::new("abc"),
    );
    assert_eq!(textinput.get_content(), "defg");
    assert_eq!(textinput.edit_point().code_point, 0);
    textinput.handle_keydown_aux(Key::Character("v".to_owned()), MODIFIERS, false);
    assert_eq!(textinput.get_content(), "abcdefg");
}

#[test]
fn test_textinput_cursor_position_correct_after_clearing_selection() {
    let mut textinput = text_input(Lines::Single, "abcdef");

    // Single line - Forward
    textinput.modify_selection(3, RopeMovement::Grapheme);
    textinput.modify_edit_point(1, RopeMovement::Grapheme);
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 3));

    // Single line - Backward
    textinput.modify_edit_point(-3, RopeMovement::Grapheme);
    textinput.modify_selection(3, RopeMovement::Grapheme);
    textinput.modify_edit_point(-1, RopeMovement::Grapheme);
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 0));

    let mut textinput = text_input(Lines::Multiple, "abc\nde\nf");

    // Multiline - Forward
    textinput.modify_selection(4, RopeMovement::Grapheme);
    textinput.modify_edit_point(1, RopeMovement::Grapheme);
    assert_eq!(textinput.edit_point(), RopeIndex::new(1, 0));

    // Multiline - Backward
    textinput.modify_edit_point(-4, RopeMovement::Grapheme);
    textinput.modify_selection(4, RopeMovement::Grapheme);
    textinput.modify_edit_point(-1, RopeMovement::Grapheme);
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 0));
}

#[test]
fn test_textinput_set_selection_with_direction() {
    let mut textinput = text_input(Lines::Single, "abcdef");
    textinput.set_selection_range_utf8(
        Utf8CodeUnitLength(2),
        Utf8CodeUnitLength(6),
        SelectionDirection::Forward,
    );
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 6));
    assert_eq!(textinput.selection_direction(), SelectionDirection::Forward);
    assert!(textinput.selection_origin().is_some());
    assert_eq!(textinput.selection_origin().unwrap(), RopeIndex::new(0, 2));

    textinput.set_selection_range_utf8(
        Utf8CodeUnitLength(2),
        Utf8CodeUnitLength(6),
        SelectionDirection::Backward,
    );
    assert_eq!(textinput.edit_point(), RopeIndex::new(0, 2));
    assert_eq!(
        textinput.selection_direction(),
        SelectionDirection::Backward
    );
    assert!(textinput.selection_origin().is_some());
    assert_eq!(textinput.selection_origin().unwrap(), RopeIndex::new(0, 6));

    textinput = text_input(Lines::Multiple, "\n\n");
    textinput.set_selection_range_utf8(
        Utf8CodeUnitLength(0),
        Utf8CodeUnitLength(1),
        SelectionDirection::Forward,
    );
    assert_eq!(textinput.edit_point(), RopeIndex::new(1, 0));
    assert_eq!(textinput.selection_direction(), SelectionDirection::Forward);
    assert!(textinput.selection_origin().is_some());
    assert_eq!(textinput.selection_origin().unwrap(), RopeIndex::new(0, 0));

    textinput = text_input(Lines::Multiple, "\n");
    textinput.set_selection_range_utf8(
        Utf8CodeUnitLength(0),
        Utf8CodeUnitLength(1),
        SelectionDirection::Forward,
    );

    assert_eq!(textinput.edit_point(), RopeIndex::new(1, 0));
    assert_eq!(textinput.selection_direction(), SelectionDirection::Forward);
    assert!(textinput.selection_origin().is_some());
    assert_eq!(textinput.selection_origin().unwrap(), RopeIndex::new(0, 0));
}

#[test]
fn test_selection_bounds() {
    let mut textinput = text_input(Lines::Single, "abcdef");

    assert_eq!(
        RopeIndex::new(0, 0),
        textinput.selection_origin_or_edit_point()
    );
    assert_eq!(RopeIndex::new(0, 0), textinput.selection_start());
    assert_eq!(RopeIndex::new(0, 0), textinput.selection_end());

    textinput.set_selection_range_utf8(
        Utf8CodeUnitLength(2),
        Utf8CodeUnitLength(5),
        SelectionDirection::Forward,
    );
    assert_eq!(
        RopeIndex::new(0, 2),
        textinput.selection_origin_or_edit_point()
    );
    assert_eq!(RopeIndex::new(0, 2), textinput.selection_start());
    assert_eq!(RopeIndex::new(0, 5), textinput.selection_end());

    textinput.set_selection_range_utf8(
        Utf8CodeUnitLength(3),
        Utf8CodeUnitLength(6),
        SelectionDirection::Backward,
    );
    assert_eq!(
        RopeIndex::new(0, 6),
        textinput.selection_origin_or_edit_point()
    );
    assert_eq!(RopeIndex::new(0, 3), textinput.selection_start());
    assert_eq!(RopeIndex::new(0, 6), textinput.selection_end());

    textinput = text_input(Lines::Multiple, "\n\n");
    textinput.set_selection_range_utf8(
        Utf8CodeUnitLength(0),
        Utf8CodeUnitLength(1),
        SelectionDirection::Forward,
    );
    assert_eq!(
        RopeIndex::new(0, 0),
        textinput.selection_origin_or_edit_point()
    );
    assert_eq!(RopeIndex::new(0, 0), textinput.selection_start());
    assert_eq!(RopeIndex::new(1, 0), textinput.selection_end());
}

#[test]
fn test_select_all() {
    let mut textinput = text_input(Lines::Single, "abc");
    textinput.set_selection_range_utf8(
        Utf8CodeUnitLength(2),
        Utf8CodeUnitLength(3),
        SelectionDirection::Backward,
    );
    textinput.select_all();
    assert_eq!(textinput.selection_direction(), SelectionDirection::Forward);
    assert_eq!(RopeIndex::new(0, 0), textinput.selection_start());
    assert_eq!(RopeIndex::new(0, 3), textinput.selection_end());
}

#[test]
fn test_backspace_in_textarea_at_beginning_of_line() {
    let mut textinput = text_input(Lines::Multiple, "first line\n");
    textinput.handle_keydown_aux(Key::Named(NamedKey::ArrowDown), Modifiers::empty(), false);
    textinput.handle_keydown_aux(Key::Named(NamedKey::Backspace), Modifiers::empty(), false);
    assert_eq!(textinput.get_content(), DOMString::from("first line"));
}
