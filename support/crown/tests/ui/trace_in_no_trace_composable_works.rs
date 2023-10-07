/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Fake jstraceable
pub trait JSTraceable {}

struct TraceableStruct;
impl JSTraceable for TraceableStruct {}

struct NotTraceableStruct;

// second generic argument must not be traceable
#[crown::trace_in_no_trace_lint::must_not_have_traceable(1)]
struct NoTraceComposable<Traceable, NoTraceable> {
    t: Traceable,
    n: NoTraceable,
}

// this is not ok i32 is traceable
struct Foo(NoTraceComposable<NotTraceableStruct, TraceableStruct>);

fn main() {}
