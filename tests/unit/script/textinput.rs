// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use msg::constellation_msg::{Key, KeyModifiers};
#[cfg(not(target_os = "macos"))]
use msg::constellation_msg::CONTROL;
#[cfg(target_os = "macos")]
use msg::constellation_msg::SUPER;
use script::clipboard_provider::DummyClipboardContext;
use script::test::DOMString;
use script::textinput::{TextInput, TextPoint, Selection, Lines, Direction, SelectionDirection};

fn text_input(lines: Lines, s: &str) -> TextInput<DummyClipboardContext> {
    TextInput::new(lines,
                   DOMString::from(s),
                   DummyClipboardContext::new(""),
                   None,
                   None,
                   SelectionDirection::None)
}

#[test]
fn test_set_content_ignores_max_length() {
    let mut textinput = TextInput::new(
        Lines::Single, DOMString::from(""), DummyClipboardContext::new(""), Some(1), None, SelectionDirection::None
    );

    textinput.set_content(DOMString::from("mozilla rocks"));
    assert_eq!(textinput.get_content(), DOMString::from("mozilla rocks"));
}

#[test]
fn test_textinput_when_inserting_multiple_lines_over_a_selection_respects_max_length() {
    let mut textinput = TextInput::new(
        Lines::Multiple,
        DOMString::from("hello\nworld"),
        DummyClipboardContext::new(""),
        Some(17),
        None,
        SelectionDirection::None,
    );

    textinput.edit_point = TextPoint { line: 0, index: 1 };
    textinput.adjust_horizontal(3, Selection::Selected);
    textinput.adjust_vertical(1, Selection::Selected);

    // Selection is now "hello\n
    //                    ------
    //                   world"
    //                   ----

    textinput.insert_string("cruel\nterrible\nbad".to_string());

    assert_eq!(textinput.get_content(), "hcruel\nterrible\nd");
}

#[test]
fn test_textinput_when_inserting_multiple_lines_still_respects_max_length() {
    let mut textinput = TextInput::new(
        Lines::Multiple,
        DOMString::from("hello\nworld"),
        DummyClipboardContext::new(""),
        Some(17),
        None,
        SelectionDirection::None
    );

    textinput.edit_point = TextPoint { line: 1, index: 0 };

    textinput.insert_string("cruel\nterrible".to_string());

    assert_eq!(textinput.get_content(), "hello\ncruel\nworld");
}

#[test]
fn test_textinput_when_content_is_already_longer_than_max_length_and_theres_no_selection_dont_insert_anything() {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from("abc"),
        DummyClipboardContext::new(""),
        Some(1),
        None,
        SelectionDirection::None,
    );

    textinput.insert_char('a');

    assert_eq!(textinput.get_content(), "abc");
}

#[test]
fn test_multi_line_textinput_with_maxlength_doesnt_allow_appending_characters_when_input_spans_lines() {
    let mut textinput = TextInput::new(
        Lines::Multiple,
        DOMString::from("abc\nd"),
        DummyClipboardContext::new(""),
        Some(5),
        None,
        SelectionDirection::None,
    );

    textinput.insert_char('a');

    assert_eq!(textinput.get_content(), "abc\nd");
}

#[test]
fn test_single_line_textinput_with_max_length_doesnt_allow_appending_characters_when_replacing_a_selection() {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from("abcde"),
        DummyClipboardContext::new(""),
        Some(5),
        None,
        SelectionDirection::None,
    );

    textinput.edit_point = TextPoint { line: 0, index: 1 };
    textinput.adjust_horizontal(3, Selection::Selected);

    // Selection is now "abcde"
    //                    ---

    textinput.replace_selection(DOMString::from("too long"));

    assert_eq!(textinput.get_content(), "atooe");
}

#[test]
fn test_single_line_textinput_with_max_length_multibyte() {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from(""),
        DummyClipboardContext::new(""),
        Some(2),
        None,
        SelectionDirection::None,
    );

    textinput.insert_char('á');
    assert_eq!(textinput.get_content(), "á");
    textinput.insert_char('é');
    assert_eq!(textinput.get_content(), "áé");
    textinput.insert_char('i');
    assert_eq!(textinput.get_content(), "áé");
}

