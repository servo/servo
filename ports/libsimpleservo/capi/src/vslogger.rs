/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use log::{self, Level, Metadata, Record};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref LOG_MODULE_FILTERS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    pub static ref LOG_PTR_FUNC: Arc<Mutex<Option<fn(*const c_char) -> bool>>> =
        Arc::new(Mutex::new(Option::None));
}

pub struct VSLogger;

impl log::Log for VSLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let modules = LOG_MODULE_FILTERS.lock().unwrap();
        let is_module_enabled =
            modules.contains(&String::from(metadata.target())) || modules.is_empty();
        return metadata.level() <= Level::Warn && is_module_enabled;
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log = format!(
                "RUST: {} - {} - {}\r\n\0",
                record.level(),
                record.target(),
                record.args()
            );
            let log_fn = &*LOG_PTR_FUNC.lock().unwrap();
            if let Some(log_fn) = *log_fn {
                log_fn(log.as_ptr() as *const c_char);
            }
        }
    }

    fn flush(&self) {}
}
