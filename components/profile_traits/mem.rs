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

/// The various kinds of memory measurement.
///
/// Here "explicit" means explicit memory allocations done by the application. It includes
/// allocations made at the OS level (via functions such as VirtualAlloc, vm_allocate, and mmap),
/// allocations made at the heap allocation level (via functions such as malloc, calloc, realloc,
/// memalign, operator new, and operator new[]) and where possible, the overhead of the heap
/// allocator itself. It excludes memory that is mapped implicitly such as code and data segments,
/// and thread stacks. "explicit" is not guaranteed to cover every explicit allocation, but it does
/// cover most (including the entire heap), and therefore it is the single best number to focus on
/// when trying to reduce memory usage.
#[derive(Deserialize, Serialize)]
pub enum ReportKind {
    /// A size measurement for an explicit allocation on the jemalloc heap. This should be used
    /// for any measurements done via the `HeapSizeOf` trait.
    ExplicitJemallocHeapSize,

    /// A size measurement for an explicit allocation on the system heap. Only likely to be used
    /// for external C or C++ libraries that don't use jemalloc.
    ExplicitSystemHeapSize,

    /// A size measurement for an explicit allocation not on the heap, e.g. via mmap().
    ExplicitNonHeapSize,

    /// A size measurement for an explicit allocation whose location is unknown or uncertain.
    ExplicitUnknownLocationSize,

    /// A size measurement for a non-explicit allocation. This kind is used for global
    /// measurements such as "resident" and "vsize", and also for measurements that cross-cut the
    /// measurements grouped under "explicit", e.g. by grouping those measurements in a way that's
    /// different to how they are grouped under "explicit".
    NonExplicitSize,
}

/// A single memory-related measurement.
#[derive(Deserialize, Serialize)]
pub struct Report {
    /// The identifying path for this report.
    pub path: Vec<String>,

    /// The report kind.
    pub kind: ReportKind,

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

/// The protocol used to send reporter requests.
#[derive(Deserialize, Serialize)]
pub struct ReporterRequest {
    /// The channel on which reports are to be sent.
    pub reports_channel: ReportsChan,
}

/// A memory reporter is capable of measuring some data structure of interest. It's structured as
/// an IPC sender that a `ReporterRequest` in transmitted over. `ReporterRequest` objects in turn
/// encapsulate the channel on which the memory profiling information is to be sent.
///
/// In many cases, clients construct `Reporter` objects by creating an IPC sender/receiver pair and
/// registering the receiving end with the router so that messages from the memory profiler end up
/// injected into the client's event loop.
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

