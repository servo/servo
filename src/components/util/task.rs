/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
pub fn spawn_named_with_send_on_failure<T: Send>(name: &str,
                                                 f: proc(): Send,
                                                 msg: T,
                                                 dest: Sender<T>) {
    let name = name.to_string();
    let future_result = TaskBuilder::new().named(name.clone()).try_future(f);

    let watch_name = format!("{:s}Watcher", name);
    spawn_named(watch_name, proc() {
        match future_result.unwrap() {
            Ok(()) => (),
            Err(..) => {
                debug!("{:s} failed, notifying constellation", name);
                dest.send(msg);
            }
        }
    });
}
