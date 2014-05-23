/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::str::IntoMaybeOwned;
use std::task;
use std::comm::{channel, Sender};
use std::task::TaskOpts;

pub fn spawn_named<S: IntoMaybeOwned<'static>>(name: S, f: proc():Send) {
    let builder = task::task().named(name);
    builder.spawn(f);
}

/// Arrange to send a particular message to a channel if the task built by
/// this `TaskBuilder` fails.
pub fn send_on_failure<T: Send>(opts: &mut TaskOpts, msg: T, dest: Sender<T>) {
    assert!(opts.notify_chan.is_none());
    let (tx, rx) = channel();
    opts.notify_chan = Some(tx);
    let watched_name = opts.name.as_ref().unwrap().as_slice().to_owned();
    let name = format!("{:s}Watcher", watched_name);
    spawn_named(name, proc() {
        match rx.recv() {
            Ok(()) => (),
            Err(..) => {
                debug!("{:s} failed, notifying constellation", watched_name);
                dest.send(msg);
            }
        }
    })
}
