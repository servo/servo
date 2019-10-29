/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use log::{self, Metadata, Record};
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref LOG_MODULE_FILTERS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
}

extern "C" {
    fn OutputDebugStringA(s: *const u8);
}

pub struct VSLogger;

impl log::Log for VSLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let modules = LOG_MODULE_FILTERS.lock().unwrap();
        let is_module_enabled =
            modules.contains(&String::from(metadata.target())) || modules.is_empty();
        return is_module_enabled;
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