#[test]
fn test_single_line_textinput_with_max_length_multi_code_unit() {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from(""),
        DummyClipboardContext::new(""),
        Some(3),
        None,
        SelectionDirection::None,
    );

    textinput.insert_char('\u{10437}');
    assert_eq!(textinput.get_content(), "\u{10437}");
    textinput.insert_char('\u{10437}');
    assert_eq!(textinput.get_content(), "\u{10437}");
    textinput.insert_char('x');
    assert_eq!(textinput.get_content(), "\u{10437}x");
    textinput.insert_char('x');
    assert_eq!(textinput.get_content(), "\u{10437}x");
}

#[test]
fn test_single_line_textinput_with_max_length_inside_char() {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from("\u{10437}"),
        DummyClipboardContext::new(""),
        Some(1),
        None,
        SelectionDirection::None,
    );

    textinput.insert_char('x');
    assert_eq!(textinput.get_content(), "\u{10437}");
}

#[test]
fn test_single_line_textinput_with_max_length_doesnt_allow_appending_characters_after_max_length_is_reached() {
    let mut textinput = TextInput::new(
        Lines::Single,
        DOMString::from("a"),
        DummyClipboardContext::new(""),
        Some(1),
        None,
        SelectionDirection::None,
    );

    textinput.insert_char('b');
    assert_eq!(textinput.get_content(), "a");
}

#[test]
fn test_textinput_delete_char() {
    let mut textinput = text_input(Lines::Single, "abcdefg");
    textinput.adjust_horizontal(2, Selection::NotSelected);
    textinput.delete_char(Direction::Backward);
    assert_eq!(textinput.get_content(), "acdefg");

    textinput.delete_char(Direction::Forward);
    assert_eq!(textinput.get_content(), "adefg");

    textinput.adjust_horizontal(2, Selection::Selected);
    textinput.delete_char(Direction::Forward);
    assert_eq!(textinput.get_content(), "afg");

    let mut textinput = text_input(Lines::Single, "a🌠b");
    // Same as "Right" key
    textinput.adjust_horizontal_by_one(Direction::Forward, Selection::NotSelected);
    textinput.delete_char(Direction::Forward);
    // Not splitting surrogate pairs.
    assert_eq!(textinput.get_content(), "ab");

    let mut textinput = text_input(Lines::Single, "abcdefg");
    textinput.adjust_horizontal(2, Selection::NotSelected);
    // Set an empty selection range.
    textinput.selection_begin = Some(textinput.edit_point);
    textinput.delete_char(Direction::Backward);
    assert_eq!(textinput.get_content(), "acdefg");
}

#[test]
fn test_textinput_insert_char() {
    let mut textinput = text_input(Lines::Single, "abcdefg");
    textinput.adjust_horizontal(2, Selection::NotSelected);
    textinput.insert_char('a');
    assert_eq!(textinput.get_content(), "abacdefg");

    textinput.adjust_horizontal(2, Selection::Selected);
    textinput.insert_char('b');
    assert_eq!(textinput.get_content(), "ababefg");

    let mut textinput = text_input(Lines::Single, "a🌠c");
    // Same as "Right" key
    textinput.adjust_horizontal_by_one(Direction::Forward, Selection::NotSelected);
    textinput.adjust_horizontal_by_one(Direction::Forward, Selection::NotSelected);
    textinput.insert_char('b');
    // Not splitting surrogate pairs.
    assert_eq!(textinput.get_content(), "a🌠bc");
}

#[test]
fn test_textinput_get_sorted_selection() {
    let mut textinput = text_input(Lines::Single, "abcdefg");
    textinput.adjust_horizontal(2, Selection::NotSelected);
    textinput.adjust_horizontal(2, Selection::Selected);
    let (begin, end) = textinput.get_sorted_selection().unwrap();
    assert_eq!(begin.index, 2);
    assert_eq!(end.index, 4);

    textinput.clear_selection();

    textinput.adjust_horizontal(-2, Selection::Selected);
    let (begin, end) = textinput.get_sorted_selection().unwrap();
    assert_eq!(begin.index, 2);
    assert_eq!(end.index, 4);
}

#[test]
fn test_textinput_replace_selection() {
    let mut textinput = text_input(Lines::Single, "abcdefg");
    textinput.adjust_horizontal(2, Selection::NotSelected);
    textinput.adjust_horizontal(2, Selection::Selected);

    textinput.replace_selection(DOMString::from("xyz"));
    assert_eq!(textinput.get_content(), "abxyzefg");
}

