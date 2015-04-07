// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use gfx::text::util::{CompressionMode, transform_text};

#[test]
fn test_transform_compress_none() {
    let test_strs = [
        "  foo bar",
        "foo bar  ",
        "foo\n bar",
        "foo \nbar",
        "  foo  bar  \nbaz",
        "foo bar baz",
        "foobarbaz\n\n",
    ];

    let mode = CompressionMode::CompressNone;
    for &test in test_strs.iter() {
        let mut new_line_pos = vec![];
        let mut trimmed_str = String::new();
        transform_text(test, mode, true, &mut trimmed_str, &mut new_line_pos);
        assert_eq!(trimmed_str, test)
    }
}

#[test]
fn test_transform_discard_newline() {
    let test_strs = [
        ("  foo bar",
         "  foo bar"),

        ("foo bar  ",
         "foo bar  "),

        ("foo\n bar",
         "foo bar"),

        ("foo \nbar",
         "foo bar"),

        ("  foo  bar  \nbaz",
         "  foo  bar  baz"),

        ("foo bar baz",
         "foo bar baz"),

        ("foobarbaz\n\n",
         "foobarbaz"),
    ];

    let mode = CompressionMode::DiscardNewline;
    for &(test, oracle) in test_strs.iter() {
        let mut new_line_pos = vec![];
        let mut trimmed_str = String::new();
        transform_text(test, mode, true, &mut trimmed_str, &mut new_line_pos);
        assert_eq!(trimmed_str, oracle)
    }
}

#[test]
fn test_transform_compress_whitespace() {
    let test_strs = [
        ("  foo bar",
         "foo bar"),

        ("foo bar  ",
         "foo bar "),

        ("foo\n bar",
         "foo\n bar"),

        ("foo \nbar",
         "foo \nbar"),

        ("  foo  bar  \nbaz",
         "foo bar \nbaz"),

        ("foo bar baz",
         "foo bar baz"),

        ("foobarbaz\n\n",
         "foobarbaz\n\n"),
    ];

    let mode = CompressionMode::CompressWhitespace;
    for &(test, oracle) in test_strs.iter() {
        let mut new_line_pos = vec![];
        let mut trimmed_str = String::new();
        transform_text(test, mode, true, &mut trimmed_str, &mut new_line_pos);
        assert_eq!(&*trimmed_str, oracle)
    }
}

#[test]
fn test_transform_compress_whitespace_newline() {
    let test_strs = vec![
        ("  foo bar",
         "foo bar"),

        ("foo bar  ",
         "foo bar "),

        ("foo\n bar",
         "foo bar"),

        ("foo \nbar",
         "foo bar"),

        ("  foo  bar  \nbaz",
         "foo bar baz"),

        ("foo bar baz",
         "foo bar baz"),

        ("foobarbaz\n\n",
         "foobarbaz "),
    ];

    let mode = CompressionMode::CompressWhitespaceNewline;
    for &(test, oracle) in test_strs.iter() {
        let mut new_line_pos = vec![];
        let mut trimmed_str = String::new();
        transform_text(test, mode, true, &mut trimmed_str, &mut new_line_pos);
        assert_eq!(&*trimmed_str, oracle)
    }
}

#[test]
fn test_transform_compress_whitespace_newline_no_incoming() {
    let test_strs = [
        ("  foo bar",
         " foo bar"),

        ("\nfoo bar",
         " foo bar"),

        ("foo bar  ",
         "foo bar "),

        ("foo\n bar",
         "foo bar"),

        ("foo \nbar",
         "foo bar"),

        ("  foo  bar  \nbaz",
         " foo bar baz"),

        ("foo bar baz",
         "foo bar baz"),

        ("foobarbaz\n\n",
         "foobarbaz "),
    ];

    let mode = CompressionMode::CompressWhitespaceNewline;
    for &(test, oracle) in test_strs.iter() {
        let mut new_line_pos = vec![];
        let mut trimmed_str = String::new();
        transform_text(test, mode, false, &mut trimmed_str, &mut new_line_pos);
        assert_eq!(trimmed_str, oracle)
    }
}
