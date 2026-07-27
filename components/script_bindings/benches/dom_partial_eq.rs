/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Microbenchmark for the `dom_struct`-generated `PartialEq` impl.
//!
//! This benchmark informs the choice of implementation for the `PartialEq`
//! impl that `#[dom_struct]` generates for DOM types (see
//! `components/dom_struct/domobject.rs`). Two candidates are compared:
//!
//! - `dom_eq_*` — the current impl, which compares reflector pointers via
//!   `DomObject::reflector(self) == DomObject::reflector(other)`,
//!   routing through multiple non-`#[inline]` cross-crate calls
//!   (`DomObject::reflector` -> `Reflector::reflector` -> `Reflector::PartialEq`).
//!
//! - `ptr_eq_*` — a direct `std::ptr::eq(self, other)` comparison, which
//!   compares the Rust allocation addresses of the two DOM objects.
//!
//! Measurements show the current reflector comparison is consistently
//! faster than `std::ptr::eq`, which is counter-intuitive.
//!
//! For results, see <https://github.com/servo/servo/wiki/DOM-%60PartialEq%60-benchmark>

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
