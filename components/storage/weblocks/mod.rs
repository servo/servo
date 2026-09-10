use std::collections::VecDeque;
use std::thread;

use profile_traits::mem::{ProcessReports, ProfilerChan as MemProfilerChan, Report};
use rustc_hash::FxHashMap;
use servo_base::generic_channel::{self, GenericReceiver, GenericSender};
use servo_url::ImmutableOrigin;
use storage_traits::weblocks::{
    LockInfoMsg, LockManagerSnapshotMsg, LockModeMsg, LockMsg, LockRequest, WebLocksThreadMsg,
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
    /// Per-origin managers,
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
                    self.obtain_lock_manager(origin).request_lock(request);
                },
                WebLocksThreadMsg::Query(sender, origin) => {
                    let snapshot = self.obtain_lock_manager(origin).snapshot_lock_state();
                    _ = sender.send(snapshot);
                },
                WebLocksThreadMsg::CollectMemoryReport(sender) => {
                    let reports = self.collect_memory_reports();
                    sender.send(ProcessReports::new(reports));
                },
                WebLocksThreadMsg::Release(name, origin) => {
                    // TODO: release lock
                },
                WebLocksThreadMsg::Abort(request_id) => {
                    // TODO: abort lock
                },
            }
        }
    }

    /// <https://www.w3.org/TR/web-locks/#obtain-a-lock-manager>
    fn obtain_lock_manager(&mut self, origin: ImmutableOrigin) -> &mut LockManager {
        // TODO: servo does not follow storage standard
        self.managers.entry(origin).or_default()
    }

    fn collect_memory_reports(&self) -> Vec<Report> {
        // TODO: actual report
        vec![]
    }
}

/// Pages and workers sharing a storage bucket opened inthe same user agent
/// share a lock manager.
/// <https://www.w3.org/TR/web-locks/#lock-manager>
#[derive(Default)]
struct LockManager {
    queue_map: FxHashMap<String, VecDeque<LockRequest>>,
    held: FxHashMap<String, Lock>,
}

/// View type is not stable, use macro to avoid &mut self.
/// <https://www.w3.org/TR/web-locks/#get-the-lock-request-queue>
macro_rules! get_lock_request_queue {
    ($self:expr, $name:expr) => {{
        // Step 1.
        if !$self.queue_map.contains_key($name) {
            $self
                .queue_map
                .insert($name.to_string(), Default::default());
        }
        // Step 2.
        $self.queue_map.get_mut($name).unwrap()
    }};
}

impl LockManager {
    /// <https://www.w3.org/TR/web-locks/#algorithm-request-lock>
    fn request_lock(&mut self, request: LockRequest) {
        // Step 3.1 to 3.3. skip to avoid &mut
        let name = request.name.clone();
        let queue = get_lock_request_queue!(self, &name);

        // Step 3.4.
        if request.steal {
            if let Some(_todo) = self.held.remove(&name) {
                // TODO: reject original lock released promise
            }
            queue.push_front(request);
        }
        // Step 3.5.
        else {
            let is_first = queue
                .front()
                .is_some_and(|first| std::ptr::eq(first, &request));
            if request.if_available && is_grantable(&request, is_first, &self.held) {
                _ = request.callback.send(None);
                // TODO: resolve promise and abort
            }
            queue.push_back(request);
        }

        // Step 3.6.
        self.process_lock_request_queue(&name);
    }

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
        for lock in self.held.values() {
            held.push(LockInfoMsg {
                name: lock.name.clone(),
                mode: lock.mode,
                client_id: lock.client_id.clone(),
            });
        }

        // Step 6.
        LockManagerSnapshotMsg { held, pending }
    }

    /// <https://www.w3.org/TR/web-locks/#process-the-lock-request-queue>
    fn process_lock_request_queue(&mut self, name: &str) {
        // Step 1. skip assert
        let queue = get_lock_request_queue!(self, name);
        // Step 2.
        while let Some(request) = queue.front() {
            // Step 2.1.
            if !is_grantable(request, true, &self.held) {
                return;
            }
            // Step 2.2.
            let request = queue.pop_front().unwrap();
            // Step 2.3 to 2.8. skip
            // TODO: especially waiting promises

            // Step 12.
            let lock = Lock {
                name: request.name.clone(),
                mode: request.mode,
                client_id: request.client_id,
            };
            // Step 13.
            self.held.insert(lock.name.clone(), lock);
            // Step 14.
            _ = request.callback.send(Some(LockMsg {
                name: request.name,
                mode: request.mode,
            }));
        }
    }
}

/// <https://www.w3.org/TR/web-locks/#grantable>
fn is_grantable(request: &LockRequest, is_first: bool, held: &FxHashMap<String, Lock>) -> bool {
    // Step 1 to 6 skip
    // Step 7.
    if !is_first {
        return false;
    }
    match request.mode {
        // Step 8.
        LockModeMsg::Exclusive => !held.contains_key(&request.name),
        // Step 9.
        LockModeMsg::Shared => !held
            .values()
            .any(|lock| lock.mode == LockModeMsg::Exclusive && lock.name == request.name),
    }
}

struct Lock {
    name: String,
    mode: LockModeMsg,
    client_id: String,
}
