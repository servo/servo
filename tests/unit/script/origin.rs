/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::origin::Origin;

#[test]
fn same_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.com/b.html"));
    assert!(a.same_origin(&b));
    assert_eq!(a.is_scheme_host_port_tuple(), true);
}

#[test]
fn identical_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    assert!(a.same_origin(&a));
}

#[test]
fn cross_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.org/b.html"));
    assert!(!a.same_origin(&b));
}

#[test]
fn alias_same_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.com/b.html"));
    let c = b.alias();
    assert!(a.same_origin(&c));
    assert!(b.same_origin(&b));
    assert!(c.same_origin(&b));
    assert_eq!(c.is_scheme_host_port_tuple(), true);
}

#[test]
fn alias_cross_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.org/b.html"));
    let c = b.alias();
    assert!(!a.same_origin(&c));
    assert!(b.same_origin(&c));
    assert!(c.same_origin(&c));
}

#[test]
fn alias_update_same_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.org/b.html"));
    let c = b.alias();
    b.set(url!("http://example.com/c.html").origin());
    assert!(a.same_origin(&c));
    assert!(b.same_origin(&c));
    assert!(c.same_origin(&c));
}

#[test]
fn alias_update_cross_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.com/b.html"));
    let c = b.alias();
    b.set(url!("http://example.org/c.html").origin());
    assert!(!a.same_origin(&c));
    assert!(b.same_origin(&c));
    assert!(c.same_origin(&c));
}

#[test]
fn alias_chain() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.com/b.html"));
    let c = b.copy();
    let d = c.alias();
    let e = d.alias();
    assert!(a.same_origin(&e));
    assert!(b.same_origin(&e));
    assert!(c.same_origin(&e));
    assert!(d.same_origin(&e));
    assert!(e.same_origin(&e));
    c.set(url!("http://example.org/c.html").origin());
    assert!(a.same_origin(&b));
    assert!(!b.same_origin(&c));
    assert!(c.same_origin(&d));
    assert!(d.same_origin(&e));
    assert!(!e.same_origin(&a));
}

#[test]
fn opaque() {
    let a = Origin::opaque_identifier();
    let b = Origin::opaque_identifier();
    assert!(!a.same_origin(&b));
    assert_eq!(a.is_scheme_host_port_tuple(), false);
}

#[test]
fn opaque_clone() {
    let a = Origin::opaque_identifier();
    let b = a.alias();
    assert!(a.same_origin(&b));
    assert_eq!(a.is_scheme_host_port_tuple(), false);
}
