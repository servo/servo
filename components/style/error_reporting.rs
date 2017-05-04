/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types used to report parsing errors.

#![deny(missing_docs)]

use cssparser::{Parser, SourcePosition};
use log;
use stylesheets::UrlExtraData;

/// A generic trait for an error reporter.
pub trait ParseErrorReporter : Sync + Send {
    /// Called when the style engine detects an error.
    ///
    /// Returns the current input being parsed, the source position it was
    /// reported from, and a message.
    fn report_error(&self,
                    input: &mut Parser,
                    position: SourcePosition,
                    message: &str,
                    url: &UrlExtraData,
                    line_number_offset: u64);
}

/// An error reporter that uses [the `log` crate](https://github.com/rust-lang-nursery/log)
/// at `info` level.
///
/// This logging is silent by default, and can be enabled with a `RUST_LOG=style=info`
/// environment variable.
/// (See [`env_logger`](https://rust-lang-nursery.github.io/log/env_logger/).)
pub struct RustLogReporter;

impl ParseErrorReporter for RustLogReporter {
    fn report_error(&self,
                    input: &mut Parser,
                    position: SourcePosition,
                    message: &str,
                    url: &UrlExtraData,
                    line_number_offset: u64) {
        if log_enabled!(log::LogLevel::Info) {
            let location = input.source_location(position);
            let line_offset = location.line + line_number_offset as usize;
            info!("Url:\t{}\n{}:{} {}", url.as_str(), line_offset, location.column, message)
        }
    }
}

/// Error reporter which silently forgets errors
pub struct NullReporter;

impl ParseErrorReporter for NullReporter {
    fn report_error(&self,
            _: &mut Parser,
            _: SourcePosition,
            _: &str,
            _: &UrlExtraData,
            _: u64) {
        // do nothing
    }
}
