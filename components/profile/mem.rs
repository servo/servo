/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Memory profiling functions.

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::thread;

use ipc_channel::ipc::{self, IpcReceiver};
use ipc_channel::router::ROUTER;
use log::debug;
use profile_traits::mem::{
    MemoryReport, MemoryReportResult, ProfilerChan, ProfilerMsg, Report, Reporter, ReporterRequest,
    ReportsChan,
};

use crate::system_reporter;

pub struct Profiler {
    /// The port through which messages are received.
    pub port: IpcReceiver<ProfilerMsg>,

    /// Registered memory reporters.
    reporters: HashMap<String, Reporter>,
}

impl Profiler {
    pub fn create() -> ProfilerChan {
        let (chan, port) = ipc::channel().unwrap();

        // Always spawn the memory profiler. If there is no timer thread it won't receive regular
        // `Print` events, but it will still receive the other events.
        thread::Builder::new()
            .name("MemoryProfiler".to_owned())
            .spawn(move || {
                let mut mem_profiler = Profiler::new(port);
                mem_profiler.start();
            })
            .expect("Thread spawning failed");

        let mem_profiler_chan = ProfilerChan(chan);

        // Register the system memory reporter, which will run on its own thread. It never needs to
        // be unregistered, because as long as the memory profiler is running the system memory
        // reporter can make measurements.
        let (system_reporter_sender, system_reporter_receiver) = ipc::channel().unwrap();
        ROUTER.add_typed_route(
            system_reporter_receiver,
            Box::new(|message| {
                let request: ReporterRequest = message.unwrap();
                system_reporter::collect_reports(request)
            }),
        );
        mem_profiler_chan.send(ProfilerMsg::RegisterReporter(
            "system-main".to_owned(),
            Reporter(system_reporter_sender),
        ));

        mem_profiler_chan
    }

    pub fn new(port: IpcReceiver<ProfilerMsg>) -> Profiler {
        Profiler {
            port,
            reporters: HashMap::new(),
        }
    }

    pub fn start(&mut self) {
        while let Ok(msg) = self.port.recv() {
            if !self.handle_msg(msg) {
                break;
            }
        }
    }

    fn handle_msg(&mut self, msg: ProfilerMsg) -> bool {
        match msg {
            ProfilerMsg::RegisterReporter(name, reporter) => {
                debug!("Registering memory reporter: {}", name);
                // Panic if it has already been registered.
                let name_clone = name.clone();
                match self.reporters.insert(name, reporter) {
                    None => true,
                    Some(_) => panic!("RegisterReporter: '{}' name is already in use", name_clone),
                }
            },

            ProfilerMsg::UnregisterReporter(name) => {
                debug!("Unregistering memory reporter: {}", name);
                // Panic if it hasn't previously been registered.
                match self.reporters.remove(&name) {
                    Some(_) => true,
                    None => panic!("UnregisterReporter: '{}' name is unknown", &name),
                }
            },

            ProfilerMsg::Report(sender) => {
                let main_pid = std::process::id();

                let reports = self.collect_reports();
                // Turn the pid -> reports map into a vector and add the
                // hint to find the main process.
                let results: Vec<MemoryReport> = reports
                    .into_iter()
                    .map(|(pid, reports)| MemoryReport {
                        pid,
                        reports,
                        is_main_process: pid == main_pid,
                    })
                    .collect();
                let _ = sender.send(MemoryReportResult { results });
                true
            },

            ProfilerMsg::Exit => false,
        }
    }

    /// Returns a map of pid -> reports
    fn collect_reports(&self) -> HashMap<u32, Vec<Report>> {
        let mut result = HashMap::new();

        for reporter in self.reporters.values() {
            let (chan, port) = ipc::channel().unwrap();
            reporter.collect_reports(ReportsChan(chan));
            if let Ok(mut reports) = port.recv() {
                result
                    .entry(reports.pid)
                    .or_insert(vec![])
                    .append(&mut reports.reports);
            }
        }
        result
    }
}
