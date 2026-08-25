/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[test]
fn test_is_cors_safelisted_request_range() {
    use net_traits::request::is_cors_safelisted_request_range;

    assert!(is_cors_safelisted_request_range(b"bytes=100-200"));
    assert!(is_cors_safelisted_request_range(b"bytes=200-"));
    assert!(!is_cors_safelisted_request_range(b"bytes=abc-def"));
    assert!(!is_cors_safelisted_request_range(b""));
}
