/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, SourcePosition};
use log;

pub trait ParseErrorReporter {
    fn report_error(&self, input: &mut Parser, position: SourcePosition, message: &str);
    fn clone(&self) -> Box<ParseErrorReporter + Send + Sync>;
}

pub struct StdoutErrorReporter;
impl ParseErrorReporter for StdoutErrorReporter {
    fn report_error(&self, input: &mut Parser, position: SourcePosition, message: &str) {
         if log_enabled!(log::LogLevel::Info) {
             let location = input.source_location(position);
             info!("{}:{} {}", location.line, location.column, message)
         }
    }

    fn clone(&self) -> Box<ParseErrorReporter + Send + Sync> {
        box StdoutErrorReporter
    }
}
