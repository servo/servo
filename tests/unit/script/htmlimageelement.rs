/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::test::DOMString;
use script::test::sizes::{parse_a_sizes_attribute, Size, test_media_query, test_length};

#[test]
fn no_default_provided() {
    assert!(parse_a_sizes_attribute(DOMString::new(), None).len() == 1);
    println!("{:?}", parse_a_sizes_attribute(DOMString::new(), None));
}

#[test]
fn default_provided() {
    assert!(parse_a_sizes_attribute(DOMString::new(), Some(2)).len() == 1);
}

#[test]
fn no_size() {
    assert!(parse_a_sizes_attribute(DOMString::new(), None).len() == 1);
}

#[test]
fn one_value() {
    let mut a = vec![];
    let media_query = test_media_query(200f32);
    let length = test_length(545f32);
    let size = Size { query: Some(media_query), length: length };
    a.push(size);
    assert_eq!(parse_a_sizes_attribute(DOMString::from("(max-width: 200px) 545px"), None), a);
}

#[test]
fn more_then_one_value() {
    let result = parse_a_sizes_attribute(DOMString::from("(min-width: 900px) 1000px,
            (max-width: 900px) and (min-width: 400px) 50em,
            100vw           "),
            None);
    assert_eq!(result.len(), 3);
}
