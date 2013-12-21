/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::local_data;
use cssparser::ast::{SyntaxError, SourceLocation};


pub struct ErrorLoggerIterator<I>(I);

impl<T, I: Iterator<Result<T, SyntaxError>>> Iterator<T> for ErrorLoggerIterator<I> {
    fn next(&mut self) -> Option<T> {
        for result in **self {
            match result {
                Ok(v) => return Some(v),
                Err(error) => log_css_error(error.location, format!("{:?}", error.reason))
            }
        }
        None
    }
}


local_data_key!(silence_errors: ())

pub fn log_css_error(location: SourceLocation, message: &str) {
    // TODO eventually this will got into a "web console" or something.
    if local_data::get(silence_errors, |silenced| silenced.is_none()) {
        error!("{:u}:{:u} {:s}", location.line, location.column, message)
    }
}


pub fn with_errors_silenced<T>(f: || -> T) -> T {
    local_data::set(silence_errors, ());
    let result = f();
    local_data::pop(silence_errors);
    result
}
