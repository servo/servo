enum CompressionMode {
    CompressNone,
    CompressWhitespace,
    CompressWhitespaceNewline,
    DiscardNewline
}

impl CompressionMode : cmp::Eq {
    pure fn eq(other: &CompressionMode) -> bool {
        match (self, *other) {
            (CompressNone, CompressNone) => true,
            (CompressWhitespace, CompressWhitespace) => true,
            (CompressWhitespaceNewline, CompressWhitespaceNewline) => true,
            (DiscardNewline, DiscardNewline) => true,
            _ => false
        }
    }
    pure fn ne(other: &CompressionMode) -> bool {
        !self.eq(other)
    }
}

// ported from Gecko's nsTextFrameUtils::TransformText. 
// 
// High level TODOs:
// * consider incoming text state (preceding spaces, arabic, etc)
// * send outgoing text state (dual of above)
// * record skipped and kept chars for mapping original to new text
// * various edge cases for bidi, CJK, combining char seqs, etc.
pub fn transform_text(text: &str, mode: CompressionMode) -> ~str {
    let out_str: ~str = ~"";
    match mode {
        CompressNone | DiscardNewline => {
            do str::each_char(text) |ch: char| {
                if is_discardable_char(ch, mode) {
                    // TODO: record skipped char
                } else {
                    // TODO: record kept char
                    if ch == '\t' {
                        // TODO: set "has tab" flag
                    }
                    str::push_char(&out_str, ch);
                }

                true
            }
        },

        CompressWhitespace | CompressWhitespaceNewline => {
            let mut in_whitespace: bool = false;
            do str::each_char(text) |ch: char| {
                // TODO: discard newlines between CJK chars
                let mut next_in_whitespace: bool = match (ch, mode) {
                    // TODO: check for following char that may create
                    // a Unicode combining-character sequence with a
                    // space, in which case it shouldn't be  compressed.
                    (' ', _)  => true,
                    ('\t', _) => true,
                    ('\n', CompressWhitespaceNewline) => true,
                    (_, _)    => false
                };
                
                if next_in_whitespace {
                    if is_always_discardable_char(ch) {
                        // revert whitespace setting, since this char was discarded
                        next_in_whitespace = in_whitespace;
                        // TODO: record skipped char
                    } else {
                        // TODO: record kept char
                        str::push_char(&out_str, ch);
                    }
                } else {
                    if in_whitespace {
                        // TODO: record skipped char
                    } else {
                        // TODO: record kept char
                        str::push_char(&out_str, ch);
                    }
                }
                // save whitespace context for next char
                in_whitespace = next_in_whitespace;
                true
            } /* /do str::each_chari */
        } 
    }

    return out_str;

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
    assert true_type_tag('c', 'm', 'a', 'p') == 0x_63_6D_61_70_u32;
}
