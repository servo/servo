/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, feature(register_tool))]
#![cfg_attr(crown, register_tool(crown))]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use dom_struct::dom_struct;
use js::gc::Traceable as JSTraceable;
use jstraceable_derive::JSTraceable;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::inheritance::HasParent;
use script_bindings::reflector::{DomObject, MutDomObject, Reflector};

/// A minimal `#[dom_struct]` type to benchmark the auto-generated `PartialEq`
/// impl for `components/dom_struct/domobject.rs`.
/// Technically, that is:
/// `crate::DomObject::reflector(self) == crate::DomObject::reflector(other)`
/// which routes through `DomObject::reflector()` -> `Reflector::reflector()` ->
/// `Reflector::PartialEq` (three non-inlined cross-crate calls).
///
/// The baseline `ptr_eq_*` calls `std::ptr::eq` directly.
///
/// Turns out, `PartialEq` is faster than `std::ptr::eq` which is counter intuitive.
#[dom_struct]
struct BenchDom {
    reflector: Reflector,
}

fn bench(c: &mut Criterion) {
    let a = Box::new(BenchDom {
        reflector: Reflector::new(),
    });
    let b = Box::new(BenchDom {
        reflector: Reflector::new(),
    });
    let same = &*a;
    let other = &*b;

    c.bench_function("ptr_eq_same", |bencher| {
        bencher.iter(|| black_box(std::ptr::eq(black_box(same), black_box(same))))
    });
    c.bench_function("ptr_eq_different", |bencher| {
        bencher.iter(|| black_box(std::ptr::eq(black_box(same), black_box(other))))
    });

    c.bench_function("dom_eq_same", |bencher| {
        bencher.iter(|| black_box(black_box(same) == black_box(same)))
    });
    c.bench_function("dom_eq_different", |bencher| {
        bencher.iter(|| black_box(black_box(same) == black_box(other)))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
