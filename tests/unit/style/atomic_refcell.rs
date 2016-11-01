/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::atomic_refcell::{AtomicRef, AtomicRefCell};

struct Foo {
    u: u32,
}

struct Bar {
    f: Foo,
}

#[test]
fn map() {
    let a = AtomicRefCell::new(Bar { f: Foo { u: 42 } });
    let b = a.borrow();
    assert_eq!(b.f.u, 42);
    let c = AtomicRef::map(b, |x| &x.f);
    assert_eq!(c.u, 42);
    let d = AtomicRef::map(c, |x| &x.u);
    assert_eq!(*d, 42);
}

/* FIXME(bholley): Enable once we have AtomicRefMut::map(), which is blocked on
 * https://github.com/Kimundi/owning-ref-rs/pull/16
#[test]
fn map_mut() {
    let a = AtomicRefCell::new(Bar { f: Foo { u: 42 } });
    let mut b = a.borrow_mut();
    assert_eq!(b.f.u, 42);
    b.f.u = 43;
    let mut c = AtomicRefMut::map(b, |x| &x.f);
    assert_eq!(c.u, 43);
    c.u = 44;
    let mut d = AtomicRefMut::map(c, |x| &x.u);
    assert_eq!(*d, 44);
    *d. = 45;
    assert_eq!(*d, 45);
}*/
