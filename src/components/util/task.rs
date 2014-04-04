/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::str::IntoMaybeOwned;
use std::task;
use std::comm::Sender;
use std::task::TaskBuilder;

pub fn spawn_named<S: IntoMaybeOwned<'static>>(name: S, f: proc()) {
    let builder = task::task().named(name);
    builder.spawn(f);
}

/// Arrange to send a particular message to a channel if the task built by
/// this `TaskBuilder` fails.
pub fn send_on_failure<T: Send>(builder: &mut TaskBuilder, msg: T, dest: Sender<T>) {
    let port = builder.future_result();
    spawn(proc() {
        match port.recv() {
            Ok(()) => (),
            Err(..) => dest.send(msg),
        }
    })
}
