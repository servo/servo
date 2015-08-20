/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net_traits::{AsyncResponseListener, ResponseAction};
use script_task::{ScriptChan, Runnable, CommonScriptMsg};
use std::sync::{Arc, Mutex};

/// An off-thread sink for async network event runnables. All such events are forwarded to
/// a target thread, where they are invoked on the provided context object.
pub struct NetworkListener<T: AsyncResponseListener + PreInvoke + Send + 'static> {
    pub context: Arc<Mutex<T>>,
    pub script_chan: Box<ScriptChan + Send>,
}

impl<T: AsyncResponseListener + PreInvoke + Send + 'static> NetworkListener<T> {
    pub fn notify(&self, action: ResponseAction) {
        if let Err(err) = self.script_chan.send(CommonScriptMsg::RunnableMsg(box ListenerRunnable {
            context: self.context.clone(),
            action: action,
        })) {
            warn!("failed to deliver network data: {:?}", err);
        }
    }
}

/// A gating mechanism that runs before invoking the runnable on the target thread.
/// If the `should_invoke` method returns false, the runnable is discarded without
/// being invoked.
pub trait PreInvoke {
    fn should_invoke(&self) -> bool {
        true
    }
}

/// A runnable for moving the async network events between threads.
struct ListenerRunnable<T: AsyncResponseListener + PreInvoke + Send> {
    context: Arc<Mutex<T>>,
    action: ResponseAction,
}

impl<T: AsyncResponseListener + PreInvoke + Send> Runnable for ListenerRunnable<T> {
    fn handler(self: Box<ListenerRunnable<T>>) {
        let this = *self;
        let context = this.context.lock().unwrap();
        if context.should_invoke() {
            this.action.process(&*context);
        }
    }
}
