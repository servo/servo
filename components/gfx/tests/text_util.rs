/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use gfx::text::util::{is_cjk, transform_text, CompressionMode};

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
        let mut trimmed_str = String::new();
        transform_text(test, mode, true, &mut trimmed_str);
        assert_eq!(trimmed_str, test)
    }
}

#[test]
fn test_transform_discard_newline() {
    let test_strs = [
        ("  foo bar", "  foo bar"),
        ("foo bar  ", "foo bar  "),
        ("foo\n bar", "foo bar"),
        ("foo \nbar", "foo bar"),
        ("  foo  bar  \nbaz", "  foo  bar  baz"),
        ("foo bar baz", "foo bar baz"),
        ("foobarbaz\n\n", "foobarbaz"),
    ];

    let mode = CompressionMode::DiscardNewline;
    for &(test, oracle) in test_strs.iter() {
        let mut trimmed_str = String::new();
        transform_text(test, mode, true, &mut trimmed_str);
        assert_eq!(trimmed_str, oracle)
    }
}

#[test]
fn test_transform_compress_whitespace() {
    let test_strs = [
        ("  foo bar", "foo bar"),
        ("foo bar  ", "foo bar "),
        ("foo\n bar", "foo\n bar"),
        ("foo \nbar", "foo \nbar"),
        ("  foo  bar  \nbaz", "foo bar \nbaz"),
        ("foo bar baz", "foo bar baz"),
        ("foobarbaz\n\n", "foobarbaz\n\n"),
    ];

    let mode = CompressionMode::CompressWhitespace;
    for &(test, oracle) in test_strs.iter() {
        let mut trimmed_str = String::new();
        transform_text(test, mode, true, &mut trimmed_str);
        assert_eq!(&*trimmed_str, oracle)
    }
}

#[test]
fn test_transform_compress_whitespace_newline() {
    let test_strs = vec![
        ("  foo bar", "foo bar"),
        ("foo bar  ", "foo bar "),
        ("foo\n bar", "foo bar"),
        ("foo \nbar", "foo bar"),
        ("  foo  bar  \nbaz", "foo bar baz"),
        ("foo bar baz", "foo bar baz"),
        ("foobarbaz\n\n", "foobarbaz "),
    ];

    let mode = CompressionMode::CompressWhitespaceNewline;
    for &(test, oracle) in test_strs.iter() {
        let mut trimmed_str = String::new();
        transform_text(test, mode, true, &mut trimmed_str);
        assert_eq!(&*trimmed_str, oracle)
    }
}

#[test]
fn test_transform_compress_whitespace_newline_no_incoming() {
    let test_strs = [
        ("  foo bar", " foo bar"),
        ("\nfoo bar", " foo bar"),
        ("foo bar  ", "foo bar "),
        ("foo\n bar", "foo bar"),
        ("foo \nbar", "foo bar"),
        ("  foo  bar  \nbaz", " foo bar baz"),
        ("foo bar baz", "foo bar baz"),
        ("foobarbaz\n\n", "foobarbaz "),
    ];

    let mode = CompressionMode::CompressWhitespaceNewline;
    for &(test, oracle) in test_strs.iter() {
        let mut trimmed_str = String::new();
        transform_text(test, mode, false, &mut trimmed_str);
        assert_eq!(trimmed_str, oracle)
    }
}

#[test]
fn test_is_cjk() {
    // Test characters from different CJK blocks
    assert_eq!(is_cjk('〇'), true);
    assert_eq!(is_cjk('㐀'), true);
    assert_eq!(is_cjk('あ'), true);
    assert_eq!(is_cjk('ア'), true);
    assert_eq!(is_cjk('㆒'), true);
    assert_eq!(is_cjk('ㆣ'), true);
    assert_eq!(is_cjk('龥'), true);
    assert_eq!(is_cjk('𰾑'), true);
    assert_eq!(is_cjk('𰻝'), true);

    // Test characters from outside CJK blocks
    assert_eq!(is_cjk('a'), false);
    assert_eq!(is_cjk('🙂'), false);
    assert_eq!(is_cjk('©'), false);
}
