/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Trims ASCII whitespace characters from a slice, and returns the trimmed input.
pub fn trim_ascii_whitespace(input: &str) -> &str {
    if input.is_empty() {
        return input;
    }

    let mut start = 0;
    {
        let mut iter = input.as_bytes().iter();
        loop {
            let byte = match iter.next() {
                Some(b) => b,
                None => return "",
            };

            if !byte.is_ascii_whitespace() {
                break;
            }
            start += 1;
        }
    }

    let mut end = input.len();
    assert!(start < end);
    {
        let mut iter = input.as_bytes()[start..].iter().rev();
        loop {
            let byte = match iter.next() {
                Some(b) => b,
                None => {
                    debug_assert!(false, "We should have caught this in the loop above!");
                    return "";
                },
            };

            if !byte.is_ascii_whitespace() {
                break;
            }
            end -= 1;
        }
    }

    &input[start..end]
}

#[test]
fn trim_ascii_whitespace_test() {
    fn test(i: &str, o: &str) {
        assert_eq!(trim_ascii_whitespace(i), o)
    }

    test("", "");
    test(" ", "");
    test(" a b c ", "a b c");
    test(" \t \t \ta b c \t \t \t \t", "a b c");
}
