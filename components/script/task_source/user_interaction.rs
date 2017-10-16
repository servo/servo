/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::event::{EventBubbles, EventCancelable, EventTask};
use dom::eventtarget::EventTarget;
use dom::window::Window;
use script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use script_thread::MainThreadScriptMsg;
use servo_atoms::Atom;
use std::fmt;
use std::result::Result;
use std::sync::mpsc::Sender;
use task::{TaskCanceller, TaskOnce};
use task_source::TaskSource;

#[derive(Clone, JSTraceable)]
pub struct UserInteractionTaskSource(pub Sender<MainThreadScriptMsg>);

impl fmt::Debug for UserInteractionTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UserInteractionTaskSource(...)")
    }
}

impl TaskSource for UserInteractionTaskSource {
    fn queue_with_canceller<T>(
        &self,
        task: T,
        canceller: &TaskCanceller,
    ) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = MainThreadScriptMsg::Common(CommonScriptMsg::Task(
            ScriptThreadEventCategory::InputEvent,
            Box::new(canceller.wrap_task(task)),
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
        let task = EventTask { target, name, bubbles, cancelable };
        let _ = self.queue(task, window.upcast());
    }
}
