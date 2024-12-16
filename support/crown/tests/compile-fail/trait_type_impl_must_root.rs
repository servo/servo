/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

#[crown::unrooted_must_root_lint::must_root]
struct Foo;

trait Trait {
    type F;
    //~^ ERROR: Type trait declaration must be marked with #[crown::unrooted_must_root_lint::must_root] to allow binding must_root types in associated types
}

struct TypeHolder;

impl Trait for TypeHolder {
    // type F in trait must be also marked as must_root if we want to do this
    type F = Foo;
}

fn main() {}
