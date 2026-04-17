/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use log::{debug, error};
use servo_config::{opts, pref};

/// The state of the thread-pool used by CoreResource.
struct ThreadPoolState {
    /// The number of active workers.
    active_workers: u32,
    /// Whether the pool can spawn additional work.
    active: bool,
}

impl ThreadPoolState {
    pub fn new() -> ThreadPoolState {
        ThreadPoolState {
            active_workers: 0,
            active: true,
        }
    }

    /// Is the pool still able to spawn new work?
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// How many workers are currently active?
    pub fn active_workers(&self) -> u32 {
        self.active_workers
    }

    /// Prevent additional work from being spawned.
    pub fn switch_to_inactive(&mut self) {
        self.active = false;
    }

    /// Add to the count of active workers.
    pub fn increment_active(&mut self) {
        self.active_workers += 1;
    }

    /// Subtract from the count of active workers.
    pub fn decrement_active(&mut self) {
        self.active_workers -= 1;
    }
}

/// The type of the ThreadPool to spawn.
pub enum ThreadPoolType {
    ImageCache,
    IndexedDB,
    WebStorage,
    Test,
}

/// Threadpools used throughout servo (except layout/stylo and webrender).
pub struct ThreadPool {
    pool: rayon::ThreadPool,
    state: Arc<Mutex<ThreadPoolState>>,
}

/// Get number of threads and names for [`ThreadPoolType`].
fn get_pool_create_information(t: &ThreadPoolType) -> (usize, String) {
    let max_thread_count = thread::available_parallelism()
        .map(|i| i.get())
        .unwrap_or(pref!(threadpools_fallback_worker_num) as usize);

    match t {
        ThreadPoolType::ImageCache => (
            max_thread_count.min(pref!(threadpools_image_cache_workers_max).max(1) as usize),
            "ImageCache".to_owned(),
        ),
        ThreadPoolType::IndexedDB => (
            max_thread_count.min(pref!(threadpools_indexeddb_workers_max).max(1) as usize),
            "IndexedDB".to_owned(),
        ),
        ThreadPoolType::WebStorage => (
            max_thread_count.min(pref!(threadpools_webstorage_workers_max).max(1) as usize),
            "WebStorage".to_owned(),
        ),
        ThreadPoolType::Test => (1, "TestPool".to_owned()),
    }
}

static GLOBAL_THREADPOOL: OnceLock<Arc<ThreadPool>> = OnceLock::new();

impl ThreadPool {
    /// Create a new ThreadPool from type.
    pub fn new(t: ThreadPoolType) -> Arc<Self> {
        if pref!(threadpools_single_pool) && !opts::get().multiprocess {
            let pool = GLOBAL_THREADPOOL.get_or_init(|| {
                let paralellism = thread::available_parallelism()
                    .map(|i| i.get())
                    .unwrap_or(pref!(threadpools_fallback_worker_num) as usize)
                    .min(pref!(threadpools_single_pool_max) as usize);
                let pool = rayon::ThreadPoolBuilder::new()
                    .thread_name(move |i| format!("GlobalPool#{i}"))
                    .num_threads(paralellism)
                    .build()
                    .unwrap();
                Arc::new(Self {
                    pool,
                    state: Arc::new(Mutex::new(ThreadPoolState::new())),
                })
            });
            pool.clone()
        } else {
            if opts::get().multiprocess {
                error!(
                    "Single threadpool and multiprocess are incompatible. Falling back to separate threadpools"
                );
            }
            let (num_threads, pool_name) = get_pool_create_information(&t);
            debug!("Creating new ThreadPool with {num_threads} threads!");
            let pool = rayon::ThreadPoolBuilder::new()
                .thread_name(move |i| format!("{pool_name}#{i}"))
                .num_threads(num_threads)
                .build()
                .unwrap();
            let state = Arc::new(Mutex::new(ThreadPoolState::new()));
            Arc::new(Self { pool, state })
        }
    }

    /// Spawn work on the thread-pool, if still active.
    ///
    /// There is no need to give feedback to the caller,
    /// because if we do not perform work,
    /// it is because the system as a whole is exiting.
    pub fn spawn<OP>(&self, work: OP)
    where
        OP: FnOnce() + Send + 'static,
    {
        {
            let mut state = self.state.lock().unwrap();
            if state.is_active() {
                state.increment_active();
            } else {
                // Don't spawn any work.
                return;
            }
        }

        let state = self.state.clone();

        self.pool.spawn(move || {
            {
                let mut state = state.lock().unwrap();
                if !state.is_active() {
                    // Decrement number of active workers and return,
                    // without doing any work.
                    return state.decrement_active();
                }
            }
            // Perform work.
            work();
            {
                // Decrement number of active workers.
                let mut state = state.lock().unwrap();
                state.decrement_active();
            }
        });
    }

    /// Prevent further work from being spawned,
    /// and wait until all workers are done,
    /// or a timeout of roughly one second has been reached.
    pub fn exit(&self) {
        {
            let mut state = self.state.lock().unwrap();
            state.switch_to_inactive();
        }
        let mut rounds = 0;
        loop {
            rounds += 1;
            {
                let state = self.state.lock().unwrap();
                let still_active = state.active_workers();

                if still_active == 0 || rounds == 10 {
                    if still_active > 0 {
                        debug!(
                            "Exiting ThreadPool with {:?} still working(should be zero)",
                            still_active
                        );
                    }
                    break;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}
