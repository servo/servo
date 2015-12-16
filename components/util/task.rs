/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use serde::Serialize;
use std::borrow::ToOwned;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::Builder;
use task_state;

pub fn spawn_named<F>(name: String, f: F)
    where F: FnOnce() + Send + 'static
{
    let builder = thread::Builder::new().name(name);
    builder.spawn(f).unwrap();
}

/// An abstraction over `Sender<T>` and `IpcSender<T>`, for use in
/// `spawn_named_with_send_on_failure`.
pub trait SendOnFailure {
    type Value;
    fn send_on_failure(&mut self, value: Self::Value);
}

impl<T> SendOnFailure for Sender<T> where T: Send + 'static {
    type Value = T;
    fn send_on_failure(&mut self, value: T) {
        self.send(value).unwrap();
    }
}

impl<T> SendOnFailure for IpcSender<T> where T: Send + Serialize + 'static {
    type Value = T;
    fn send_on_failure(&mut self, value: T) {
        self.send(value).unwrap();
    }
}

/// Arrange to send a particular message to a channel if the task fails.
pub fn spawn_named_with_send_on_failure<F, T, S>(name: String,
                                                 state: task_state::TaskState,
                                                 f: F,
                                                 msg: T,
                                                 mut dest: S)
                                                 where F: FnOnce() + Send + 'static,
                                                       T: Send + 'static,
                                                       S: Send + SendOnFailure<Value=T> + 'static {
    let future_handle = thread::Builder::new().name(name.to_owned()).spawn(move || {
        task_state::initialize(state);
        f()
    }).unwrap();

    let watcher_name = format!("{}Watcher", name);
    Builder::new().name(watcher_name).spawn(move || {
        match future_handle.join() {
            Ok(()) => (),
            Err(..) => {
                debug!("{} failed, notifying constellation", name);
                dest.send_on_failure(msg);
            }
        }
    }).unwrap();
}

