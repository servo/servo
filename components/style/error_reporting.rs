/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types used to report parsing errors.

#![deny(missing_docs)]

use cssparser::{Parser, SourcePosition};
use log;

/// A generic trait for an error reporter.
pub trait ParseErrorReporter {
    /// Called the style engine detects an error.
    ///
    /// Returns the current input being parsed, the source position it was
    /// reported from, and a message.
    fn report_error(&self, input: &mut Parser, position: SourcePosition, message: &str);
    /// Clone this error reporter.
    ///
    /// TODO(emilio): I'm pretty sure all the box shenanigans can go away.
    fn clone(&self) -> Box<ParseErrorReporter + Send + Sync>;
}

/// An error reporter that reports the errors to the `info` log channel.
///
/// TODO(emilio): The name of this reporter is a lie, and should be renamed!
pub struct StdoutErrorReporter;
impl ParseErrorReporter for StdoutErrorReporter {
    fn report_error(&self, input: &mut Parser, position: SourcePosition, message: &str) {
         if log_enabled!(log::LogLevel::Info) {
             let location = input.source_location(position);
             info!("{}:{} {}", location.line, location.column, message)
         }
    }

    fn clone(&self) -> Box<ParseErrorReporter + Send + Sync> {
        Box::new(StdoutErrorReporter)
    }
}
