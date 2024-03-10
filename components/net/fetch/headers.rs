/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::iter::Peekable;
use std::str::Chars;

use headers::HeaderMap;
use net_traits::fetch::headers::get_value_from_header_list;

/// <https://fetch.spec.whatwg.org/#http-tab-or-space>
const HTTP_TAB_OR_SPACE: &[char] = &['\u{0009}', '\u{0020}'];

/// <https://fetch.spec.whatwg.org/#determine-nosniff>
pub fn determine_nosniff(headers: &HeaderMap) -> bool {
    let values = get_header_value_as_list("x-content-type-options", headers);

    match values {
        None => false,
        Some(values) => !values.is_empty() && values[0].eq_ignore_ascii_case("nosniff"),
    }
}

/// <https://fetch.spec.whatwg.org/#concept-header-list-get-decode-split>
fn get_header_value_as_list(name: &str, headers: &HeaderMap) -> Option<Vec<String>> {
    fn char_is_not_quote_or_comma(c: char) -> bool {
        c != '\u{0022}' && c != '\u{002C}'
    }

    // Step 1
    let initial_value = get_value_from_header_list(name, headers);

    if let Some(input) = initial_value {
        // https://fetch.spec.whatwg.org/#header-value-get-decode-and-split
        // Step 1
        let input = input.into_iter().map(char::from).collect::<String>();

        // Step 2
        let mut position = input.chars().peekable();

        // Step 3
        let mut values: Vec<String> = vec![];

        // Step 4
        let mut value = String::new();

        // Step 5
        while position.peek().is_some() {
            // Step 5.1
            value += &*collect_sequence(&mut position, char_is_not_quote_or_comma);

            // Step 5.2
            if let Some(&ch) = position.peek() {
                if ch == '\u{0022}' {
                    // Step 5.2.1.1
                    value += &*collect_http_quoted_string(&mut position, false);

                    // Step 5.2.1.2
                    if position.peek().is_some() {
                        continue;
                    }
                } else {
                    // ch == '\u{002C}'

                    // Step 5.2.2.2
                    position.next();
                }
            }

            // Step 5.3
            value = value.trim_matches(HTTP_TAB_OR_SPACE).to_string();

            // Step 5.4
            values.push(value);

            // Step 5.5
            value = String::new();
        }

        return Some(values);
    }

    // Step 2
    None
}

/// <https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points>
fn collect_sequence<F>(position: &mut Peekable<Chars>, condition: F) -> String
where
    F: Fn(char) -> bool,
{
    // Step 1
    let mut result = String::new();

    // Step 2
    while let Some(&ch) = position.peek() {
        if !condition(ch) {
            break;
        }
        result.push(ch);
        position.next();
    }

    // Step 3
    result
}

/// <https://fetch.spec.whatwg.org/#collect-an-http-quoted-string>
fn collect_http_quoted_string(position: &mut Peekable<Chars>, extract_value: bool) -> String {
    fn char_is_not_quote_or_backslash(c: char) -> bool {
        c != '\u{0022}' && c != '\u{005C}'
    }

    // Step 2
    // We will store the 'extracted value' or the raw value
    let mut value = String::new();

    // Step 3, 4
    let should_be_quote = position.next();
    if let Some(ch) = should_be_quote {
        if !extract_value {
            value.push(ch)
        }
    }

    // Step 5
    loop {
        // Step 5.1
        value += &*collect_sequence(position, char_is_not_quote_or_backslash);

        // Step 5.2
        if position.peek().is_none() {
            break;
        }

        // Step 5.3, 5.4
        let quote_or_backslash = position.next().unwrap();
        if !extract_value {
            value.push(quote_or_backslash);
        }

        if quote_or_backslash == '\u{005C}' {
            if let Some(ch) = position.next() {
                value.push(ch);
            } else {
                // Step 5.5.1
                if extract_value {
                    value.push('\u{005C}');
                }
                break;
            }
        } else {
            // Step 5.6.1
            // assert quote_or_backslash is a quote

            // Step 5.6.2
            break;
        }
    }

    // Step 6, 7
    value
}
