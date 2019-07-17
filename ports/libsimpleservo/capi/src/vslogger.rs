/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use log::{self, Level, Metadata, Record};

extern "C" {
    fn OutputDebugStringA(s: *const u8);
}

pub struct VSLogger;

impl log::Log for VSLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Warn
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log = format!(
                "RUST: {} - {} - {}\r\n\0",
                record.level(),
                record.target(),
                record.args()
            );
            unsafe {
                OutputDebugStringA(log.as_ptr());
            };
        }
    }

    fn flush(&self) {}
}
