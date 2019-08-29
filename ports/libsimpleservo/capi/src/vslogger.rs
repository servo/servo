/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::LOG_MODULE_FILTERS;
use log::{self, Level, Metadata, Record};
use std::ffi::CStr;
use std::os::raw::c_char;

extern "C" {
    fn OutputDebugStringA(s: *const u8);
}

pub struct VsloggerModuleFilter {
    pub vslogger_filter: Vec<String>,
}

impl VsloggerModuleFilter {
    pub fn is_log_module_empty(&self) -> bool {
        if self.vslogger_filter.is_empty() {
            return true;
        } else {
            return false;
        }
    }
}

pub struct VSLogger;

impl VSLogger {
    pub fn add_module_to_filter(&self, mod_list: *mut *mut c_char, _mod_size: i32) -> Vec<String> {
        let _vec = (0..(_mod_size - 1))
            .map(|i| unsafe {
                CStr::from_ptr(*mod_list.offset(i as isize))
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();

        return _vec;
    }
}

impl log::Log for VSLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if !LOG_MODULE_FILTERS.lock().unwrap().is_log_module_empty() {
            if LOG_MODULE_FILTERS
                .lock()
                .unwrap()
                .vslogger_filter
                .contains(&String::from(metadata.target()))
            {
                return metadata.level() <= Level::Warn;
            } else {
                return false;
            }
        } else {
            return metadata.level() <= Level::Warn;
        }
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
