/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[test]
fn test_url_plugin() {
    assert_eq!("ftp://google.com/",
               url!("ftp://google.com").to_string());

    assert_eq!("ftp://google.com:443/",
               url!("ftp://google.com:443").to_string());

    assert_eq!("ftp://google.com:443/a/b/c",
               url!("ftp://google.com:443/a/b/c").to_string());

    assert_eq!("ftp://google.com:443/?a=b&c=d",
               url!("ftp://google.com:443?a=b&c=d").to_string());

    assert_eq!("http://[2001::1]/",
               url!("http://[2001::1]:80").to_string());

    assert_eq!("about:blank",
               url!("about:blank").to_string());
}
