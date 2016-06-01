/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net_traits::{Action, AsyncResponseListener, ResponseAction};
use script_runtime::ScriptThreadEventCategory::NetworkEvent;
use script_runtime::{CommonScriptMsg, ScriptChan};
use script_thread::Runnable;
use std::sync::{Arc, Mutex};

/// An off-thread sink for async network event runnables. All such events are forwarded to
/// a target thread, where they are invoked on the provided context object.
pub struct NetworkListener<Listener: PreInvoke + Send + 'static> {
    pub context: Arc<Mutex<Listener>>,
    pub script_chan: Box<ScriptChan + Send>,
}

impl<Listener: PreInvoke + Send + 'static> NetworkListener<Listener> {
    pub fn notify<A: Action<Listener>+Send+'static>(&self, action: A) {
        if let Err(err) = self.script_chan.send(CommonScriptMsg::RunnableMsg(NetworkEvent, box ListenerRunnable {
            context: self.context.clone(),
            action: action,
        })) {
            warn!("failed to deliver network data: {:?}", err);
        }
    }
}

// helps type inference
impl<Listener: AsyncResponseListener + PreInvoke + Send + 'static> NetworkListener<Listener> {
    pub fn notify_action(&self, action: ResponseAction) {
        self.notify(action);
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
struct ListenerRunnable<A: Action<Listener>+Send+'static, Listener: PreInvoke + Send> {
    context: Arc<Mutex<Listener>>,
    action: A,
}

impl<A: Action<Listener>+Send+'static, Listener: PreInvoke + Send> Runnable for ListenerRunnable<A, Listener> {
    fn handler(self: Box<ListenerRunnable<A, Listener>>) {
        let this = *self;
        let mut context = this.context.lock().unwrap();
        if context.should_invoke() {
            this.action.process(&mut *context);
        }
    }
}
