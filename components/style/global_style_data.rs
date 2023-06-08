/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Global style data

use crate::context::StyleSystemOptions;
#[cfg(feature = "gecko")]
use crate::gecko_bindings::bindings;
use crate::parallel::STYLE_THREAD_STACK_SIZE_KB;
use crate::shared_lock::SharedRwLock;
use crate::thread_state;
#[cfg(feature = "gecko")]
use gecko_profiler;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use rayon;
use std::{io, thread};

/// Global style data
pub struct GlobalStyleData {
    /// Shared RWLock for CSSOM objects
    pub shared_lock: SharedRwLock,

    /// Global style system options determined by env vars.
    pub options: StyleSystemOptions,
}

/// Global thread pool.
pub struct StyleThreadPool {
    /// How many threads parallel styling can use. If not using a thread pool, this is set to `None`.
    pub num_threads: Option<usize>,

    /// The parallel styling thread pool.
    ///
    /// For leak-checking purposes, we want to terminate the thread-pool, which
    /// waits for all the async jobs to complete. Thus the RwLock.
    style_thread_pool: RwLock<Option<rayon::ThreadPool>>,
}

fn thread_name(index: usize) -> String {
    format!("Style#{}", index)
}

lazy_static! {
    /// JoinHandles for spawned style threads. These will be joined during
    /// StyleThreadPool::shutdown() after exiting the thread pool.
    ///
    /// This would be quite inefficient if rayon destroyed and re-created
    /// threads regularly during threadpool operation in response to demand,
    /// however rayon actually never destroys its threads until the entire
    /// thread pool is shut-down, so the size of this list is bounded.
    static ref STYLE_THREAD_JOIN_HANDLES: Mutex<Vec<thread::JoinHandle<()>>> =
        Mutex::new(Vec::new());
}

fn thread_spawn(options: rayon::ThreadBuilder) -> io::Result<()> {
    let mut b = thread::Builder::new();
    if let Some(name) = options.name() {
        b = b.name(name.to_owned());
    }
    if let Some(stack_size) = options.stack_size() {
        b = b.stack_size(stack_size);
    }
    let join_handle = b.spawn(|| options.run())?;
    STYLE_THREAD_JOIN_HANDLES.lock().push(join_handle);
    Ok(())
}

fn thread_startup(_index: usize) {
    thread_state::initialize_layout_worker_thread();
    #[cfg(feature = "gecko")]
    unsafe {
        bindings::Gecko_SetJemallocThreadLocalArena(true);
        let name = thread_name(_index);
        gecko_profiler::register_thread(&name);
    }
}

fn thread_shutdown(_: usize) {
    #[cfg(feature = "gecko")]
    unsafe {
        gecko_profiler::unregister_thread();
        bindings::Gecko_SetJemallocThreadLocalArena(false);
    }
}

impl StyleThreadPool {
    /// Shuts down the thread pool, waiting for all work to complete.
    pub fn shutdown() {
        if STYLE_THREAD_JOIN_HANDLES.lock().is_empty() {
            return;
        }
        {
            // Drop the pool.
            let _ = STYLE_THREAD_POOL.lock().unwrap().style_thread_pool.write().take();
        }

        // Join spawned threads until all of the threads have been joined. This
        // will usually be pretty fast, as on shutdown there should be basically
        // no threads left running.
        while let Some(join_handle) = STYLE_THREAD_JOIN_HANDLES.lock().pop() {
            let _ = join_handle.join();
        }
    }

    /// Returns a reference to the thread pool.
    ///
    /// We only really want to give read-only access to the pool, except
    /// for shutdown().
    pub fn pool(&self) -> RwLockReadGuard<Option<rayon::ThreadPool>> {
        self.style_thread_pool.read()
    }
}

#[cfg(feature = "servo")]
fn stylo_threads_pref() -> i32 {
    use servo_config::pref;
    pref!(layout.threads) as i32
}

#[cfg(feature = "gecko")]
fn stylo_threads_pref() -> i32 {
    static_prefs::pref!("layout.css.stylo-threads")
}

/// The performance benefit of additional threads seems to level off at around six, so we cap it
/// there on many-core machines (see bug 1431285 comment 14).
pub(crate) const STYLO_MAX_THREADS: usize = 6;

lazy_static! {
    /// Global thread pool
    pub static ref STYLE_THREAD_POOL: std::sync::Mutex<StyleThreadPool> = {
        use std::cmp;
        // We always set this pref on startup, before layout or script have had a chance of
        // accessing (and thus creating) the thread-pool.
        let threads_pref: i32 = stylo_threads_pref();
        let num_threads = if threads_pref >= 0 {
            threads_pref as usize
        } else {
            use num_cpus;
            // The default heuristic is num_virtual_cores * .75. This gives us three threads on a
            // hyper-threaded dual core, and six threads on a hyper-threaded quad core.
            let threads = cmp::max(num_cpus::get() * 3 / 4, 1);
            // There's no point in creating a thread pool if there's one thread.
            if threads == 1 { 0 } else { threads }
        };

        let num_threads = cmp::min(num_threads, STYLO_MAX_THREADS);
        let (pool, num_threads) = if num_threads < 1 {
            (None, None)
        } else {
            let workers = rayon::ThreadPoolBuilder::new()
                .spawn_handler(thread_spawn)
                .num_threads(num_threads)
                .thread_name(thread_name)
                .start_handler(thread_startup)
                .exit_handler(thread_shutdown)
                .stack_size(STYLE_THREAD_STACK_SIZE_KB * 1024)
                .build();
            (workers.ok(), Some(num_threads))
        };

        std::sync::Mutex::new(StyleThreadPool {
            num_threads,
            style_thread_pool: RwLock::new(pool),
        })
    };

    /// Global style data
    pub static ref GLOBAL_STYLE_DATA: GlobalStyleData = GlobalStyleData {
        shared_lock: SharedRwLock::new_leaked(),
        options: StyleSystemOptions::default(),
    };
}
