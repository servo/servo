/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<<<<<<< e00fdb92cce81f71aeb51d963f7de1e74a12d180
use script::dom::bindings::str::{ByteString};
use script::dom::headers; // correct syntax?
=======
use script::dom::bindings::str::ByteString;
use script::dom::headers;
>>>>>>> Add the append method for the Headers API

#[test]
fn test_normalize_empty_bytestring() {
    // empty ByteString test
    let empty_bytestring = ByteString::new(vec![]);
    let actual_normalized_empty_bytestring = headers::normalize(empty_bytestring);
    let expected_normalized_empty_bytestring = ByteString::new(vec![]);
    assert_eq!(actual_normalized_empty_bytestring, expected_normalized_empty_bytestring);
}

#[test]
fn test_normalize_all_whitespace_bytestring() {
    // All whitespace test. A horizontal tab, a line feed, a carriage return , and a space
    let all_whitespace_bytestring = ByteString::new(vec![0x09, 0x0A, '\n' as u8, 0x20]);
    let actual_normalized_whitespace_bytestring = headers::normalize(all_whitespace_bytestring);
    let expected_normalized_whitespace_bytestring = ByteString::new(vec![]);
    assert_eq!(actual_normalized_whitespace_bytestring, expected_normalized_whitespace_bytestring);
}

#[test]
fn test_normalize_non_empty_no_whitespace_bytestring() {
    // Non-empty, no whitespace ByteString test
    let no_whitespace_bytestring = ByteString::new(vec!['S' as u8, '!' as u8]);
<<<<<<< e00fdb92cce81f71aeb51d963f7de1e74a12d180
    let actual_normalized_no_whitespace_bytestring = headers::normalize(no_whitespace_bytestring);
=======
    let actual_normalgized_no_whitespace_bytestring = headers::normalize(no_whitespace_bytestring);
>>>>>>> Add the append method for the Headers API
    let expected_normalized_no_whitespace_bytestring = ByteString::new(vec!['S' as u8, '!' as u8]);
    assert_eq!(actual_normalized_no_whitespace_bytestring, expected_normalized_no_whitespace_bytestring);
}

#[test]
fn test_normalize_non_empty_leading_whitespace_bytestring() {
    // Non-empty, leading whitespace, no trailing whitespace ByteString test
    let leading_whitespace_bytestring = ByteString::new(vec![0x09, 0x0A, '\n' as u8, 0x20, 'S' as u8, '!' as u8]);
    let actual_normalized_leading_whitespace_bytestring = headers::normalize(leading_whitespace_bytestring);
    let expected_normalized_leading_whitespace_bytestring = ByteString::new(vec!['S' as u8, '!' as u8]);
    assert_eq!(actual_normalized_leading_whitespace_bytestring, expected_normalized_leading_whitespace_bytestring);
}

#[test]
fn test_normalize_non_empty_no_leading_whitespace_trailing_whitespace_bytestring() {
    // Non-empty, no leading whitespace, but with trailing whitespace ByteString test
    let trailing_whitespace_bytestring = ByteString::new(vec!['S' as u8, '!' as u8, 0x09, 0x0A, '\n' as u8, 0x20]);
    let actual_normalized_trailing_whitespace_bytestring = headers::normalize(trailing_whitespace_bytestring);
    let expected_normalized_trailing_whitespace_bytestring = ByteString::new(vec!['S' as u8, '!' as u8]);
    assert_eq!(actual_normalized_trailing_whitespace_bytestring, expected_normalized_trailing_whitespace_bytestring);
}

#[test]
fn test_normalize_non_empty_leading_and_trailing_whitespace_bytestring() {
    // Non-empty, leading whitespace, and trailing whitespace ByteString test
<<<<<<< e00fdb92cce81f71aeb51d963f7de1e74a12d180
    let whitespace_sandwich_bytestring = ByteString::new(vec![0x09, 0x0A, '\n' as u8, 0x20, 'S' as u8, '!' as u8, 0x09, 0x0A, '\n' as u8, 0x20]);
=======
    let whitespace_sandwich_bytestring =
        ByteString::new(vec![0x09, 0x0A, '\n' as u8, 0x20, 'S' as u8, '!' as u8, 0x09, 0x0A, '\n' as u8, 0x20]);
>>>>>>> Add the append method for the Headers API
    let actual_normalized_whitespace_sandwich_bytestring = headers::normalize(whitespace_sandwich_bytestring);
    let expected_normalized_whitespace_sandwich_bytestring = ByteString::new(vec!['S' as u8, '!' as u8]);
    assert_eq!(actual_normalized_whitespace_sandwich_bytestring, expected_normalized_whitespace_sandwich_bytestring);
}

#[test]
fn test_normalize_non_empty_leading_trailing_and_internal_whitespace_bytestring() {
<<<<<<< e00fdb92cce81f71aeb51d963f7de1e74a12d180
    // Non-empty, leading whitespace, trailing whitespace, and internal whitespace ByteString test
    let whitespace_bigmac_bytestring = ByteString::new(vec![0x09, 0x0A, '\n' as u8, 0x20, 'S' as u8, 0x09, 0x0A, '\n' as u8, 0x20, '!' as u8, 0x09, 0x0A, '\n' as u8, 0x20]);
    let actual_normalized_whitespace_bigmac_bytestring = headers::normalize(whitespace_bigmac_bytestring);
    let expected_normalized_whitespace_bigmac_bytestring = ByteString::new(vec!['S' as u8, 0x09, 0x0A, '\n' as u8, 0x20, '!' as u8]);
=======
    // Non-empty, leading whitespace, trailing whitespace,
    // and internal whitespace ByteString test
    let whitespace_bigmac_bytestring =
        ByteString::new(vec![0x09, 0x0A, '\n' as u8, 0x20, 'S' as u8,
                             0x09, 0x0A, '\n' as u8, 0x20, '!' as u8,
                             0x09, 0x0A, '\n' as u8, 0x20]);
    let actual_normalized_whitespace_bigmac_bytestring = headers::normalize(whitespace_bigmac_bytestring);
    let expected_normalized_whitespace_bigmac_bytestring =
        ByteString::new(vec!['S' as u8, 0x09, 0x0A, '\n' as u8, 0x20, '!' as u8]);
>>>>>>> Add the append method for the Headers API
    assert_eq!(actual_normalized_whitespace_bigmac_bytestring, expected_normalized_whitespace_bigmac_bytestring);
}
