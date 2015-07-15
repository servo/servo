/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! APIs for memory profiling.

#![deny(missing_docs)]

use ipc_channel::ipc::IpcSender;

/// Front-end representation of the profiler used to communicate with the
/// profiler.
#[derive(Clone)]
pub struct ProfilerChan(pub IpcSender<ProfilerMsg>);

impl ProfilerChan {
    /// Send `msg` on this `IpcSender`.
    ///
    /// Panics if the send fails.
    pub fn send(&self, msg: ProfilerMsg) {
        let ProfilerChan(ref c) = *self;
        c.send(msg).unwrap();
    }
}

/// A single memory-related measurement.
#[derive(Deserialize, Serialize)]
pub struct Report {
    /// The identifying path for this report.
    pub path: Vec<String>,

    /// The size, in bytes.
    pub size: usize,
}

/// A channel through which memory reports can be sent.
#[derive(Clone, Deserialize, Serialize)]
pub struct ReportsChan(pub IpcSender<Vec<Report>>);

impl ReportsChan {
    /// Send `report` on this `IpcSender`.
    ///
    /// Panics if the send fails.
    pub fn send(&self, report: Vec<Report>) {
        let ReportsChan(ref c) = *self;
        c.send(report).unwrap();
    }
}

/// A memory reporter is capable of measuring some data structure of interest. Because it needs to
/// be passed to and registered with the Profiler, it's typically a "small" (i.e. easily cloneable)
/// value that provides access to a "large" data structure, e.g. a channel that can inject a
/// request for measurements into the event queue associated with the "large" data structure.
#[derive(Deserialize, Serialize)]
pub struct Reporter(pub IpcSender<ReporterRequest>);

impl Reporter {
    /// Collect one or more memory reports. Returns true on success, and false on failure.
    pub fn collect_reports(&self, reports_chan: ReportsChan) {
        self.0.send(ReporterRequest {
            reports_channel: reports_chan,
        }).unwrap()
    }
}

/// The protocol used to send reporter requests.
#[derive(Deserialize, Serialize)]
pub struct ReporterRequest {
    /// The channel on which reports are to be sent.
    pub reports_channel: ReportsChan,
}

/// An easy way to build a path for a report.
#[macro_export]
macro_rules! path {
    ($($x:expr),*) => {{
        use std::borrow::ToOwned;
        vec![$( $x.to_owned() ),*]
    }}
}

/// Messages that can be sent to the memory profiler thread.
#[derive(Deserialize, Serialize)]
pub enum ProfilerMsg {
    /// Register a Reporter with the memory profiler. The String is only used to identify the
    /// reporter so it can be unregistered later. The String must be distinct from that used by any
    /// other registered reporter otherwise a panic will occur.
    RegisterReporter(String, Reporter),

    /// Unregister a Reporter with the memory profiler. The String must match the name given when
    /// the reporter was registered. If the String does not match the name of a registered reporter
    /// a panic will occur.
    UnregisterReporter(String),

    /// Triggers printing of the memory profiling metrics.
    Print,

    /// Tells the memory profiler to shut down.
    Exit,
}

