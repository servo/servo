/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Fake jstraceable
pub trait JSTraceable {}

#[crown::trace_in_no_trace_lint::must_not_have_traceable]
struct NoTrace<T>(T);

struct Bar;
impl JSTraceable for Bar {}

struct Foo(NoTrace<Bar>);

fn main() {}
