/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;

use net_traits::request::Origin;
use servo_url::{ImmutableOrigin, ServoUrl};
use url::Host;

#[test]
fn is_opaque() {
    let a = Origin::Origin(ImmutableOrigin::new_opaque());
    assert!(a.is_opaque());
}

#[test]
fn is_potentially_trustworthy_opaque() {
    let a = Origin::Origin(ImmutableOrigin::new_opaque());
    assert_eq!(a.is_potentially_trustworthy(), false);
}

#[test]
fn is_potentially_trustworthy_scheme_https() {
    let url = ServoUrl::parse("https://example.com/a.html").unwrap();
    let host = url.host().unwrap();

    assert!(url.is_secure_scheme());

    let a = Origin::Origin(ImmutableOrigin::Tuple(
        url.scheme().to_string(),
        host.to_owned(),
        url.port().unwrap_or(443),
    ));
    assert!(a.is_potentially_trustworthy());
}

#[test]
fn is_potentially_trustworthy_scheme_file() {
    let path = Path::new("../../resources/servo.css")
        .canonicalize()
        .unwrap();
    let url = ServoUrl::from_file_path(path.clone()).unwrap();

    assert_eq!(url.scheme(), "file");

    let a = Origin::Origin(ImmutableOrigin::Tuple(
        url.scheme().to_string(),
        Host::Domain(String::from("")),
        url.port().unwrap_or(80),
    ));
    assert!(a.is_potentially_trustworthy());
}

#[test]
fn is_potentially_trustworthy_host_local_ipv4() {
    let url = ServoUrl::parse("http://127.0.0.1/a.html").unwrap();
    let host = url.host().unwrap();

    assert_eq!(url.is_secure_scheme(), false);

    let a = Origin::Origin(ImmutableOrigin::Tuple(
        url.scheme().to_string(),
        host.to_owned(),
        url.port().unwrap_or(80),
    ));
    assert!(a.is_potentially_trustworthy());
}

#[test]
fn is_potentially_trustworthy_host_non_local_ipv4() {
    let url = ServoUrl::parse("http://168.45.2.16/a.html").unwrap();
    let host = url.host().unwrap();

    assert_eq!(url.is_secure_scheme(), false);

    let a = Origin::Origin(ImmutableOrigin::Tuple(
        url.scheme().to_string(),
        host.to_owned(),
        url.port().unwrap_or(80),
    ));
    assert_eq!(a.is_potentially_trustworthy(), false);
}

#[test]
fn is_potentially_trustworthy_host_local_ipv6() {
    let url = ServoUrl::parse("http://[::1]/a.html").unwrap();
    let host = url.host().unwrap();

    assert_eq!(url.is_secure_scheme(), false);

    let a = Origin::Origin(ImmutableOrigin::Tuple(
        url.scheme().to_string(),
        host.to_owned(),
        url.port().unwrap_or(80),
    ));
    assert!(a.is_potentially_trustworthy());
}

#[test]
fn is_potentially_trustworthy_host_non_local_ipv6() {
    let url = ServoUrl::parse("http://[e9f2:5144:c8b9:8c8d:2106:0072:8dc7:0f6b]/a.html").unwrap();
    let host = url.host().unwrap();

    assert_eq!(url.is_secure_scheme(), false);

    let a = Origin::Origin(ImmutableOrigin::Tuple(
        url.scheme().to_string(),
        host.to_owned(),
        url.port().unwrap_or(80),
    ));
    assert_eq!(a.is_potentially_trustworthy(), false);
}

#[test]
fn is_potentially_trustworthy_host_localhost_tld() {
    let url = ServoUrl::parse("http://localhost/a.html").unwrap();
    let host = url.host().unwrap();

    assert_eq!(url.is_secure_scheme(), false);

    let a = Origin::Origin(ImmutableOrigin::Tuple(
        url.scheme().to_string(),
        host.to_owned(),
        url.port().unwrap_or(80),
    ));
    assert!(a.is_potentially_trustworthy());
}

fn is_potentially_trustworthy_host_localhost_subdomain() {
    let url = ServoUrl::parse("http://localhost.example.com/b.css").unwrap();
    let host = url.host().unwrap();

    assert_eq!(url.is_secure_scheme(), false);

    let a = Origin::Origin(ImmutableOrigin::Tuple(
        url.scheme().to_string(),
        host.to_owned(),
        url.port().unwrap_or(80),
    ));
    assert_eq!(a.is_potentially_trustworthy(), false);
}
