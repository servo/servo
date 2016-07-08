/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::refcounted::Trusted;
use dom::event::{EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use script_thread::{MainThreadScriptMsg, Runnable, ScriptThread};
use std::result::Result;
use std::sync::mpsc::Sender;
use string_cache::Atom;
use task_source::TaskSource;

#[derive(JSTraceable, Clone)]
pub struct DOMManipulationTaskSource(pub Sender<MainThreadScriptMsg>);

impl TaskSource<DOMManipulationTask> for DOMManipulationTaskSource {
    fn queue(&self, msg: DOMManipulationTask) -> Result<(), ()> {
        self.0.send(MainThreadScriptMsg::DOMManipulation(msg)).map_err(|_| ())
    }
}

impl DOMManipulationTaskSource {
    pub fn queue_event(&self,
                       target: &EventTarget,
                       name: Atom,
                       bubbles: EventBubbles,
                       cancelable: EventCancelable) {
        let target = Trusted::new(target);
        let _ = self.0.send(MainThreadScriptMsg::DOMManipulation(DOMManipulationTask::FireEvent(
            target, name, bubbles, cancelable)));
    }

    pub fn queue_simple_event(&self, target: &EventTarget, name: Atom) {
        let target = Trusted::new(target);
        let _ = self.0.send(MainThreadScriptMsg::DOMManipulation(DOMManipulationTask::FireSimpleEvent(
            target, name)));
    }
}

pub enum DOMManipulationTask {
    // https://dom.spec.whatwg.org/#concept-event-fire
    FireEvent(Trusted<EventTarget>, Atom, EventBubbles, EventCancelable),
    // https://html.spec.whatwg.org/multipage/#fire-a-simple-event
    FireSimpleEvent(Trusted<EventTarget>, Atom),

    Runnable(Box<Runnable + Send>),
}

impl DOMManipulationTask {
    pub fn handle_task(self, script_thread: &ScriptThread) {
        use self::DOMManipulationTask::*;

        match self {
            FireEvent(element, name, bubbles, cancelable) => {
                let target = element.root();
                target.fire_event(&*name, bubbles, cancelable);
            }
            FireSimpleEvent(element, name) => {
                let target = element.root();
                target.fire_simple_event(&*name);
            }
            Runnable(runnable) => {
                if !runnable.is_cancelled() {
                    runnable.main_thread_handler(script_thread);
                }
            }
        }
    }
}