#[test]
fn test_textinput_replace_selection_multibyte_char() {
    let mut textinput = text_input(Lines::Single, "é");
    textinput.adjust_horizontal_by_one(Direction::Forward, Selection::Selected);

    textinput.replace_selection(DOMString::from("e"));
    assert_eq!(textinput.get_content(), "e");
}

#[test]
fn test_textinput_current_line_length() {
    let mut textinput = text_input(Lines::Multiple, "abc\nde\nf");
    assert_eq!(textinput.current_line_length(), 3);

    textinput.adjust_vertical(1, Selection::NotSelected);
    assert_eq!(textinput.current_line_length(), 2);

    textinput.adjust_vertical(1, Selection::NotSelected);
    assert_eq!(textinput.current_line_length(), 1);
}

#[test]
fn test_textinput_adjust_vertical() {
    let mut textinput = text_input(Lines::Multiple, "abc\nde\nf");
    textinput.adjust_horizontal(3, Selection::NotSelected);
    textinput.adjust_vertical(1, Selection::NotSelected);
    assert_eq!(textinput.edit_point.line, 1);
    assert_eq!(textinput.edit_point.index, 2);

    textinput.adjust_vertical(-1, Selection::NotSelected);
    assert_eq!(textinput.edit_point.line, 0);
    assert_eq!(textinput.edit_point.index, 2);

    textinput.adjust_vertical(2, Selection::NotSelected);
    assert_eq!(textinput.edit_point.line, 2);
    assert_eq!(textinput.edit_point.index, 1);
}

#[test]
fn test_textinput_adjust_vertical_multibyte() {
    let mut textinput = text_input(Lines::Multiple, "áé\nae");

    textinput.adjust_horizontal_by_one(Direction::Forward, Selection::NotSelected);
    assert_eq!(textinput.edit_point.line, 0);
    assert_eq!(textinput.edit_point.index, 2);

    textinput.adjust_vertical(1, Selection::NotSelected);
    assert_eq!(textinput.edit_point.line, 1);
    assert_eq!(textinput.edit_point.index, 1);
}

#[test]
fn test_textinput_adjust_horizontal() {
    let mut textinput = text_input(Lines::Multiple, "abc\nde\nf");
    textinput.adjust_horizontal(4, Selection::NotSelected);
    assert_eq!(textinput.edit_point.line, 1);
    assert_eq!(textinput.edit_point.index, 0);

    textinput.adjust_horizontal(1, Selection::NotSelected);
    assert_eq!(textinput.edit_point.line, 1);
    assert_eq!(textinput.edit_point.index, 1);

    textinput.adjust_horizontal(2, Selection::NotSelected);
    assert_eq!(textinput.edit_point.line, 2);
    assert_eq!(textinput.edit_point.index, 0);

    textinput.adjust_horizontal(-1, Selection::NotSelected);
    assert_eq!(textinput.edit_point.line, 1);
    assert_eq!(textinput.edit_point.index, 2);
}

#[test]
fn test_textinput_handle_return() {
    let mut single_line_textinput = text_input(Lines::Single, "abcdef");
    single_line_textinput.adjust_horizontal(3, Selection::NotSelected);
    single_line_textinput.handle_return();
    assert_eq!(single_line_textinput.get_content(), "abcdef");

    let mut multi_line_textinput = text_input(Lines::Multiple, "abcdef");
    multi_line_textinput.adjust_horizontal(3, Selection::NotSelected);
    multi_line_textinput.handle_return();
    assert_eq!(multi_line_textinput.get_content(), "abc\ndef");
}

#[test]
fn test_textinput_select_all() {
    let mut textinput = text_input(Lines::Multiple, "abc\nde\nf");
    assert_eq!(textinput.edit_point.line, 0);
    assert_eq!(textinput.edit_point.index, 0);

    textinput.select_all();
    assert_eq!(textinput.edit_point.line, 2);
    assert_eq!(textinput.edit_point.index, 1);
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

    assert_eq!(textinput.edit_point.line, 0);
    assert_eq!(textinput.edit_point.index, 0);
    textinput.adjust_horizontal(3, Selection::Selected);
    assert_eq!(textinput.edit_point.line, 0);
    assert_eq!(textinput.edit_point.index, 3);
    textinput.set_content(DOMString::from("de"));
    assert_eq!(textinput.get_content(), "de");
    assert_eq!(textinput.edit_point.line, 0);
    assert_eq!(textinput.edit_point.index, 2);
}

