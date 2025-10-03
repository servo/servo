/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str::FromStr;

use servo_url::ServoUrl;
use url::Url;

#[test]
fn test_matches_about_blank_matches_simple_about_blank() {
    let mut url = ServoUrl::from_str("about:blank").unwrap();
    assert!(url.path().contains("blank"));
    assert!(url.scheme() == "about");
    assert!(url.matches_about_blank());

    // Cannot set password.
    assert!(url.set_password(Some("Test")).is_err());
    assert!(url.matches_about_blank());

    // Cannot set username.
    assert!(url.set_username("user1").is_err());
    assert!(url.matches_about_blank());

    // Note: cannot set host, `set_host` not available on `ServoUrl`.
    assert!(url.host().is_none());
}

#[test]
fn test_matches_about_blank_does_not_match() {
    let mut url = ServoUrl::from_str("ftp://user1:secret1@example.com").unwrap();
    assert!(!url.matches_about_blank());

    url.set_password(Some("Test")).unwrap();
    assert!(!url.matches_about_blank());

    url.set_username("user1").unwrap();
    assert!(!url.matches_about_blank());
}

#[test]
fn test_matches_about_blank_does_not_match_from_other_url() {
    let mut url = Url::parse("https://example.com").unwrap();

    // Cannot construct something passing off like about:blank from url.
    assert!(url.set_scheme("about").is_err());
    assert!(url.set_host(None).is_err());
    assert!(url.set_password(Some("Test")).is_ok());
    url.set_path("blank");

    let servo_url = ServoUrl::from_url(url);
    assert!(!servo_url.matches_about_blank());
}

#[test]
fn test_matches_about_blank_does_not_match_invariants_maintained_from_url() {
    // Invariants of about:blank maintained at the url level as well.
    let mut url = Url::parse("about:blank").unwrap();

    // Cannot set password.
    assert!(url.set_password(Some("Test")).is_err());

    // Cannot set username.
    assert!(url.set_username("user1").is_err());

    // Cannot set host.
    assert!(url.set_host(Some("rust-lang.org")).is_err());

    let servo_url = ServoUrl::from_url(url.clone());
    assert!(servo_url.matches_about_blank());

    // Can set path, but match will fail.
    url.set_path("test");
    let servo_url = ServoUrl::from_url(url);
    assert!(!servo_url.matches_about_blank());
}
