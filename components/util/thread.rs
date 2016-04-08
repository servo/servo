/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use opts;
use serde::Serialize;
use std::borrow::ToOwned;
use std::io::{Write, stderr};
use std::panic::{PanicInfo, take_hook, set_hook};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::Builder;
use thread_state;

pub fn spawn_named<F>(name: String, f: F)
    where F: FnOnce() + Send + 'static
{
    let builder = thread::Builder::new().name(name);

    if opts::get().full_backtraces {
        builder.spawn(f).unwrap();
        return;
    }

    let f_with_hook = move || {
        let hook = take_hook();

        let new_hook = move |info: &PanicInfo| {
            let payload = info.payload();
            if let Some(s) = payload.downcast_ref::<String>() {
                if s.contains("SendError") {
                    let err = stderr();
                    let _ = write!(err.lock(), "Thread \"{}\" panicked with an unwrap of \
                                                `SendError` (backtrace skipped)\n",
                           thread::current().name().unwrap_or("<unknown thread>"));
                    return;
                } else if s.contains("RecvError")  {
                    let err = stderr();
                    let _ = write!(err.lock(), "Thread \"{}\" panicked with an unwrap of \
                                                `RecvError` (backtrace skipped)\n",
                           thread::current().name().unwrap_or("<unknown thread>"));
                    return;
                }
            }
            hook(&info);
        };
        set_hook(Box::new(new_hook));
        f();
    };

    builder.spawn(f_with_hook).unwrap();
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

/// Arrange to send a particular message to a channel if the thread fails.
pub fn spawn_named_with_send_on_failure<F, T, S>(name: String,
                                                 state: thread_state::ThreadState,
                                                 f: F,
                                                 msg: T,
                                                 mut dest: S)
                                                 where F: FnOnce() + Send + 'static,
                                                       T: Send + 'static,
                                                       S: Send + SendOnFailure<Value=T> + 'static {
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
                dest.send_on_failure(msg);
            }
        }
    }).unwrap();
}
