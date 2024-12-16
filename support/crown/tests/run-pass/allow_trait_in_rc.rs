/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

#![allow(dead_code)]

use std::rc::Rc;

trait TypeHolderTrait {
    #[crown::unrooted_must_root_lint::must_root]
    #[crown::unrooted_must_root_lint::allow_unrooted_in_rc]
    type F;
}

struct TypeHolder;
impl TypeHolderTrait for TypeHolder {
    type F = Foo;
}

#[crown::unrooted_must_root_lint::must_root]
#[crown::unrooted_must_root_lint::allow_unrooted_in_rc]
struct Foo;

fn foo<T: TypeHolderTrait>() -> Rc<T::F> {
    unimplemented!()
}

fn main() {}
