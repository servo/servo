/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::any::Any;
use std::boxed::FnBox;
use std::cell::RefCell;
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

    // Set the panic handler only once. It is global.
    HOOK_SET.call_once(|| {
        // The original backtrace-printing hook. We still want to call this
        let hook = take_hook();

        let new_hook = move |info: &PanicInfo| {
            let payload = info.payload();
            let name = thread::current().name().unwrap_or("<unknown thread>").to_string();
            // Notify error handlers stored in LOCAL_INFO if any
            LOCAL_INFO.with(|i| {
                if let Some(local_info) = i.borrow_mut().take() {
                    debug!("Thread `{}` failed, notifying error handlers", name);
                    (local_info.fail).call_box((payload, ));
                } else {
                    hook(&info);
                }
            });
        };
        set_hook(Box::new(new_hook));
    });

}
