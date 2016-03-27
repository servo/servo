/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::refcounted::Trusted;
use dom::event::{EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use script_thread::{MainThreadRunnable, MainThreadScriptMsg, Runnable, ScriptThread};
use std::result::Result;
use std::sync::mpsc::Sender;
use string_cache::Atom;
use task_source::TaskSource;

#[derive(JSTraceable)]
pub struct DOMManipulationTaskSource(pub Sender<MainThreadScriptMsg>);

impl TaskSource<DOMManipulationTask> for DOMManipulationTaskSource {
    fn queue(&self, msg: DOMManipulationTask) -> Result<(), ()> {
        self.0.send(MainThreadScriptMsg::DOMManipulation(msg)).map_err(|_| ())
    }

    fn clone(&self) -> Box<TaskSource<DOMManipulationTask> + Send> {
        box DOMManipulationTaskSource((&self.0).clone())
    }
}

pub enum DOMManipulationTask {
    // https://html.spec.whatwg.org/multipage/#the-end step 7
    DocumentProgress(Box<Runnable + Send>),
    // https://dom.spec.whatwg.org/#concept-event-fire
    FireEvent(Atom, Trusted<EventTarget>, EventBubbles, EventCancelable),
    // https://html.spec.whatwg.org/multipage/#fire-a-simple-event
    FireSimpleEvent(Atom, Trusted<EventTarget>),
    // https://html.spec.whatwg.org/multipage/#details-notification-task-steps
    FireToggleEvent(Box<Runnable + Send>),
    // https://html.spec.whatwg.org/multipage/#planned-navigation
    PlannedNavigation(Box<Runnable + Send>),
    // https://html.spec.whatwg.org/multipage/#send-a-storage-notification
    SendStorageNotification(Box<MainThreadRunnable + Send>)
}

impl DOMManipulationTask {
    pub fn handle_msg(self, script_thread: &ScriptThread) {
        use self::DOMManipulationTask::*;

        match self {
            DocumentProgress(runnable) => runnable.handler(),
            FireEvent(name, element, bubbles, cancelable) => {
                let target = element.root();
                target.fire_event(&*name, bubbles, cancelable);
            }
            FireSimpleEvent(name, element) => {
                let target = element.root();
                target.fire_simple_event(&*name);
            }
            FireToggleEvent(runnable) => runnable.handler(),
            PlannedNavigation(runnable) => runnable.handler(),
            SendStorageNotification(runnable) => runnable.handler(script_thread)
        }
    }
}
