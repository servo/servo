/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use opts;
use std::any::Any;
use std::boxed::FnBox;
use std::cell::RefCell;
use std::io::{Write, stderr};
use std::panic::{PanicInfo, take_hook, set_hook};
use std::sync::{Once, ONCE_INIT};
use std::thread;

// only set the panic hook once
static HOOK_SET: Once = ONCE_INIT;

/// TLS data pertaining to how failures should be reported
pub struct PanicHandlerLocal {
    /// failure handler passed through spawn_named_with_send_on_failure
    pub fail: Box<FnBox(&Any)>
}

thread_local!(pub static LOCAL_INFO: RefCell<Option<PanicHandlerLocal>> = RefCell::new(None));

/// Set the thread-local panic hook
pub fn set_thread_local_hook(local: Box<FnBox(&Any)>) {
    LOCAL_INFO.with(|i| *i.borrow_mut() = Some(PanicHandlerLocal { fail: local }));
}

/// Initiates the custom panic hook
/// Should be called in main() after arguments have been parsed
pub fn initiate_panic_hook() {

    // store it locally, we can't trust that opts::get() will work whilst panicking
    let full_backtraces = opts::get().full_backtraces;

    // Set the panic handler only once. It is global.
    HOOK_SET.call_once(|| {
        // The original backtrace-printing hook. We still want to call this
        let hook = take_hook();

        let new_hook = move |info: &PanicInfo| {
            let payload = info.payload();
            let name = thread::current().name().unwrap_or("<unknown thread>").to_string();
            // Notify error handlers stored in LOCAL_INFO if any
            LOCAL_INFO.with(|i| {
                if let Some(info) = i.borrow_mut().take() {
                    debug!("Thread `{}` failed, notifying error handlers", name);
                    (info.fail).call_box((payload, ));
                }
            });

            // Skip cascading SendError/RecvError backtraces if allowed
            if !full_backtraces {
                if let Some(s) = payload.downcast_ref::<String>() {
                    if s.contains("SendError") {
                        let err = stderr();
                        let _ = write!(err.lock(), "Thread \"{}\" panicked with an unwrap of \
                                                    `SendError` (backtrace skipped)\n", name);
                        return;
                    } else if s.contains("RecvError")  {
                        let err = stderr();
                        let _ = write!(err.lock(), "Thread \"{}\" panicked with an unwrap of \
                                                    `RecvError` (backtrace skipped)\n", name);
                        return;
                    }
                }
            }

            // Call the old hook to get the backtrace
            hook(&info);
        };
        set_hook(Box::new(new_hook));
    });


}
