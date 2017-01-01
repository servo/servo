/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use test::Bencher;

struct Foo {
    u: u32,
}

struct Bar {
    f: Foo,
}

impl Default for Bar {
    fn default() -> Self {
        Bar { f: Foo { u: 42 } }
    }
}

// FIXME(bholley): Add tests to exercise this in concurrent scenarios.

#[test]
fn immutable() {
    let a = AtomicRefCell::new(Bar::default());
    let _first = a.borrow();
    let _second = a.borrow();
}

#[test]
fn mutable() {
    let a = AtomicRefCell::new(Bar::default());
    let _ = a.borrow_mut();
}

#[test]
fn interleaved() {
    let a = AtomicRefCell::new(Bar::default());
    {
        let _ = a.borrow_mut();
    }
    {
        let _first = a.borrow();
        let _second = a.borrow();
    }
    {
        let _ = a.borrow_mut();
    }
}

#[test]
#[should_panic(expected = "already immutably borrowed")]
fn immutable_then_mutable() {
    let a = AtomicRefCell::new(Bar::default());
    let _first = a.borrow();
    let _second = a.borrow_mut();
}

#[test]
#[should_panic(expected = "already mutably borrowed")]
fn mutable_then_immutable() {
    let a = AtomicRefCell::new(Bar::default());
    let _first = a.borrow_mut();
    let _second = a.borrow();
}

#[test]
#[should_panic(expected = "already mutably borrowed")]
fn double_mutable() {
    let a = AtomicRefCell::new(Bar::default());
    let _first = a.borrow_mut();
    let _second = a.borrow_mut();
}

#[test]
fn map() {
    let a = AtomicRefCell::new(Bar::default());
    let b = a.borrow();
    assert_eq!(b.f.u, 42);
    let c = AtomicRef::map(b, |x| &x.f);
    assert_eq!(c.u, 42);
    let d = AtomicRef::map(c, |x| &x.u);
    assert_eq!(*d, 42);
}

#[bench]
fn immutable_borrow(b: &mut Bencher) {
    let a = AtomicRefCell::new(Bar::default());
    b.iter(|| a.borrow());
}

#[bench]
fn immutable_second_borrow(b: &mut Bencher) {
    let a = AtomicRefCell::new(Bar::default());
    let _first = a.borrow();
    b.iter(|| a.borrow());
}

#[bench]
fn immutable_third_borrow(b: &mut Bencher) {
    let a = AtomicRefCell::new(Bar::default());
    let _first = a.borrow();
    let _second = a.borrow();
    b.iter(|| a.borrow());
}

#[bench]
fn mutable_borrow(b: &mut Bencher) {
    let a = AtomicRefCell::new(Bar::default());
    b.iter(|| a.borrow_mut());
}

#[test]
fn map_mut() {
    let a = AtomicRefCell::new(Bar::default());
    let mut b = a.borrow_mut();
    assert_eq!(b.f.u, 42);
    b.f.u = 43;
    let mut c = AtomicRefMut::map(b, |x| &mut x.f);
    assert_eq!(c.u, 43);
    c.u = 44;
    let mut d = AtomicRefMut::map(c, |x| &mut x.u);
    assert_eq!(*d, 44);
    *d = 45;
    assert_eq!(*d, 45);
}
