/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::iter::Peekable;
use std::str::{Chars, FromStr};

use data_url::mime::Mime as DataUrlMime;
use headers::HeaderMap;

/// <https://fetch.spec.whatwg.org/#http-tab-or-space>
const HTTP_TAB_OR_SPACE: &[char] = &['\u{0009}', '\u{0020}'];

/// <https://fetch.spec.whatwg.org/#concept-header-list-get>
pub fn get_value_from_header_list(name: &str, headers: &HeaderMap) -> Option<Vec<u8>> {
    let values = headers.get_all(name).iter().map(|val| val.as_bytes());

    // Step 1: If list does not contain name, then return null.
    if values.size_hint() == (0, Some(0)) {
        return None;
    }

    // Step 2: Return the values of all headers in list whose name is a byte-case-insensitive match
    // for name, separated from each other by 0x2C 0x20, in order.
    Some(values.collect::<Vec<&[u8]>>().join(&[0x2C, 0x20][..]))
}

/// <https://fetch.spec.whatwg.org/#forbidden-method>
pub fn is_forbidden_method(method: &[u8]) -> bool {
    matches!(
        method.to_ascii_lowercase().as_slice(),
        b"connect" | b"trace" | b"track"
    )
}

/// <https://fetch.spec.whatwg.org/#concept-header-list-get-decode-split>
pub fn get_decode_and_split_header_name(name: &str, headers: &HeaderMap) -> Option<Vec<String>> {
    // Step 1: Let value be the result of getting name from list.
    // Step 2: If value is null, then return null.
    // Step 3: Return the result of getting, decoding, and splitting value.
    get_value_from_header_list(name, headers).map(get_decode_and_split_header_value)
}

/// <https://fetch.spec.whatwg.org/#header-value-get-decode-and-split>
pub fn get_decode_and_split_header_value(value: Vec<u8>) -> Vec<String> {
    fn char_is_not_quote_or_comma(c: char) -> bool {
        c != '\u{0022}' && c != '\u{002C}'
    }

    // Step 1: Let input be the result of isomorphic decoding value.
    let input = value.into_iter().map(char::from).collect::<String>();

    // Step 2: Let position be a position variable for input, initially pointing at the start of
    // input.
    let mut position = input.chars().peekable();

    // Step 3: Let values be a list of strings, initially « ».
    let mut values: Vec<String> = vec![];

    // Step 4: Let temporaryValue be the empty string.
    let mut temporary_value = String::new();

    // Step 5: While true:
    while position.peek().is_some() {
        // Step 5.1: Append the result of collecting a sequence of code points that are not U+0022
        // (") or U+002C (,) from input, given position, to temporaryValue.
        temporary_value += &*collect_sequence(&mut position, char_is_not_quote_or_comma);

        // Step 5.2: If position is not past the end of input and the code point at position within
        // input is U+0022 ("):
        if let Some(&ch) = position.peek() {
            if ch == '\u{0022}' {
                // Step 5.2.1: Append the result of collecting an HTTP quoted string from input,
                // given position, to temporaryValue.
                temporary_value += &*collect_http_quoted_string(&mut position, false);

                // Step 5.2.2: If position is not past the end of input, then continue.
                if position.peek().is_some() {
                    continue;
                }
            } else {
                // Step 5.2.2: If position is not past the end of input, then continue.
                position.next();
            }
        }

        // Step 5.3: Remove all HTTP tab or space from the start and end of temporaryValue.
        temporary_value = temporary_value.trim_matches(HTTP_TAB_OR_SPACE).to_string();

        // Step 5.4: Append temporaryValue to values.
        values.push(temporary_value);

        // Step 5.5: Set temporaryValue to the empty string.
        temporary_value = String::new();
    }

    values
}

/// <https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points>
fn collect_sequence<F>(position: &mut Peekable<Chars>, condition: F) -> String
where
    F: Fn(char) -> bool,
{
    // Step 1: Let result be the empty string.
    let mut result = String::new();

    // Step 2: While position doesn’t point past the end of input and the code point at position
    // within input meets the condition condition:
    while let Some(&ch) = position.peek() {
        if !condition(ch) {
            break;
        }

        // Step 2.1: Append that code point to the end of result.
        result.push(ch);

        // Step 2.2: Advance position by 1.
        position.next();
    }

    // Step 3: Return result.
    result
}

