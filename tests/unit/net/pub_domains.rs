/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::pub_domains::is_pub_domain;

#[test]
fn test_is_pub_domain_plain() {
    assert!(is_pub_domain("com"));
    assert!(is_pub_domain(".org"));
    assert!(is_pub_domain("za.org"));
    assert!(is_pub_domain("xn--od0alg.hk"));
    assert!(is_pub_domain("xn--krdsherad-m8a.no"));
}

#[test]
fn test_is_pub_domain_wildcard() {
    assert!(is_pub_domain("hello.bd"));
    assert!(is_pub_domain("world.jm"));
    assert!(is_pub_domain("toto.kobe.jp"));
}

#[test]
fn test_is_pub_domain_exception() {
    assert!(is_pub_domain("www.ck") == false);
    assert!(is_pub_domain("city.kawasaki.jp") == false);
    assert!(is_pub_domain("city.nagoya.jp") == false);
    assert!(is_pub_domain("teledata.mz") == false);
}

#[test]
fn test_is_pub_domain_not() {
    assert!(is_pub_domain(".servo.org") == false);
    assert!(is_pub_domain("www.mozilla.org") == false);
    assert!(is_pub_domain("publicsuffix.org") == false);
    assert!(is_pub_domain("hello.world.jm") == false);
    assert!(is_pub_domain("toto.toto.kobe.jp") == false);
}
