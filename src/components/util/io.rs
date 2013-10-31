/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rt::io::{io_error, EndOfFile};

/// Ignore the end-of-file condition within a block of code.
pub fn ignoring_eof<U>(cb: &fn() -> U) -> U {
    io_error::cond.trap(|e|
        match e.kind {
            EndOfFile => (),
            _ => io_error::cond.raise(e)
        }).inside(cb)
}
