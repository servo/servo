/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

#![allow(dead_code)]

#[crown::unrooted_must_root_lint::must_root]
struct MustBeRooted;

struct CanBeUnrooted {
    val: MustBeRooted,
    //~^ ERROR: Type must be rooted, use #[crown::unrooted_must_root_lint::must_root] on the struct definition to propagate. [crown::unrooted_must_root]
}


fn main() {}
