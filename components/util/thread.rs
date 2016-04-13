/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use opts;
use serde::Serialize;
use std::io::{Write, stderr};
use std::panic::{PanicInfo, take_hook, set_hook};
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::thread;
use thread_state;

pub fn spawn_named<F>(name: String, f: F)
    where F: FnOnce() + Send + 'static
{
    spawn_named_with_send_on_failure_maybe(name, None, f, (), ());
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

impl SendOnFailure for () {
    type Value = ();
    fn send_on_failure(&mut self, _: ()) {}
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
                                                 dest: S)
                                                 where F: FnOnce() + Send + 'static,
                                                       T: Send + 'static,
                                                       S: Send + SendOnFailure<Value=T> + 'static {

    spawn_named_with_send_on_failure_maybe(name, Some(state), f, msg, dest);
}

fn spawn_named_with_send_on_failure_maybe<F, T, S>(name: String,
                                                 state: Option<thread_state::ThreadState>,
                                                 f: F,
                                                 msg: T,
                                                 dest: S)
                                                 where F: FnOnce() + Send + 'static,
                                                       T: Send + 'static,
                                                       S: Send + SendOnFailure<Value=T> + 'static {


    let builder = thread::Builder::new().name(name.clone());

    // store it locally, we can't trust that opts::get() will work whilst panicking
    let full_backtraces = opts::get().full_backtraces;

    // The panic handler can be called multiple times
    // We only send the message the first time.
    let dest = Mutex::new(dest);
    let msg = Mutex::new(Some(msg));

    let f_with_hook = move || {
        if let Some(state) = state {
            thread_state::initialize(state);
        }
        let hook = take_hook();

        let new_hook = move |info: &PanicInfo| {
            let payload = info.payload();

            // if the caller asked us to notify a thread, do so
            if let (Ok(mut dest), Ok(mut msg)) = (dest.try_lock(), msg.try_lock()) {
                if let Some(msg) = msg.take() {
                    debug!("Thread `{}` failed, notifying error handlers", name.clone());
                    dest.send_on_failure(msg);
                }
            }
            if !full_backtraces {
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
            }

            // the old panic hook that prints a full backtrace
            hook(&info);
        };
        set_hook(Box::new(new_hook));
        f();
    };


    builder.spawn(f_with_hook).unwrap();
}
