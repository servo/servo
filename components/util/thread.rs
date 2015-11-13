/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::Builder;
use thread_state;

pub fn spawn_named<F>(name: String, f: F)
    where F: FnOnce() + Send + 'static
{
    let builder = thread::Builder::new().name(name);
    builder.spawn(f).unwrap();
}

/// Arrange to send a particular message to a channel if the thread fails.
pub fn spawn_named_with_send_on_failure<F, T>(name: String,
                                              state: thread_state::ThreadState,
                                              f: F,
                                              msg: T,
                                              dest: Sender<T>)
    where F: FnOnce() + Send + 'static,
          T: Send + 'static
{
    let future_handle = thread::Builder::new().name(name.to_owned()).spawn(move || {
        thread_state::initialize(state);
        f()
    }).unwrap();

    let watcher_name = format!("{}Watcher", name);
    Builder::new().name(watcher_name).spawn(move || {
        match future_handle.join() {
            Ok(()) => (),
            Err(..) => {
                debug!("{} failed, notifying constellation", name);
                dest.send(msg).unwrap();
            }
        }
    }).unwrap();
}
