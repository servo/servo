/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use opts;
use serde::Serialize;
use std::boxed::FnBox;
use std::cell::RefCell;
use std::io::{Write, stderr};
use std::panic::{PanicInfo, take_hook, set_hook};
use std::sync::mpsc::Sender;
use std::sync::{Once, ONCE_INIT};
use std::thread;
use thread_state;

pub fn spawn_named<F>(name: String, f: F)
    where F: FnOnce() + Send + 'static
{
    spawn_named_with_send_on_failure_maybe(name, None, f, || {});
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

    spawn_named_with_send_on_failure_maybe(name, Some(state), f, move || dest.send_on_failure(msg));
}

// only set the panic hook once
static HOOK_SET: Once = ONCE_INIT;

/// TLS data pertaining to how failures should be reported
struct PanicHandlerLocal {
    /// failure handler passed through spawn_named_with_send_on_failure
    fail: Box<FnBox() + Send + 'static>
}

thread_local!(static LOCAL_INFO: RefCell<Option<PanicHandlerLocal>> = RefCell::new(None));

fn spawn_named_with_send_on_failure_maybe<F, G>(name: String,
                                                 state: Option<thread_state::ThreadState>,
                                                 f: F,
                                                 fail: G)
                                                 where F: FnOnce() + Send + 'static,
                                                       G: FnOnce() + Send + 'static {


    let builder = thread::Builder::new().name(name.clone());

    // store it locally, we can't trust that opts::get() will work whilst panicking
    let full_backtraces = opts::get().full_backtraces;

    let local = PanicHandlerLocal {
        fail: Box::new(fail),
    };

    // Set the panic handler only once. It is global.
    HOOK_SET.call_once(|| {
        // The original backtrace-printing hook. We still want to call this
        let hook = take_hook();

        let new_hook = move |info: &PanicInfo| {
            let payload = info.payload();

            // Notify error handlers stored in LOCAL_INFO if any
            LOCAL_INFO.with(|i| {
                if let Some(info) = i.borrow_mut().take() {
                    debug!("Thread `{}` failed, notifying error handlers", name.clone());
                    (info.fail)();
                }
            });

            // Skip cascading SendError/RecvError backtraces if allowed
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

            // Call the old hook to get the backtrace
            hook(&info);
        };
        set_hook(Box::new(new_hook));
    });

    let f_with_state = move || {
        if let Some(state) = state {
            thread_state::initialize(state);
        }

        // set the handler
        LOCAL_INFO.with(|i| {
            *i.borrow_mut() = Some(local);
        });
        f();
    };


    builder.spawn(f_with_state).unwrap();
}
