/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
// compile-flags: --error-format=human
//@rustc-env:RUSTC_BOOTSTRAP=1

#[crown::unrooted_must_root_lint::must_root]
struct Foo(i32);
#[crown::unrooted_must_root_lint::must_root]
struct Bar(Foo);

fn foo1(_: &Foo) {}
fn foo2(_: &()) -> &Foo {
    unimplemented!()
}

fn main() {}
