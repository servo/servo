use std::{collections::VecDeque, thread};

use profile_traits::mem::{ProcessReports, ProfilerChan as MemProfilerChan, Report};
use rustc_hash::{FxHashMap, FxHashSet};
use servo_base::generic_channel::{self, GenericReceiver, GenericSender};
use servo_url::ImmutableOrigin;
use storage_traits::weblocks::{
    LockInfoMsg, LockManagerSnapshotMsg, LockModeMsg, WebLocksThreadMsg,
};

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

struct WebLocksManager {
    port: GenericReceiver<WebLocksThreadMsg>,
    managers: FxHashMap<ImmutableOrigin, LockManager>,
}

impl WebLocksManager {
    fn new(port: GenericReceiver<WebLocksThreadMsg>) -> Self {
        Self {
            port,
            managers: Default::default(),
        }
    }

    fn start(&mut self) {
        loop {
            match self.port.recv().unwrap() {
                WebLocksThreadMsg::Request(request, origin) => {
                    // TODO: reply request
                },
                WebLocksThreadMsg::Query(sender, origin) => {
                    let snapshot = self
                        .managers
                        .get(&origin)
                        .map(|manager| manager.snapshot_lock_state())
                        .unwrap_or_default();
                    _ = sender.send(snapshot);
                },
                WebLocksThreadMsg::CollectMemoryReport(sender) => {
                    let reports = self.collect_memory_reports();
                    sender.send(ProcessReports::new(reports));
                },
                WebLocksThreadMsg::Release() => {
                    // TODO: release lock
                },
                WebLocksThreadMsg::Abort() => {
                    // TODO: abort lock
                },
            }
        }
    }

    fn collect_memory_reports(&self) -> Vec<Report> {
        // TODO: actual report
        vec![]
    }
}

/// Pages and workers sharing a storage bucket opened inthe same user agent
/// share a lock manager.
/// <https://www.w3.org/TR/web-locks/#lock-manager>
struct LockManager {
    queue_map: FxHashMap<String, VecDeque<LockRequest>>,
    held: FxHashSet<Lock>,
}

impl LockManager {
    /// <https://www.w3.org/TR/web-locks/#snapshot-the-lock-state>
    fn snapshot_lock_state(&self) -> LockManagerSnapshotMsg {
        // Step 1. skip assert
        // Step 2.
        let mut pending = vec![];
        // Step 3.
        for queue in self.queue_map.values() {
            for request in queue.iter() {
                pending.push(LockInfoMsg {
                    name: request.name.clone(),
                    mode: request.mode,
                    client_id: request.client_id.clone(),
                });
            }
        }

        // Step 4.
        let mut held = vec![];
        // Step 5.
        for lock in self.held.iter() {
            held.push(LockInfoMsg {
                name: lock.name.clone(),
                mode: lock.mode,
                client_id: lock.client_id.clone(),
            });
        }

        // Step 6.
        LockManagerSnapshotMsg { held, pending }
    }
}

struct LockRequest {
    pub name: String,
    pub mode: LockModeMsg,
    pub client_id: String,
}

struct Lock {
    name: String,
    mode: LockModeMsg,
    client_id: String,
}
