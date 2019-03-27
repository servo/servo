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
use rayon;
use std::env;

/// Global style data
pub struct GlobalStyleData {
    /// Shared RWLock for CSSOM objects
    pub shared_lock: SharedRwLock,

    /// Global style system options determined by env vars.
    pub options: StyleSystemOptions,
}

/// Global thread pool
pub struct StyleThreadPool {
    /// How many threads parallel styling can use.
    pub num_threads: usize,

    /// The parallel styling thread pool.
    pub style_thread_pool: Option<rayon::ThreadPool>,
}

fn thread_name(index: usize) -> String {
    format!("StyleThread#{}", index)
}

fn thread_startup(_index: usize) {
    thread_state::initialize_layout_worker_thread();
    #[cfg(feature = "gecko")]
    unsafe {
        use std::ffi::CString;

        bindings::Gecko_SetJemallocThreadLocalArena(true);
        let name = thread_name(_index);
        let name = CString::new(name).unwrap();
        // Gecko_RegisterProfilerThread copies the passed name here.
        bindings::Gecko_RegisterProfilerThread(name.as_ptr());
    }
}

fn thread_shutdown(_: usize) {
    #[cfg(feature = "gecko")]
    unsafe {
        bindings::Gecko_UnregisterProfilerThread();
        bindings::Gecko_SetJemallocThreadLocalArena(false);
    }
}

lazy_static! {
    /// Global thread pool
    pub static ref STYLE_THREAD_POOL: StyleThreadPool = {
        let stylo_threads = env::var("STYLO_THREADS")
            .map(|s| s.parse::<usize>().expect("invalid STYLO_THREADS value"));
        let mut num_threads = match stylo_threads {
            Ok(num) => num,
            #[cfg(feature = "servo")]
            _ => {
                use servo_config::pref;
                // We always set this pref on startup, before layout or script
                // have had a chance of accessing (and thus creating) the
                // thread-pool.
                pref!(layout.threads) as usize
            }
            #[cfg(feature = "gecko")]
            _ => {
                // The default heuristic is num_virtual_cores * .75. This gives
                // us three threads on a hyper-threaded dual core, and six
                // threads on a hyper-threaded quad core. The performance
                // benefit of additional threads seems to level off at around
                // six, so we cap it there on many-core machines
                // (see bug 1431285 comment 14).
                use num_cpus;
                use std::cmp;
                cmp::min(cmp::max(num_cpus::get() * 3 / 4, 1), 6)
            }
        };

        // If num_threads is one, there's no point in creating a thread pool, so
        // force it to zero.
        //
        // We allow developers to force a one-thread pool for testing via a
        // special environmental variable.
        if num_threads == 1 {
            let force_pool = env::var("FORCE_STYLO_THREAD_POOL")
                .ok().map_or(false, |s| s.parse::<usize>().expect("invalid FORCE_STYLO_THREAD_POOL value") == 1);
            if !force_pool {
                num_threads = 0;
            }
        }

        let pool = if num_threads < 1 {
            None
        } else {
            let workers = rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                // Enable a breadth-first rayon traversal. This causes the work
                // queue to be always FIFO, rather than FIFO for stealers and
                // FILO for the owner (which is what rayon does by default). This
                // ensures that we process all the elements at a given depth before
                // proceeding to the next depth, which is important for style sharing.
                .breadth_first()
                .thread_name(thread_name)
                .start_handler(thread_startup)
                .exit_handler(thread_shutdown)
                .stack_size(STYLE_THREAD_STACK_SIZE_KB * 1024)
                .build();
            workers.ok()
        };

        StyleThreadPool {
            num_threads: num_threads,
            style_thread_pool: pool,
        }
    };
    /// Global style data
    pub static ref GLOBAL_STYLE_DATA: GlobalStyleData = GlobalStyleData {
        shared_lock: SharedRwLock::new(),
        options: StyleSystemOptions::default(),
    };
}
