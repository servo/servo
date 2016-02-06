/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::origin::Origin;

#[test]
fn same_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.com/b.html"));
    assert!(a == b);
    assert_eq!(a.is_scheme_host_port_tuple(), true);
}

#[test]
fn identical_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    assert!(a == a);
}

#[test]
fn cross_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.org/b.html"));
    assert!(a != b);
}

#[test]
fn alias_same_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.com/b.html"));
    let c = b.alias();
    assert!(a == c);
    assert!(b == c);
    assert!(c == c);
    assert_eq!(c.is_scheme_host_port_tuple(), true);
}

#[test]
fn alias_cross_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.org/b.html"));
    let c = b.alias();
    assert!(a != c);
    assert!(b == c);
    assert!(c == c);
}

#[test]
fn alias_update_same_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.org/b.html"));
    let c = b.alias();
    b.set(Origin::new(&url!("http://example.com/c.html")));
    assert!(a == c);
    assert!(b == c);
    assert!(c == c);
}

#[test]
fn alias_clone_update_same_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.org/b.html"));
    let c = b.alias();
    let d = c.clone();
    b.set(Origin::new(&url!("http://example.com/c.html")));
    assert!(a == c);
    assert!(b == c);
    assert!(c == d);
}

#[test]
fn alias_update_cross_origin() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.com/b.html"));
    let c = b.alias();
    b.set(Origin::new(&url!("http://example.org/c.html")));
    assert!(a != c);
    assert!(b == c);
    assert!(c == c);
}

#[test]
fn alias_chain() {
    let a = Origin::new(&url!("http://example.com/a.html"));
    let b = Origin::new(&url!("http://example.com/b.html"));
    let c = b.alias();
    let d = c.alias();
    let e = d.alias();
    assert!(a == e);
    assert!(b == e);
    assert!(c == e);
    assert!(d == e);
    assert!(e == e);
    c.set(Origin::new(&url!("http://example.org/c.html")));
    assert!(a == b);
    assert!(b != c);
    assert!(c == d);
    assert!(d == e);
    assert!(e != a);
}

#[test]
fn opaque() {
    let a = Origin::opaque_identifier();
    let b = Origin::opaque_identifier();
    assert!(a != b);
    assert_eq!(a.is_scheme_host_port_tuple(), false);
}

#[test]
fn opaque_clone() {
    let a = Origin::opaque_identifier();
    let b = a.clone();
    assert!(a == b);
    assert_eq!(a.is_scheme_host_port_tuple(), false);
}

#[test]
fn opaque_alias() {
    let a = Origin::opaque_identifier();
    let b = a.alias();
    assert!(a == b);
    assert_eq!(b.is_scheme_host_port_tuple(), false);
}

#[test]
fn set_deliases() {
    let a = Origin::new(&url!("http://example.com/foo.html"));
    let b = a.alias();
    a.set(b.alias());
    assert!(a == b);
    assert_eq!(a.is_scheme_host_port_tuple(), true);
    assert_eq!(b.is_scheme_host_port_tuple(), true);
}
