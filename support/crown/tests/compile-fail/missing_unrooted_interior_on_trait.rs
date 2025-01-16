/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

#![allow(dead_code)]

trait TypeHolderTrait {
    #[crown::unrooted_must_root_lint::must_root]
    type F: Default;
}

struct TypeHolder;
impl TypeHolderTrait for TypeHolder {
    type F = Foo;
}

#[crown::unrooted_must_root_lint::must_root]
#[derive(Default)]
struct Foo;

struct MustBeRooted<T: TypeHolderTrait>(T::F);
//~^ ERROR: Type must be rooted, use #[crown::unrooted_must_root_lint::must_root] on the struct definition to propagate. [crown::unrooted_must_root]

fn main() {}
