/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

enum CompressionMode {
    CompressNone,
    CompressWhitespace,
    CompressWhitespaceNewline,
    DiscardNewline
}

impl Eq for CompressionMode {
    fn eq(&self, other: &CompressionMode) -> bool {
        match (*self, *other) {
            (CompressNone, CompressNone) => true,
            (CompressWhitespace, CompressWhitespace) => true,
            (CompressWhitespaceNewline, CompressWhitespaceNewline) => true,
            (DiscardNewline, DiscardNewline) => true,
            _ => false
        }
    }
    fn ne(&self, other: &CompressionMode) -> bool {
        !(*self).eq(other)
    }
}

// ported from Gecko's nsTextFrameUtils::TransformText. 
// 
// High level TODOs:
//
// * Issue #113: consider incoming text state (arabic, etc)
//               and propogate outgoing text state (dual of above) 
//
// * Issue #114: record skipped and kept chars for mapping original to new text
//
// * Untracked: various edge cases for bidi, CJK, etc.
pub fn transform_text(text: &str, mode: CompressionMode, incoming_whitespace: bool) -> (~str, bool) {
    let mut out_str: ~str = ~"";
    let out_whitespace = match mode {
        CompressNone | DiscardNewline => {
            for str::each_char(text) |ch: char| {
                if is_discardable_char(ch, mode) {
                    // TODO: record skipped char
                } else {
                    // TODO: record kept char
                    if ch == '\t' {
                        // TODO: set "has tab" flag
                    }
                    str::push_char(&mut out_str, ch);
                }
            }
            text.len() > 0 && is_in_whitespace(text.char_at_reverse(0), mode)
        },

        CompressWhitespace | CompressWhitespaceNewline => {
            let mut in_whitespace: bool = incoming_whitespace;
            for str::each_char(text) |ch: char| {
                // TODO: discard newlines between CJK chars
                let mut next_in_whitespace: bool = is_in_whitespace(ch, mode);
                
                if !next_in_whitespace {
                    if is_always_discardable_char(ch) {
                        // revert whitespace setting, since this char was discarded
                        next_in_whitespace = in_whitespace;
                        // TODO: record skipped char
                    } else {
                        // TODO: record kept char
                        str::push_char(&mut out_str, ch);
                    }
                } else { /* next_in_whitespace; possibly add a space char */
                    if in_whitespace {
                        // TODO: record skipped char
                    } else {
                        // TODO: record kept char
                        str::push_char(&mut out_str, ' ');
                    }
                }
                // save whitespace context for next char
                in_whitespace = next_in_whitespace;
            } /* /for str::each_char */
            in_whitespace
        } 
    };

    return (out_str, out_whitespace);

    fn is_in_whitespace(ch: char, mode: CompressionMode) -> bool {
        match (ch, mode) {
            (' ', _)  => true,
            ('\t', _) => true,
            ('\n', CompressWhitespaceNewline) => true,
            (_, _)    => false
        }
    }

    fn is_discardable_char(ch: char, mode: CompressionMode) -> bool {
        if is_always_discardable_char(ch) {
            return true;
        }
        match mode {
            DiscardNewline | CompressWhitespaceNewline => ch == '\n',
            _ => false
        }
    }

    fn is_always_discardable_char(_ch: char) -> bool {
        // TODO: check for bidi control chars, soft hyphens.
        false
    }
}

pub fn float_to_fixed(before: int, f: float) -> i32 {
    (1i32 << before) * (f as i32)
}

pub fn fixed_to_float(before: int, f: i32) -> float {
    f as float * 1.0f / ((1i32 << before) as float)
}

pub fn fixed_to_rounded_int(before: int, f: i32) -> int {
    let half = 1i32 << (before-1);
    if f > 0i32 {
        ((half + f) >> before) as int
    } else {
       -((half - f) >> before) as int
    }
}

/* Generate a 32-bit TrueType tag from its 4 characters */
pub fn true_type_tag(a: char, b: char, c: char, d: char) -> u32 {
    (a << 24 | b << 16 | c << 8 | d) as u32
}

#[test]
fn test_true_type_tag() {
    assert!(true_type_tag('c', 'm', 'a', 'p') == 0x_63_6D_61_70_u32);
}

