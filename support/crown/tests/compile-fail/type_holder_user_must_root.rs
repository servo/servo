/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

struct Foo(i32);

struct Bar<TH: TypeHolderTrait>(TH::F);
//~^ ERROR: Type must be rooted, use #[crown::unrooted_must_root_lint::must_root] on the struct definition to propagate

trait TypeHolderTrait {
    #[crown::unrooted_must_root_lint::must_root]
    type F;
    //~^ Mismatched use of #[crown::unrooted_must_root_lint::must_root] between associated type declaration and impl definition. [crown::unrooted_must_root]
}

struct TypeHolder;

impl TypeHolderTrait for TypeHolder {
    type F = Foo;
}

fn main() {}
