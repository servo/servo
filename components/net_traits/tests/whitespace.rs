/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate net_traits;

#[test]
fn test_trim_http_whitespace() {
    fn test_trim(in_: &[u8], out: &[u8]) {
        let b = net_traits::trim_http_whitespace(in_);
        assert_eq!(b, out);
    }

    test_trim(b"", b"");

    test_trim(b" ", b"");
    test_trim(b"a", b"a");
    test_trim(b" a", b"a");
    test_trim(b"a ", b"a");
    test_trim(b" a ", b"a");

    test_trim(b"\t", b"");
    test_trim(b"a", b"a");
    test_trim(b"\ta", b"a");
    test_trim(b"a\t", b"a");
    test_trim(b"\ta\t", b"a");
}
