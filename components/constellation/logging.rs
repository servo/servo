/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::sync::Arc;
use std::thread;

use backtrace::Backtrace;
use base::id::TopLevelBrowsingContextId;
use compositing_traits::ConstellationMsg as FromCompositorMsg;
use crossbeam_channel::Sender;
use log::{Level, LevelFilter, Log, Metadata, Record};
use parking_lot::ReentrantMutex;
use script_traits::{LogEntry, ScriptMsg as FromScriptMsg, ScriptToConstellationChan};

/// The constellation uses logging to perform crash reporting.
/// The constellation receives all `warn!`, `error!` and `panic!` messages,
/// and generates a crash report when it receives a panic.

/// A logger directed at the constellation from content processes
/// #[derive(Clone)]
pub struct FromScriptLogger {
    /// A channel to the constellation
    pub script_to_constellation_chan: Arc<ReentrantMutex<ScriptToConstellationChan>>,
}

/// The constellation uses logging to perform crash reporting.
/// The constellation receives all `warn!`, `error!` and `panic!` messages,
/// and generates a crash report when it receives a panic.

/// A logger directed at the constellation from content processes
impl FromScriptLogger {
    /// Create a new constellation logger.
    pub fn new(script_to_constellation_chan: ScriptToConstellationChan) -> FromScriptLogger {
        FromScriptLogger {
            script_to_constellation_chan: Arc::new(ReentrantMutex::new(
                script_to_constellation_chan,
            )),
        }
    }

    /// The maximum log level the constellation logger is interested in.
    pub fn filter(&self) -> LevelFilter {
        LevelFilter::Warn
    }
}

impl Log for FromScriptLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Warn
    }

    fn log(&self, record: &Record) {
        if let Some(entry) = log_entry(record) {
            let thread_name = thread::current().name().map(ToOwned::to_owned);
            let msg = FromScriptMsg::LogEntry(thread_name, entry);
            let chan = self.script_to_constellation_chan.lock();
            let _ = chan.send(msg);
        }
    }

    fn flush(&self) {}
}

/// A logger directed at the constellation from the compositor
#[derive(Clone)]
pub struct FromCompositorLogger {
    /// A channel to the constellation
    pub constellation_chan: Arc<ReentrantMutex<Sender<FromCompositorMsg>>>,
}

impl FromCompositorLogger {
    /// Create a new constellation logger.
    pub fn new(constellation_chan: Sender<FromCompositorMsg>) -> FromCompositorLogger {
        FromCompositorLogger {
            constellation_chan: Arc::new(ReentrantMutex::new(constellation_chan)),
        }
    }

    /// The maximum log level the constellation logger is interested in.
    pub fn filter(&self) -> LevelFilter {
        LevelFilter::Warn
    }
}

impl Log for FromCompositorLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Warn
    }

    fn log(&self, record: &Record) {
        if let Some(entry) = log_entry(record) {
            let top_level_id = TopLevelBrowsingContextId::installed();
            let thread_name = thread::current().name().map(ToOwned::to_owned);
            let msg = FromCompositorMsg::LogEntry(top_level_id, thread_name, entry);
            let chan = self.constellation_chan.lock();
            let _ = chan.send(msg);
        }
    }

    fn flush(&self) {}
}

/// Rust uses `Record` for storing logging, but servo converts that to
/// a `LogEntry`. We do this so that we can record panics as well as log
/// messages, and because `Record` does not implement serde (de)serialization,
/// so cannot be used over an IPC channel.
fn log_entry(record: &Record) -> Option<LogEntry> {
    match record.level() {
        Level::Error if thread::panicking() => Some(LogEntry::Panic(
            format!("{}", record.args()),
            format!("{:?}", Backtrace::new()),
        )),
        Level::Error => Some(LogEntry::Error(format!("{}", record.args()))),
        Level::Warn => Some(LogEntry::Warn(format!("{}", record.args()))),
        _ => None,
    }
}
