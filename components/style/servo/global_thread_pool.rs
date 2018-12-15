//! Global thread pool for parsing stylesheets when async parsing is enabled.
//! Currently each stylesheet is parsed on a different thread in the pool, but
//! otherwise per-sheet parsing is sequential.

use num_cpus;
use rayon;
use std::cmp;
use std::env;

/// Global thread pool for parsing stylesheets.
pub struct StyleThreadPool {
    /// How many threads parallel styling can use.
    pub num_threads: usize,

    /// The parallel styling thread pool.
    pub thread_pool: Option<rayon::ThreadPool>,
}

fn thread_name(index: usize) -> String {
    format!("StyleThread#{}", index)
}

lazy_static! {
    /// The global style thread pool instance.
    pub static ref STYLE_THREAD_POOL: StyleThreadPool = {
        let style_threads = env::var("STYLO_THREADS")
            .map(|s| s.parse::<usize>().expect("invalid STYLO_THREADS value"));
        let mut num_threads = match style_threads {
            Ok(num) => num,
            // The default heuristic is num_virtual_cores * .75. This gives us
            // three threads on a hyper-threaded dual core, and six threads on
            // a hyper-threaded quad core. The performance benefit of additional
            // threads seems to level off at around six, so we cap it there on
            // many-core machines (see bug 1431285 comment 14).
            _ => cmp::min(cmp::max(num_cpus::get() * 3 / 4, 1), 6),
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
            rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                // Enable a breadth-first rayon traversal. This causes the work
                // queue to be always FIFO, rather than FIFO for stealers and
                // FILO for the owner (which is what rayon does by default). This
                // ensures that we process all the elements at a given depth before
                // proceeding to the next depth, which is important for style sharing.
                .breadth_first()
                .thread_name(thread_name)
                //.stack_size(STYLE_THREAD_STACK_SIZE_KB * 1024)
                .build()
                .ok()
        };

        StyleThreadPool {
            num_threads: num_threads,
            thread_pool: pool,
        }
    };
}
