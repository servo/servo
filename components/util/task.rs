/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::task;
use std::comm::Sender;
use std::task::TaskBuilder;
// use rtinstrument;
use task_state;

pub fn spawn_named<S: IntoCow<'static, String, str>>(name: S, f: proc():Send) {
    let builder = task::TaskBuilder::new().named(name);
    builder.spawn(proc() {
        // rtinstrument::instrument(f);
        f();
    });
}

/// Arrange to send a particular message to a channel if the task fails.
pub fn spawn_named_with_send_on_failure<T: Send>(name: &'static str,
                                                 state: task_state::TaskState,
                                                 f: proc(): Send,
                                                 msg: T,
                                                 dest: Sender<T>) {
    let future_result = TaskBuilder::new().named(name).try_future(proc() {
        task_state::initialize(state);
        // FIXME: Find replacement for this post-runtime removal
        // rtinstrument::instrument(f);
        f();
    });

    let watched_name = name.to_owned();
    let watcher_name = format!("{}Watcher", watched_name);
    TaskBuilder::new().named(watcher_name).spawn(proc() {
        //rtinstrument::instrument(proc() {
            match future_result.into_inner() {
                Ok(()) => (),
                Err(..) => {
                    debug!("{} failed, notifying constellation", name);
                    dest.send(msg);
                }
            }
        //});
    });
}
