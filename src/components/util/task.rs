/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use native;
use rustrt::task::{TaskOpts, Result};
use std::str::IntoMaybeOwned;
use std::task;
use std::comm::Sender;
use std::task::TaskBuilder;

pub fn spawn_named<S: IntoMaybeOwned<'static>>(name: S, f: proc():Send) {
    let builder = task::TaskBuilder::new().named(name);
    builder.spawn(f);
}

/// Arrange to send a particular message to a channel if the task built by
/// this `TaskBuilder` fails.
pub fn send_on_failure<T: Send>(builder: &mut TaskBuilder, msg: T, dest: Sender<T>) {
    let port = builder.future_result();
    let watched_name = builder.opts.name.as_ref().unwrap().as_slice().to_string();
    let name = format!("{:s}Watcher", watched_name);
    spawn_named(name, proc() {
        match port.recv() {
            Ok(()) => (),
            Err(..) => {
                debug!("{:s} failed, notifying constellation", watched_name);
                dest.send(msg);
            }
        }
    })
}

pub fn send_on_failure_native<T: Send>(name: &str, msg: T, dest: Sender<T>, f: proc():Send) {
    let mut task_opts = TaskOpts::new();
    task_opts.name = Some(name.to_string().into_maybe_owned());

    let watched_name = name.to_string();
    task_opts.on_exit = Some(proc(result: Result) {
        match result {
            Ok(()) => (),
            Err(..) => {
                debug!("{:s} failed, notifying constellation", watched_name);
                dest.send(msg);
            }
        }
    });
    native::task::spawn_opts(task_opts, f);
}
