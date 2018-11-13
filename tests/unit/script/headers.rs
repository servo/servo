/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::test::{ByteString, normalize_value};

#[test]
fn test_normalize_empty_bytestring() {
    // empty ByteString test
    let empty_bytestring = ByteString::new(vec![]);
    let actual = normalize_value(empty_bytestring);
    let expected = ByteString::new(vec![]);
    assert_eq!(actual, expected);
}

#[test]
fn test_normalize_all_whitespace_bytestring() {
    // All whitespace test. A horizontal tab, a line feed, a carriage return , and a space
    let all_whitespace_bytestring = ByteString::new(vec![b'\t', b'\n', b'\r', b' ']);
    let actual = normalize_value(all_whitespace_bytestring);
    let expected = ByteString::new(vec![]);
    assert_eq!(actual, expected);
}

#[test]
fn test_normalize_non_empty_no_whitespace_bytestring() {
    // Non-empty, no whitespace ByteString test
    let no_whitespace_bytestring = ByteString::new(vec![b'S', b'!']);
    let actual = normalize_value(no_whitespace_bytestring);
    let expected = ByteString::new(vec![b'S', b'!']);
    assert_eq!(actual, expected);
}

#[test]
fn test_normalize_non_empty_leading_whitespace_bytestring() {
    // Non-empty, leading whitespace, no trailing whitespace ByteString test
    let leading_whitespace_bytestring = ByteString::new(vec![b'\t', b'\n', b' ', b'\r', b'S', b'!']);
    let actual = normalize_value(leading_whitespace_bytestring);
    let expected = ByteString::new(vec![b'S', b'!']);
    assert_eq!(actual, expected);
}

#[test]
fn test_normalize_non_empty_no_leading_whitespace_trailing_whitespace_bytestring() {
    // Non-empty, no leading whitespace, but with trailing whitespace ByteString test
    let trailing_whitespace_bytestring = ByteString::new(vec![b'S', b'!', b'\t', b'\n', b' ', b'\r']);
    let actual = normalize_value(trailing_whitespace_bytestring);
    let expected = ByteString::new(vec![b'S', b'!']);
    assert_eq!(actual, expected);
}

#[test]
fn test_normalize_non_empty_leading_and_trailing_whitespace_bytestring() {
    // Non-empty, leading whitespace, and trailing whitespace ByteString test
    let whitespace_sandwich_bytestring =
        ByteString::new(vec![b'\t', b'\n', b' ', b'\r', b'S', b'!', b'\t', b'\n', b' ', b'\r']);
    let actual = normalize_value(whitespace_sandwich_bytestring);
    let expected = ByteString::new(vec![b'S', b'!']);
    assert_eq!(actual, expected);
}

#[test]
fn test_normalize_non_empty_leading_trailing_and_internal_whitespace_bytestring() {
    // Non-empty, leading whitespace, trailing whitespace,
    // and internal whitespace ByteString test
    let whitespace_bigmac_bytestring =
        ByteString::new(vec![b'\t', b'\n', b' ', b'\r', b'S',
                             b'\t', b'\n', b' ', b'\r', b'!',
                             b'\t', b'\n', b' ', b'\r']);
    let actual = normalize_value(whitespace_bigmac_bytestring);
    let expected = ByteString::new(vec![b'S', b'\t', b'\n', b' ', b'\r', b'!']);
    assert_eq!(actual, expected);
}
