use std::thread;

use profile_traits::mem::{ProcessReports, ProfilerChan as MemProfilerChan, Report};
use rustc_hash::FxHashMap;
use servo_base::generic_channel::{self, GenericReceiver, GenericSender};
use servo_url::ImmutableOrigin;
use storage_traits::weblocks::{LockManagerSnapshotMsg, WebLocksThreadMsg};

pub trait WebLocksThreadFactory {
    fn new(mem_profiler_chan: MemProfilerChan, reporter_name: String) -> Self;
}

impl WebLocksThreadFactory for GenericSender<WebLocksThreadMsg> {
    fn new(mem_profiler_chan: MemProfilerChan, reporter_name: String) -> Self {
        let (chan, port) = generic_channel::channel().unwrap();
        let chan2 = chan.clone();
        thread::Builder::new()
            .name("WebLocksManager".to_owned())
            .spawn(move || {
                mem_profiler_chan.run_with_memory_reporting(
                    || WebLocksManager::new(port).start(),
                    reporter_name,
                    chan2,
                    WebLocksThreadMsg::CollectMemoryReport,
                );
            })
            .expect("thread spawning failed");
        chan
    }
}

struct OriginEntry {}

struct WebLocksManager {
    port: GenericReceiver<WebLocksThreadMsg>,
    locks: FxHashMap<ImmutableOrigin, OriginEntry>,
}

impl WebLocksManager {
    fn new(port: GenericReceiver<WebLocksThreadMsg>) -> Self {
        Self {
            port,
            locks: Default::default(),
        }
    }

    fn start(&mut self) {
        loop {
            match self.port.recv().unwrap() {
                WebLocksThreadMsg::Request(sender, name) => {
                    // TODO: reply request
                },
                WebLocksThreadMsg::Query(sender) => {
                    // TODO: actual fill message
                    let msg = LockManagerSnapshotMsg {
                        held: vec![],
                        pending: vec![],
                    };
                    _ = sender.send(msg);
                },
                WebLocksThreadMsg::CollectMemoryReport(sender) => {
                    let reports = self.collect_memory_reports();
                    sender.send(ProcessReports::new(reports));
                },
            }
        }
    }

    fn collect_memory_reports(&self) -> Vec<Report> {
        // TODO: actual report
        vec![]
    }
}
