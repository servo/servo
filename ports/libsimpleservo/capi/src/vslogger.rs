/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::OUTPUT_LOG_HANDLER;
use log::{self, Metadata, Record};
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref LOG_MODULE_FILTERS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
}

pub struct VSLogger;

impl log::Log for VSLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let modules = LOG_MODULE_FILTERS.lock().unwrap();
        modules.is_empty() ||
            modules.iter().any(|module| {
                metadata.target() == module ||
                    metadata.target().starts_with(&format!("{}::", module))
            })
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log = format!(
                "RUST: {} - {} - {}\r\n\0",
                record.level(),
                record.target(),
                record.args()
            );
            if let Some(handler) = OUTPUT_LOG_HANDLER.lock().unwrap().as_ref() {
                (handler)(log.as_ptr() as _, log.len() as u32);
            }
        }
    }

    fn flush(&self) {}
}
