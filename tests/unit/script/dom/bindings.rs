/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::dom::bindings::str::ByteString;

#[test]
fn test_byte_string_move() {
    let mut byte_str = ByteString::new(vec![0x73, 0x65, 0x72, 0x76, 0x6f]);
    let mut byte_vec = byte_str.bytes();

    assert_eq!(byte_vec, "servo".as_bytes());
    assert_eq!(*byte_str, []);

    byte_vec = byte_str.into();
    assert_eq!(byte_vec, Vec::new());
}
