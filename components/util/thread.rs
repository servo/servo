/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use backtrace::Backtrace;
use ipc_channel::ipc::IpcSender;
use panicking;
use serde::Serialize;
use std::any::Any;
use std::thread;
use thread_state;

pub fn spawn_named<F>(name: String, f: F)
    where F: FnOnce() + Send + 'static
{
    thread::Builder::new().name(name).spawn(f).expect("Thread spawn failed");
}

/// Arrange to send a particular message to a channel if the thread fails.
pub fn spawn_named_with_send_on_panic<F, Id>(name: String,
                                             state: thread_state::ThreadState,
                                             f: F,
                                             id: Id,
                                             panic_chan: IpcSender<(Id, String, String)>)
    where F: FnOnce() + Send + 'static,
          Id: Copy + Send + Serialize + 'static,
{
    thread::Builder::new().name(name).spawn(move || {
        thread_state::initialize(state);
        panicking::set_thread_local_hook(Box::new(move |payload: &Any| {
            debug!("Thread failed, notifying constellation");
            let reason = payload.downcast_ref::<String>().map(|s| String::from(&**s))
                .or(payload.downcast_ref::<&'static str>().map(|s| String::from(*s)))
                .unwrap_or_else(|| String::from("<unknown reason>"));
            // FIXME(ajeffrey): Really we should send the backtrace itself,
            // not just a string representation. Unfortunately we can't, because
            // Servo won't compile backtrace with the serialize-serde feature:
            //
            // .../quasi_macros-0.9.0/src/lib.rs:19:29: 19:32 error: mismatched types:
            //  expected `&mut syntex::Registry`,
            //     found `&mut rustc_plugin::Registry<'_>`
            // (expected struct `syntex::Registry`,
            //     found struct `rustc_plugin::Registry`) [E0308]
            // .../quasi_macros-0.9.0/src/lib.rs:19     quasi_codegen::register(reg);
            //
            // so for the moment we just send a debug string.
            let backtrace = format!("{:?}", Backtrace::new());
            let _ = panic_chan.send((id, reason, backtrace));
        }));
        f()
    }).expect("Thread spawn failed");
}