/// <https://fetch.spec.whatwg.org/#collect-an-http-quoted-string>
fn collect_http_quoted_string(position: &mut Peekable<Chars>, extract_value: bool) -> String {
    fn char_is_not_quote_or_backslash(c: char) -> bool {
        c != '\u{0022}' && c != '\u{005C}'
    }

    // Step 2: let value be the empty string
    // We will store the 'extracted value' or the raw value
    let mut value = String::new();

    // Step 3, 4
    let should_be_quote = position.next();
    if let Some(ch) = should_be_quote {
        if !extract_value {
            value.push(ch)
        }
    }

    // Step 5: While true:
    loop {
        // Step 5.1: Append the result of collecting a sequence of code points that are not U+0022
        // (") or U+005C (\) from input, given position, to value.
        value += &*collect_sequence(position, char_is_not_quote_or_backslash);

        // Step 5.2: If position is past the end of input, then break.
        if position.peek().is_none() {
            break;
        }

        // Step 5.3: Let quoteOrBackslash be the code point at position within input.
        // Step 5.4: Advance position by 1.
        let quote_or_backslash = position.next().unwrap();

        if !extract_value {
            value.push(quote_or_backslash);
        }

        // Step 5.5: If quoteOrBackslash is U+005C (\), then:
        if quote_or_backslash == '\u{005C}' {
            if let Some(ch) = position.next() {
                // Step 5.5.2: Append the code point at position within input to value.
                value.push(ch);
            } else {
                // Step 5.5.1: If position is past the end of input, then append U+005C (\) to value and break.
                if extract_value {
                    value.push('\u{005C}');
                }

                break;
            }
        } else {
            // Step 5.6.1: Assert quote_or_backslash is a quote
            assert_eq!(quote_or_backslash, '\u{0022}');

            // Step 5.6.2: break
            break;
        }
    }

    // Step 6, 7
    value
}

/// <https://fetch.spec.whatwg.org/#concept-header-extract-mime-type>
/// This function uses data_url::Mime to parse the MIME Type because
/// mime::Mime does not provide a parser following the Fetch spec
/// see <https://github.com/hyperium/mime/issues/106>
pub fn extract_mime_type_as_dataurl_mime(headers: &HeaderMap) -> Option<DataUrlMime> {
    // > 1: Let charset be null.
    let mut charset = None;
    // > 2: Let essence be null.
    let mut essence = String::new();
    // > 3: Let mimeType be null.
    let mut mime_type = None;

    // > 4: Let values be the result of getting, decoding, and splitting `Content-Type`
    // from headers.
    // > 5: If values is null, then return failure.
    let headers_values = get_decode_and_split_header_name("content-type", headers)?;

    // > 6: For each value of values:
    for header_value in headers_values.iter() {
        // > 6.1: Let temporaryMimeType be the result of parsing value.
        match DataUrlMime::from_str(header_value) {
            // > 6.2: If temporaryMimeType is failure or its essence is "*/*", then continue.
            Err(_) => continue,
            Ok(temp_mime) => {
                let temp_essence = format!("{}/{}", temp_mime.type_, temp_mime.subtype);

                // > 6.2: If temporaryMimeType is failure or its essence is "*/*", then
                // continue.
                if temp_essence == "*/*" {
                    continue;
                }

                // > 6.3: Set mimeType to temporaryMimeType.
                mime_type = Some(DataUrlMime {
                    type_: temp_mime.type_.to_string(),
                    subtype: temp_mime.subtype.to_string(),
                    parameters: temp_mime.parameters.clone(),
                });

                // > 6.4: If mimeType’s essence is not essence, then:
                let temp_charset = &temp_mime.get_parameter("charset");
                if temp_essence != essence {
                    // > 6.4.1: Set charset to null.
                    // > 6.4.2: If mimeType’s parameters["charset"] exists, then set
                    //   charset to mimeType’s parameters["charset"].
                    charset = temp_charset.map(|c| c.to_string());
                    // > 6.4.3: Set essence to mimeType’s essence.
                    essence = temp_essence.to_owned();
                } else {
                    // > 6.5: Otherwise, if mimeType’s parameters["charset"] does not exist,
                    //   and charset is non-null, set mimeType’s parameters["charset"] to charset.
                    if temp_charset.is_none() && charset.is_some() {
                        let DataUrlMime {
                            type_: t,
                            subtype: st,
                            parameters: p,
                        } = mime_type.unwrap();
                        let mut params = p;
                        params.push(("charset".to_string(), charset.clone().unwrap()));
                        mime_type = Some(DataUrlMime {
                            type_: t.to_string(),
                            subtype: st.to_string(),
                            parameters: params,
                        })
                    }
                }
            },
        }
    }

    // > 7: If mimeType is null, then return failure.
    // > 8: Return mimeType.
    mime_type
}

pub fn extract_mime_type(headers: &HeaderMap) -> Option<Vec<u8>> {
    extract_mime_type_as_dataurl_mime(headers).map(|m| format!("{}", m).into_bytes())
}

pub fn extract_mime_type_as_mime(headers: &HeaderMap) -> Option<mime::Mime> {
    extract_mime_type_as_dataurl_mime(headers).and_then(|mime: DataUrlMime| {
        // Try to transform a data-url::mime::Mime into a mime::Mime
        let mut mime_as_str = format!("{}/{}", mime.type_, mime.subtype);
        for p in mime.parameters {
            mime_as_str.push_str(format!("; {}={}", p.0, p.1).as_str());
        }
        mime_as_str.parse().ok()
    })
}
