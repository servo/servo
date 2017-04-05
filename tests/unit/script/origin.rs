/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};

#[test]
fn same_origin() {
    let a = MutableOrigin::new(ServoUrl::parse("http://example.com/a.html").unwrap().origin());
    let b = MutableOrigin::new(ServoUrl::parse("http://example.com/b.html").unwrap().origin());
    assert!(a.same_origin(&b));
    assert_eq!(a.is_tuple(), true);
}

#[test]
fn identical_origin() {
    let a = MutableOrigin::new(ServoUrl::parse("http://example.com/a.html").unwrap().origin());
    assert!(a.same_origin(&a));
}

#[test]
fn cross_origin() {
    let a = MutableOrigin::new(ServoUrl::parse("http://example.com/a.html").unwrap().origin());
    let b = MutableOrigin::new(ServoUrl::parse("http://example.org/b.html").unwrap().origin());
    assert!(!a.same_origin(&b));
}

#[test]
fn clone_same_origin() {
    let a = MutableOrigin::new(ServoUrl::parse("http://example.com/a.html").unwrap().origin());
    let b = MutableOrigin::new(ServoUrl::parse("http://example.com/b.html").unwrap().origin());
    let c = b.clone();
    assert!(a.same_origin(&c));
    assert!(b.same_origin(&b));
    assert!(c.same_origin(&b));
    assert_eq!(c.is_tuple(), true);
}

#[test]
fn clone_cross_origin() {
    let a = MutableOrigin::new(ServoUrl::parse("http://example.com/a.html").unwrap().origin());
    let b = MutableOrigin::new(ServoUrl::parse("http://example.org/b.html").unwrap().origin());
    let c = b.clone();
    assert!(!a.same_origin(&c));
    assert!(b.same_origin(&c));
    assert!(c.same_origin(&c));
}

#[test]
fn opaque() {
    let a = MutableOrigin::new(ImmutableOrigin::new_opaque());
    let b = MutableOrigin::new(ImmutableOrigin::new_opaque());
    assert!(!a.same_origin(&b));
    assert_eq!(a.is_tuple(), false);
}

#[test]
fn opaque_clone() {
    let a = MutableOrigin::new(ImmutableOrigin::new_opaque());
    let b = a.clone();
    assert!(a.same_origin(&b));
    assert_eq!(a.is_tuple(), false);
}
