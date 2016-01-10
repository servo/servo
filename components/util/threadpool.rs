/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A load-balancing thread pool.
//!
//! This differs in implementation from std::sync::ThreadPool in that each job is
//! up for grabs by any of the child threads in the pool.
//!

//
// This is based on the cargo thread pool.
// https://github.com/rust-lang/cargo/blob/master/src/cargo/util/pool.rs
//
// The only difference is that a normal channel is used instead of a sync_channel.
//

use std::boxed::FnBox;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use thread::spawn_named;

pub struct ThreadPool {
    tx: Sender<Box<FnBox() + Send + 'static>>,
}

impl ThreadPool {
    pub fn new(threads: u32) -> ThreadPool {
        assert!(threads > 0);
        let (tx, rx) = channel();

        let state = Arc::new(Mutex::new(rx));

        for i in 0..threads {
            let state = state.clone();
            spawn_named(
                format!("ThreadPoolWorker {}/{}", i + 1, threads),
                move || worker(&*state));
        }

        return ThreadPool { tx: tx };

        fn worker(rx: &Mutex<Receiver<Box<FnBox() + Send + 'static>>>) {
            while let Ok(job) = rx.lock().unwrap().recv() {
                job.call_box(());
            }
        }
    }

    pub fn execute<F>(&self, job: F)
        where F: FnOnce() + Send + 'static
    {
        self.tx.send(Box::new(job)).unwrap();
    }
}
