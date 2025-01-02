/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

/// Mock `JSTraceable`
pub trait JSTraceable {}
impl JSTraceable for i32 {}

// second generic argument must not be traceable
#[crown::trace_in_no_trace_lint::must_not_have_traceable(0)]
struct NoTrace<NoTraceable> {
    n: NoTraceable,
}

struct Foo(NoTrace<i32>);

fn main() {}
