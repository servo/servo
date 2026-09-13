use std::collections::VecDeque;
use std::thread;

use profile_traits::mem::{ProcessReports, ProfilerChan as MemProfilerChan, Report};
use rustc_hash::FxHashMap;
use servo_base::generic_channel::{self, GenericCallback, GenericReceiver, GenericSender};
use servo_url::ImmutableOrigin;
use storage_traits::weblocks::{
    LockId, LockInfoMsg, LockManagerSnapshotMsg, LockModeMsg, LockMsg, LockRequest, LockRequestId,
    WebLocksThreadMsg,
};

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
                WebLocksThreadMsg::Release(lock_id, name, origin) => {
                    self.release_lock(origin, name, lock_id);
                },
                WebLocksThreadMsg::Abort(request_id, origin, name) => {
                    self.abort_request(origin, name, request_id);
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

    /// <https://www.w3.org/TR/web-locks/#algorithm-release-lock>
    fn release_lock(&mut self, origin: ImmutableOrigin, name: String, lock_id: LockId) {
        // Step 1. skip assert
        // Step 2-5. Let
        let manager = self.obtain_lock_manager(origin);

        // Step 6. Remove lock from the manager’s held lock set.
        manager.held.remove(&lock_id);

        // Step 7. Process the lock request queue queue.
        manager.process_lock_request_queue(&name);
    }

    /// <https://www.w3.org/TR/web-locks/#algorithm-abort-request>
    fn abort_request(&mut self, origin: ImmutableOrigin, name: String, request_id: LockRequestId) {
        // Step 1. skip assert
        // Step 2-5. Let
        let manager = self.obtain_lock_manager(origin);
        let queue = get_lock_request_queue!(manager, &name);

        // Step 6. Remove request from queue.
        if let Some(pos) = queue.iter().position(|request| request.id == request_id) {
            queue.remove(pos);
        }

        // Step 7. Process the lock request queue queue.
        manager.process_lock_request_queue(&name);
    }
}

/// Pages and workers sharing a storage bucket opened inthe same user agent
/// share a lock manager.
/// <https://www.w3.org/TR/web-locks/#lock-manager>
#[derive(Default)]
struct LockManager {
    queue_map: FxHashMap<String, VecDeque<LockRequest>>,
    held: FxHashMap<LockId, Lock>,
}

impl LockManager {
    /// <https://www.w3.org/TR/web-locks/#algorithm-request-lock>
    fn request_lock(&mut self, request: LockRequest) {
        // Step 3.1 to 3.3. skip to avoid &mut
        let name = request.name.clone();
        let queue = get_lock_request_queue!(self, &name);

        // Step 3.4. If steal is true,
        if request.steal {
            // Step 3.4.1.
            if let Some((_, lock)) = self.held.extract_if(|_, v| v.name == name).next() {
                _ = lock.released_callback.send(false);
            }
            // Step 3.4.2. Prepend request in queue.
            queue.push_front(request);
        }
        // Step 3.5. Otherwise
        else {
            let is_first = queue
                .front()
                .is_some_and(|first| std::ptr::eq(first, &request));

            // Step 3.5.1. if ifAvailable and not grantable, enqueue on callback event loop
            if request.if_available && is_grantable(&request, is_first, &self.held) {
                _ = request.held_callback.send(None);
            } else {
                // Step 3.5.2. Enqueue request in queue.
                queue.push_back(request);
            }
        }

        // Step 3.6. Process the lock request queue queue.
        self.process_lock_request_queue(&name);
    }

    /// <https://www.w3.org/TR/web-locks/#process-the-lock-request-queue>
    fn process_lock_request_queue(&mut self, name: &str) {
        let queue = get_lock_request_queue!(self, name);

        // Step 1. skip assert

        // Step 2. For each request of queue:
        while let Some(request) = queue.front() {
            // Step 2.1. If request is not grantable, return.
            if !is_grantable(request, true, &self.held) {
                return;
            }
            // Step 2.2. Remove request from queue.
            let request = queue.pop_front().unwrap();

            // Step 2.3 to 2.8. skip
            // TODO: especially waiting promises

            // Step 12. Let lock be a new lock with ...
            let lock_id = LockId::next();
            let lock = Lock {
                id: lock_id,
                name: request.name.clone(),
                mode: request.mode,
                client_id: request.client_id,
                released_callback: request.released_callback,
            };

            // Step 13. Append lock to manager’s held lock set.
            self.held.insert(lock_id, lock);

            // Step 14. Enqueue the following steps on callback’s relevant settings
            // object’s responsible event loop.
            // The inner steps continue on that thread.
            _ = request.held_callback.send(Some(LockMsg {
                id: lock_id,
                name: request.name,
                mode: request.mode,
            }));
        }
    }

    /// <https://www.w3.org/TR/web-locks/#snapshot-the-lock-state>
    fn snapshot_lock_state(&self) -> LockManagerSnapshotMsg {
        // Step 1. skip assert
        // Step 2. Let pending be a new list.
        let mut pending = vec![];
        // Step 3. For each queue of lock request queue map:
        for queue in self.queue_map.values() {
            // Step 3.1. For each request of queue:
            for request in queue.iter() {
                // Step 3.1.1. Append ...
                pending.push(LockInfoMsg {
                    name: request.name.clone(),
                    mode: request.mode,
                    client_id: request.client_id.clone(),
                });
            }
        }

        // Step 4. Let held be a new list.
        let mut held = vec![];
        // Step 5. For each lock of manager’s held lock set:
        for lock in self.held.values() {
            // Step 5.1. Append ...
            held.push(LockInfoMsg {
                name: lock.name.clone(),
                mode: lock.mode,
                client_id: lock.client_id.clone(),
            });
        }

        // Step 6. Resolve promise with held and pending.
        LockManagerSnapshotMsg { held, pending }
    }
}

/// <https://www.w3.org/TR/web-locks/#grantable>
fn is_grantable(request: &LockRequest, is_first: bool, held: &FxHashMap<LockId, Lock>) -> bool {
    // Step 1 to 6 skip
    // Step 7.
    if !is_first {
        return false;
    }
    match request.mode {
        // Step 8.
        LockModeMsg::Exclusive => !held.values().any(|lock| lock.name == request.name),
        // Step 9.
        LockModeMsg::Shared => !held
            .values()
            .any(|lock| lock.mode == LockModeMsg::Exclusive && lock.name == request.name),
    }
}

struct Lock {
    id: LockId,
    name: String,
    mode: LockModeMsg,
    client_id: String,
    // TODO: waiting promise => receiver? or msg? Msg::WaitingDone or similar is better.
    released_callback: GenericCallback<bool>,
}
