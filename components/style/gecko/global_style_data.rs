/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Global style data

use context::StyleSystemOptions;
use gecko_bindings::bindings::{Gecko_RegisterProfilerThread, Gecko_UnregisterProfilerThread};
use num_cpus;
use rayon;
use shared_lock::SharedRwLock;
use std::cmp;
use std::env;
use std::ffi::CString;

/// Global style data
pub struct GlobalStyleData {
    /// How many threads parallel styling can use.
    pub num_threads: usize,

    /// The parallel styling thread pool.
    pub style_thread_pool: Option<rayon::ThreadPool>,

    /// Shared RWLock for CSSOM objects
    pub shared_lock: SharedRwLock,

    /// Global style system options determined by env vars.
    pub options: StyleSystemOptions,
}

fn thread_name(index: usize) -> String {
    format!("StyleThread#{}", index)
}

fn thread_startup(index: usize) {
    let name = thread_name(index);
    let name = CString::new(name).unwrap();
    unsafe {
        // Gecko_RegisterProfilerThread copies the passed name here.
        Gecko_RegisterProfilerThread(name.as_ptr());
    }
}

fn thread_shutdown(_: usize) {
    unsafe {
        Gecko_UnregisterProfilerThread();
    }
}

lazy_static! {
    /// Global style data
    pub static ref GLOBAL_STYLE_DATA: GlobalStyleData = {
        let stylo_threads = env::var("STYLO_THREADS")
            .map(|s| s.parse::<usize>().expect("invalid STYLO_THREADS value"));
        let num_threads = match stylo_threads {
            Ok(num) => num,
            _ => cmp::max(num_cpus::get() * 3 / 4, 1),
        };

        let pool = if num_threads <= 1 {
            None
        } else {
            let configuration = rayon::Configuration::new()
                .num_threads(num_threads)
                .thread_name(thread_name)
                .start_handler(thread_startup)
                .exit_handler(thread_shutdown);
            let pool = rayon::ThreadPool::new(configuration).ok();
            pool
        };

        GlobalStyleData {
            num_threads: num_threads,
            style_thread_pool: pool,
            shared_lock: SharedRwLock::new(),
            options: StyleSystemOptions::default(),
        }
    };
}
