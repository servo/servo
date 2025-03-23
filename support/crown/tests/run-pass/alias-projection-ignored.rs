/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

#![allow(dead_code)]

fn main() {}

struct CanvasId(u64);

trait CanvasContext {
    type ID;

    fn context_id(&self) -> Self::ID;
}

#[crown::unrooted_must_root_lint::must_root]
struct Context;

impl CanvasContext for Context {
    type ID = CanvasId;

    fn context_id(&self) -> Self::ID { CanvasId(0) }
}
