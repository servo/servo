/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::{io_error, IoError};

/// Helper for catching an I/O error and wrapping it in a Result object. The
/// return result will be the last I/O error that happened or the result of the
/// closure if no error occurred.
///
/// FIXME: This is a copy of std::rt::io::result which doesn't exist yet in our
/// version of Rust.  We should switch after the next Rust upgrade.
pub fn result<T>(cb: || -> T) -> Result<T, IoError> {
    let mut err = None;
    let ret = io_error::cond.trap(|e| err = Some(e)).inside(cb);
    match err {
        Some(e) => Err(e),
        None => Ok(ret),
    }
}
