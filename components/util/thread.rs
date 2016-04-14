/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use panicking;
use serde::Serialize;
use std::any::Any;
use std::borrow::ToOwned;
use std::sync::mpsc::Sender;
use std::thread;
use thread_state;

pub type PanicReason = Option<String>;

pub fn spawn_named<F>(name: String, f: F)
    where F: FnOnce() + Send + 'static
{
    spawn_named_with_send_on_failure_maybe(name, None, f, |_| {});
}

pub trait AddFailureDetails {
    fn add_panic_message(&mut self, message: String);
    fn add_panic_object(&mut self, object: &Any) {
        if let Some(message) = object.downcast_ref::<String>() {
            self.add_panic_message(message.to_owned());
        } else if let Some(&message) = object.downcast_ref::<&'static str>() {
            self.add_panic_message(message.to_owned());
        }
    }
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
        // Discard any errors to avoid double-panic
        let _ = self.send(value);
    }
}

impl<T> SendOnFailure for IpcSender<T> where T: Send + Serialize + 'static {
    type Value = T;
    fn send_on_failure(&mut self, value: T) {
        // Discard any errors to avoid double-panic
        let _ = self.send(value);
    }
}

/// Arrange to send a particular message to a channel if the thread fails.
pub fn spawn_named_with_send_on_failure<F, T, S>(name: String,
                                                 state: thread_state::ThreadState,
                                                 f: F,
                                                 mut msg: T,
                                                 mut dest: S)
    where F: FnOnce() + Send + 'static,
          T: Send + AddFailureDetails + 'static,
          S: Send + SendOnFailure + 'static,
          S::Value: From<T>,
{
    spawn_named_with_send_on_failure_maybe(name, Some(state), f,
                                           move |err| {
                                               msg.add_panic_object(err);
                                               dest.send_on_failure(S::Value::from(msg));
                                           });
}

fn spawn_named_with_send_on_failure_maybe<F, G>(name: String,
                                                 state: Option<thread_state::ThreadState>,
                                                 f: F,
                                                 fail: G)
                                                 where F: FnOnce() + Send + 'static,
                                                       G: FnOnce(&(Any + Send)) + Send + 'static {


    let builder = thread::Builder::new().name(name.clone());

    let local = panicking::PanicHandlerLocal {
        fail: Box::new(fail),
    };

    let f_with_state = move || {
        if let Some(state) = state {
            thread_state::initialize(state);
        }

        // set the handler
        panicking::LOCAL_INFO.with(|i| {
            *i.borrow_mut() = Some(local);
        });
        f();
    };


    builder.spawn(f_with_state).unwrap();
}
