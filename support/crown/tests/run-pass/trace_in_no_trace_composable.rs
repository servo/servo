/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

/// Mock `JSTraceable`
pub trait JSTraceable {}

struct TraceableStruct;
impl JSTraceable for TraceableStruct {}

struct NotTraceableStruct;

// `must_not_have_traceable(1)` indicates that the second generic argument should
// not be traceable and this test verifies that this lint passes.
#[crown::trace_in_no_trace_lint::must_not_have_traceable(1)]
struct NoTraceComposable<Traceable, NoTraceable> {
    t: Traceable,
    n: NoTraceable,
}

// this is ok: TraceableStruct is traceable
struct Foo(NoTraceComposable<TraceableStruct, NotTraceableStruct>);

fn main() {}
