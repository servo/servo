/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

/// Mock `JSTraceable`
pub trait JSTraceable {}

struct TraceableStruct;
impl JSTraceable for TraceableStruct {}

struct NotTraceableStruct;

#[crown::trace_in_no_trace_lint::must_not_have_traceable]
struct NoTrace<T>(T);

struct Foo(NoTrace<TraceableStruct>);
//~^ ERROR: must_not_have_traceable marked wrapper must not have jsmanaged inside on 0-th position. Consider removing the wrapper.

fn main() {}
