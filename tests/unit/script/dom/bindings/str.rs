/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::test::DOMString;

#[test]
fn test_unicode_string_split_on_ascii_whitespace() {
    let test_cases = vec![
        ("", ""),
        ("  \t黑aaab", "黑aaab"),
        ("三体!\n\n", "三体!"),
        ("#β", "#β"),
        ("  #β\n ", "#β"),
    ];
    for (s, expected) in test_cases {
        let mut dom_str = DOMString::from(s);
        dom_str.strip_leading_and_trailing_ascii_whitespace();
        assert_eq!(dom_str.as_ref(), expected)
    }
}
