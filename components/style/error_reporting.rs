/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types used to report parsing errors.

#![deny(missing_docs)]

use cssparser::{Parser, SourcePosition};
use log;
use servo_url::ServoUrl;

/// A generic trait for an error reporter.
pub trait ParseErrorReporter : Sync + Send {
    /// Called the style engine detects an error.
    ///
    /// Returns the current input being parsed, the source position it was
    /// reported from, and a message.
    fn report_error(&self,
                    input: &mut Parser,
                    position: SourcePosition,
                    message: &str,
                    url: &ServoUrl);
}

/// An error reporter that reports the errors to the `info` log channel.
///
/// TODO(emilio): The name of this reporter is a lie, and should be renamed!
pub struct StdoutErrorReporter;
impl ParseErrorReporter for StdoutErrorReporter {
    fn report_error(&self,
                    input: &mut Parser,
                    position: SourcePosition,
                    message: &str,
                    url: &ServoUrl) {
        if log_enabled!(log::LogLevel::Info) {
            let location = input.source_location(position);
            info!("Url:\t{}\n{}:{} {}", url.as_str(), location.line, location.column, message)
        }
    }
}
