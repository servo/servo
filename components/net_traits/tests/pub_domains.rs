/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate net_traits;

use net_traits::pub_domains::{is_pub_domain, is_reg_domain, pub_suffix, reg_suffix};

// These tests may need to be updated if the PSL changes.

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
    assert_eq!(is_pub_domain("www.ck"), false);
    assert_eq!(is_pub_domain("city.kawasaki.jp"), false);
    assert_eq!(is_pub_domain("city.nagoya.jp"), false);
    assert_eq!(is_pub_domain("teledata.mz"), false);
}

#[test]
fn test_is_pub_domain_not() {
    assert_eq!(is_pub_domain(""), false);
    assert_eq!(is_pub_domain("."), false);
    assert_eq!(is_pub_domain("..."), false);
    assert_eq!(is_pub_domain(".servo.org"), false);
    assert_eq!(is_pub_domain("www.mozilla.org"), false);
    assert_eq!(is_pub_domain("publicsuffix.org"), false);
    assert_eq!(is_pub_domain("hello.world.jm"), false);
    assert_eq!(is_pub_domain("toto.toto.kobe.jp"), false);
}

#[test]
fn test_is_pub_domain() {
    assert!(!is_pub_domain("city.yokohama.jp"));
    assert!(!is_pub_domain("foo.bar.baz.yokohama.jp"));
    assert!(!is_pub_domain("foo.bar.city.yokohama.jp"));
    assert!(!is_pub_domain("foo.bar.com"));
    assert!(!is_pub_domain("foo.bar.tokyo.jp"));
    assert!(!is_pub_domain("foo.bar.yokohama.jp"));
    assert!(!is_pub_domain("foo.city.yokohama.jp"));
    assert!(!is_pub_domain("foo.com"));
    assert!(!is_pub_domain("foo.tokyo.jp"));
    assert!(!is_pub_domain("yokohama.jp"));
    assert!(is_pub_domain("com"));
    assert!(is_pub_domain("foo.yokohama.jp"));
    assert!(is_pub_domain("jp"));
    assert!(is_pub_domain("tokyo.jp"));
}

#[test]
fn test_is_reg_domain() {
    assert!(!is_reg_domain("com"));
    assert!(!is_reg_domain("foo.bar.baz.yokohama.jp"));
    assert!(!is_reg_domain("foo.bar.com"));
    assert!(!is_reg_domain("foo.bar.tokyo.jp"));
    assert!(!is_reg_domain("foo.city.yokohama.jp"));
    assert!(!is_reg_domain("foo.yokohama.jp"));
    assert!(!is_reg_domain("jp"));
    assert!(!is_reg_domain("tokyo.jp"));
    assert!(is_reg_domain("city.yokohama.jp"));
    assert!(is_reg_domain("foo.bar.yokohama.jp"));
    assert!(is_reg_domain("foo.com"));
    assert!(is_reg_domain("foo.tokyo.jp"));
    assert!(is_reg_domain("yokohama.jp"));
}

#[test]
fn test_pub_suffix() {
    assert_eq!(pub_suffix("city.yokohama.jp"), "yokohama.jp");
    assert_eq!(pub_suffix("com"), "com");
    assert_eq!(pub_suffix("foo.bar.baz.yokohama.jp"), "baz.yokohama.jp");
    assert_eq!(pub_suffix("foo.bar.com"), "com");
    assert_eq!(pub_suffix("foo.bar.tokyo.jp"), "tokyo.jp");
    assert_eq!(pub_suffix("foo.bar.yokohama.jp"), "bar.yokohama.jp");
    assert_eq!(pub_suffix("foo.city.yokohama.jp"), "yokohama.jp");
    assert_eq!(pub_suffix("foo.com"), "com");
    assert_eq!(pub_suffix("foo.tokyo.jp"), "tokyo.jp");
    assert_eq!(pub_suffix("foo.yokohama.jp"), "foo.yokohama.jp");
    assert_eq!(pub_suffix("jp"), "jp");
    assert_eq!(pub_suffix("tokyo.jp"), "tokyo.jp");
    assert_eq!(pub_suffix("yokohama.jp"), "jp");
}

#[test]
fn test_reg_suffix() {
    assert_eq!(reg_suffix("city.yokohama.jp"), "city.yokohama.jp");
    assert_eq!(reg_suffix("com"), "com");
    assert_eq!(reg_suffix("foo.bar.baz.yokohama.jp"), "bar.baz.yokohama.jp");
    assert_eq!(reg_suffix("foo.bar.com"), "bar.com");
    assert_eq!(reg_suffix("foo.bar.tokyo.jp"), "bar.tokyo.jp");
    assert_eq!(reg_suffix("foo.bar.yokohama.jp"), "foo.bar.yokohama.jp");
    assert_eq!(reg_suffix("foo.city.yokohama.jp"), "city.yokohama.jp");
    assert_eq!(reg_suffix("foo.com"), "foo.com");
    assert_eq!(reg_suffix("foo.tokyo.jp"), "foo.tokyo.jp");
    assert_eq!(reg_suffix("foo.yokohama.jp"), "foo.yokohama.jp");
    assert_eq!(reg_suffix("jp"), "jp");
    assert_eq!(reg_suffix("tokyo.jp"), "tokyo.jp");
    assert_eq!(reg_suffix("yokohama.jp"), "yokohama.jp");
}

#[test]
fn test_weirdness() {
    // These are weird results, but AFAICT they are spec-compliant.
    assert!(pub_suffix("city.yokohama.jp") != pub_suffix(pub_suffix("city.yokohama.jp")));
    assert!(!is_pub_domain(pub_suffix("city.yokohama.jp")));
}
