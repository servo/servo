/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use gstreamer::DebugCategory;
use gstreamer::DebugColorFlags;
use gstreamer::DebugLevel;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CATEGORY: DebugCategory =
        DebugCategory::new("servosrc", DebugColorFlags::empty(), Some("Servo"));
}

pub static LOGGER: ServoSrcLogger = ServoSrcLogger;

pub struct ServoSrcLogger;

impl log::Log for ServoSrcLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let lvl = match record.level() {
            log::Level::Error => DebugLevel::Error,
            log::Level::Warn => DebugLevel::Warning,
            log::Level::Info => DebugLevel::Info,
            log::Level::Debug => DebugLevel::Debug,
            log::Level::Trace => DebugLevel::Trace,
        };
        CATEGORY.log::<gstreamer::Object>(
            None,
            lvl,
            record.file().unwrap_or(""),
            record.module_path().unwrap_or(""),
            record.line().unwrap_or(0),
            record.args().clone(),
        );
    }

    fn flush(&self) {}
}