#[test]
fn test_transform_compress_none() {

    let  test_strs : ~[~str] = ~[~"  foo bar",
                                 ~"foo bar  ",
                                 ~"foo\n bar",
                                 ~"foo \nbar",
                                 ~"  foo  bar  \nbaz",
                                 ~"foo bar baz",
                                 ~"foobarbaz\n\n"];
    let mode = CompressNone;

    for uint::range(0, test_strs.len()) |i| {
        (trimmed_str, _out) = transform_text(test_strs[i], mode, true);
        assert!(trimmed_str == test_strs[i])
    }
}

#[test]
fn test_transform_discard_newline() {

    let  test_strs : ~[~str] = ~[~"  foo bar",
                                 ~"foo bar  ",
                                 ~"foo\n bar",
                                 ~"foo \nbar",
                                 ~"  foo  bar  \nbaz",
                                 ~"foo bar baz",
                                 ~"foobarbaz\n\n"];

    let  oracle_strs : ~[~str] = ~[~"  foo bar",
                                   ~"foo bar  ",
                                   ~"foo bar",
                                   ~"foo bar",
                                   ~"  foo  bar  baz",
                                   ~"foo bar baz",
                                   ~"foobarbaz"];

    assert!(test_strs.len() == oracle_strs.len());
    let mode = DiscardNewline;

    for uint::range(0, test_strs.len()) |i| {
        (trimmed_str, _out) = transform_text(test_strs[i], mode, true);
        assert!(trimmed_str == oracle_strs[i])
    }
}

#[test]
fn test_transform_compress_whitespace() {
    let  test_strs : ~[~str] = ~[~"  foo bar",
                                 ~"foo bar  ",
                                 ~"foo\n bar",
                                 ~"foo \nbar",
                                 ~"  foo  bar  \nbaz",
                                 ~"foo bar baz",
                                 ~"foobarbaz\n\n"];

    let oracle_strs : ~[~str] = ~[~" foo bar",
                                 ~"foo bar ",
                                 ~"foo\n bar",
                                 ~"foo \nbar",
                                 ~" foo bar \nbaz",
                                 ~"foo bar baz",
                                 ~"foobarbaz\n\n"];

    assert!(test_strs.len() == oracle_strs.len());
    let mode = CompressWhitespace;

    for uint::range(0, test_strs.len()) |i| {
        (trimmed_str, _out) = transform_text(test_strs[i], mode, true);
        assert!(trimmed_str == oracle_strs[i])
    }
}

#[test]
fn test_transform_compress_whitespace_newline() {
    let  test_strs : ~[~str] = ~[~"  foo bar",
                                 ~"foo bar  ",
                                 ~"foo\n bar",
                                 ~"foo \nbar",
                                 ~"  foo  bar  \nbaz",
                                 ~"foo bar baz",
                                 ~"foobarbaz\n\n"];

    let oracle_strs : ~[~str] = ~[~"foo bar",
                                 ~"foo bar ",
                                 ~"foo bar",
                                 ~"foo bar",
                                 ~" foo bar baz",
                                 ~"foo bar baz",
                                 ~"foobarbaz "];

    assert!(test_strs.len() == oracle_strs.len());
    let mode = CompressWhitespaceNewline;

    for uint::range(0, test_strs.len()) |i| {
        (trimmed_str, _out) = transform_text(test_strs[i], mode, true);
        assert!(trimmed_str == oracle_strs[i])
    }
}

#[test]
fn test_transform_compress_whitespace_newline() {
    let  test_strs : ~[~str] = ~[~"  foo bar",
                                 ~"\nfoo bar",
                                 ~"foo bar  ",
                                 ~"foo\n bar",
                                 ~"foo \nbar",
                                 ~"  foo  bar  \nbaz",
                                 ~"foo bar baz",
                                 ~"foobarbaz\n\n"];

    let oracle_strs : ~[~str] = ~[~" foo bar",
                                 ~" foo bar",
                                 ~"foo bar ",
                                 ~"foo bar",
                                 ~"foo bar",
                                 ~" foo bar baz",
                                 ~"foo bar baz",
                                 ~"foobarbaz "];

    assert!(test_strs.len() == oracle_strs.len());
    let mode = CompressWhitespaceNewline;

    for uint::range(0, test_strs.len()) |i| {
        (trimmed_str, _out) = transform_text(test_strs[i], mode, false);
        assert!(trimmed_str == oracle_strs[i])
    }
}