#[test]
fn test_clipboard_paste() {
    #[cfg(target_os = "macos")]
    const MODIFIERS: KeyModifiers = SUPER;
    #[cfg(not(target_os = "macos"))]
    const MODIFIERS: KeyModifiers = CONTROL;

    let mut textinput = TextInput::new(Lines::Single,
                                       DOMString::from("defg"),
                                       DummyClipboardContext::new("abc"),
                                       None,
                                       None,
                                       SelectionDirection::None);
    assert_eq!(textinput.get_content(), "defg");
    assert_eq!(textinput.edit_point.index, 0);
    textinput.handle_keydown_aux(Some('v'), Key::V, MODIFIERS);
    assert_eq!(textinput.get_content(), "abcdefg");
}

#[test]
fn test_textinput_cursor_position_correct_after_clearing_selection() {
    let mut textinput = text_input(Lines::Single, "abcdef");

    // Single line - Forward
    textinput.adjust_horizontal(3, Selection::Selected);
    textinput.adjust_horizontal(1, Selection::NotSelected);
    assert_eq!(textinput.edit_point.index, 3);

    textinput.adjust_horizontal(-3, Selection::NotSelected);
    textinput.adjust_horizontal(3, Selection::Selected);
    textinput.adjust_horizontal_by_one(Direction::Forward, Selection::NotSelected);
    assert_eq!(textinput.edit_point.index, 3);

    // Single line - Backward
    textinput.adjust_horizontal(-3, Selection::NotSelected);
    textinput.adjust_horizontal(3, Selection::Selected);
    textinput.adjust_horizontal(-1, Selection::NotSelected);
    assert_eq!(textinput.edit_point.index, 0);

    textinput.adjust_horizontal(-3, Selection::NotSelected);
    textinput.adjust_horizontal(3, Selection::Selected);
    textinput.adjust_horizontal_by_one(Direction::Backward, Selection::NotSelected);
    assert_eq!(textinput.edit_point.index, 0);


    let mut textinput = text_input(Lines::Multiple, "abc\nde\nf");

    // Multiline - Forward
    textinput.adjust_horizontal(4, Selection::Selected);
    textinput.adjust_horizontal(1, Selection::NotSelected);
    assert_eq!(textinput.edit_point.index, 0);
    assert_eq!(textinput.edit_point.line, 1);

    textinput.adjust_horizontal(-4, Selection::NotSelected);
    textinput.adjust_horizontal(4, Selection::Selected);
    textinput.adjust_horizontal_by_one(Direction::Forward, Selection::NotSelected);
    assert_eq!(textinput.edit_point.index, 0);
    assert_eq!(textinput.edit_point.line, 1);

    // Multiline - Backward
    textinput.adjust_horizontal(-4, Selection::NotSelected);
    textinput.adjust_horizontal(4, Selection::Selected);
    textinput.adjust_horizontal(-1, Selection::NotSelected);
    assert_eq!(textinput.edit_point.index, 0);
    assert_eq!(textinput.edit_point.line, 0);

    textinput.adjust_horizontal(-4, Selection::NotSelected);
    textinput.adjust_horizontal(4, Selection::Selected);
    textinput.adjust_horizontal_by_one(Direction::Backward, Selection::NotSelected);
    assert_eq!(textinput.edit_point.index, 0);
    assert_eq!(textinput.edit_point.line, 0);
}


#[test]
fn test_textinput_set_selection_with_direction() {
    let mut textinput = text_input(Lines::Single, "abcdef");
    textinput.selection_direction = SelectionDirection::Forward;
    textinput.set_selection_range(2, 6);
    assert_eq!(textinput.edit_point.line, 0);
    assert_eq!(textinput.edit_point.index, 6);

    assert!(textinput.selection_begin.is_some());
    assert_eq!(textinput.selection_begin.unwrap().line, 0);
    assert_eq!(textinput.selection_begin.unwrap().index, 2);

    textinput.selection_direction = SelectionDirection::Backward;
    textinput.set_selection_range(2, 6);
    assert_eq!(textinput.edit_point.line, 0);
    assert_eq!(textinput.edit_point.index, 2);

    assert!(textinput.selection_begin.is_some());
    assert_eq!(textinput.selection_begin.unwrap().line, 0);
    assert_eq!(textinput.selection_begin.unwrap().index, 6);
}
