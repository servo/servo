/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::event::{EventBubbles, EventCancelable, EventRunnable};
use dom::eventtarget::EventTarget;
use dom::window::Window;
use script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use script_thread::{MainThreadScriptMsg, Runnable, RunnableWrapper};
use servo_atoms::Atom;
use std::fmt;
use std::result::Result;
use std::sync::mpsc::Sender;
use task_source::TaskSource;

#[derive(Clone, JSTraceable)]
pub struct UserInteractionTaskSource(pub Sender<MainThreadScriptMsg>);

impl fmt::Debug for UserInteractionTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UserInteractionTaskSource(...)")
    }
}

impl TaskSource for UserInteractionTaskSource {
    fn queue_with_wrapper<T>(
        &self,
        msg: Box<T>,
        wrapper: &RunnableWrapper,
    ) -> Result<(), ()>
    where
        T: Runnable + Send + 'static,
    {
        let msg = MainThreadScriptMsg::Common(CommonScriptMsg::RunnableMsg(
            ScriptThreadEventCategory::InputEvent,
            wrapper.wrap_runnable(msg),
        ));
        self.0.send(msg).map_err(|_| ())
    }
}

impl UserInteractionTaskSource {
    pub fn queue_event(&self,
                       target: &EventTarget,
                       name: Atom,
                       bubbles: EventBubbles,
                       cancelable: EventCancelable,
                       window: &Window) {
        let target = Trusted::new(target);
        let runnable = box EventRunnable {
            target: target,
            name: name,
            bubbles: bubbles,
            cancelable: cancelable,
        };
        let _ = self.queue(runnable, window.upcast());
    }
}
