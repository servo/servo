/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use encoding_rs::{UTF_8, UTF_16BE, UTF_16LE};
use script::test::encoding_detection::{
    get_xml_encoding, prescan_the_byte_stream_to_determine_the_encoding,
};

#[test]
fn html_encoding_with_xml_declaration() {
    assert_eq!(
        prescan_the_byte_stream_to_determine_the_encoding(&[0x3C, 0x0, 0x3F, 0x0, 0x78, 0x0, 0x42]),
        Some(UTF_16LE)
    );

    assert_eq!(
        prescan_the_byte_stream_to_determine_the_encoding(&[0x0, 0x3C, 0x0, 0x3F, 0x0, 0x78, 0x42]),
        Some(UTF_16BE)
    );
}

#[test]
fn meta_charset_within_comment() {
    assert_eq!(
        prescan_the_byte_stream_to_determine_the_encoding(b"<!-- <meta charset='utf8'> -->"),
        None
    );
}

#[test]
fn meta_charset_with_preceding_comment() {
    assert_eq!(
        prescan_the_byte_stream_to_determine_the_encoding(b"<!-- --> <meta charset='utf8'>"),
        Some(UTF_8)
    );

    assert_eq!(
        prescan_the_byte_stream_to_determine_the_encoding(b"<!--> <meta charset='utf8'>"),
        Some(UTF_8)
    );
}

#[test]
fn xml_encoding_invalid_start() {
    assert_eq!(get_xml_encoding(b"<?xmX encoding='UTF8'>"), None);
}

#[test]
fn xml_encoding_outside_of_declaration() {
    assert_eq!(get_xml_encoding(b"<?xml> encoding='UTF8'"), None);
}

#[test]
fn xml_encoding_missing_quotes() {
    // Missing opening quote
    assert_eq!(get_xml_encoding(b"<?xml encoding=UTF8'>"), None);

    // Missing closing quote
    assert_eq!(get_xml_encoding(b"<?xml encoding='UTF8>"), None);
}

#[test]
fn xml_encoding_containing_whitespace_within_quotes() {
    assert_eq!(get_xml_encoding(b"<?xml encoding=' UTF8'>"), None);
}

#[test]
fn xml_encoding_single_quotes() {
    assert_eq!(get_xml_encoding(b"<?xml encoding='UTF8'>"), Some(UTF_8));
}

#[test]
fn xml_encoding_double_quotes() {
    assert_eq!(get_xml_encoding(b"<?xml encoding=\"UTF8\">"), Some(UTF_8));
}

#[test]
fn xml_encoding_with_whitespace_around_equal_sign() {
    assert_eq!(
        get_xml_encoding(b"<?xml encoding \x00 =  \x00 \"UTF8\">"),
        Some(UTF_8)
    );
}
