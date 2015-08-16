/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A load-balancing task pool.
//!
//! This differs in implementation from std::sync::TaskPool in that each job is
//! up for grabs by any of the child tasks in the pool.
//!

//
// This is based on the cargo task pool.
// https://github.com/rust-lang/cargo/blob/master/src/cargo/util/pool.rs
//
// The only difference is that a normal channel is used instead of a sync_channel.
//

use task::spawn_named;
use std::boxed::FnBox;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};

pub struct TaskPool {
    tx: Sender<Box<FnBox() + Send + 'static>>,
}

impl TaskPool {
    pub fn new(tasks: u32) -> TaskPool {
        assert!(tasks > 0);
        let (tx, rx) = channel();

        let state = Arc::new(Mutex::new(rx));

        for i in 0..tasks {
            let state = state.clone();
            spawn_named(
                format!("TaskPoolWorker {}/{}", i + 1, tasks),
                move || worker(&*state));
        }

        return TaskPool { tx: tx };

        fn worker(rx: &Mutex<Receiver<Box<FnBox() + Send + 'static>>>) {
            loop {
                let job = rx.lock().unwrap().recv();
                match job {
                    Ok(job) => job.call_box(()),
                    Err(..) => break,
                }
            }
        }
    }

    pub fn execute<F>(&self, job: F)
        where F: FnOnce() + Send + 'static
    {
        self.tx.send(Box::new(job)).unwrap();
    }
}
