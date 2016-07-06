/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::refcounted::Trusted;
use dom::element::Element;
use dom::event::{EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use script_thread::{MainThreadScriptMsg, Runnable, RunnableWrapper, ScriptThread};
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
                       cancelable: EventCancelable,
                       wrapper: Option<RunnableWrapper>) {
        let target = Trusted::new(target);
        let runnable = match wrapper {
            Some(wrapper) => {
                wrapper.wrap_runnable(EventRunnable {
                    target: target,
                    name: name,
                    bubbles: bubbles,
                    cancelable: cancelable,
                })
            },
            None => {
                box EventRunnable {
                    target: target,
                    name: name,
                    bubbles: bubbles,
                    cancelable: cancelable,
                }
            }
        };
        let _ = self.0.send(MainThreadScriptMsg::DOMManipulation(DOMManipulationTask(runnable)));
    }

    pub fn queue_simple_event(&self, target: &EventTarget, name: Atom, wrapper: Option<RunnableWrapper>) {
        let target = Trusted::new(target);
        let runnable = match wrapper {
            Some(wrapper) => {
                wrapper.wrap_runnable(SimpleEventRunnable {
                    target: target,
                    name: name,
                })
            },
            None => {
                box SimpleEventRunnable {
                    target: target,
                    name: name,
                }
            }
        };
        let _ = self.0.send(MainThreadScriptMsg::DOMManipulation(DOMManipulationTask(runnable)));
    }
}

struct EventRunnable {
    target: Trusted<EventTarget>,
    name: Atom,
    bubbles: EventBubbles,
    cancelable: EventCancelable,
}

impl Runnable for EventRunnable {
    fn name(&self) -> &'static str { "EventRunnable" }

    fn handler(self: Box<EventRunnable>) {
        let target = self.target.root();
        target.fire_event(&*self.name, self.bubbles, self.cancelable);
    }
}

struct SimpleEventRunnable {
    target: Trusted<EventTarget>,
    name: Atom,
}

impl Runnable for SimpleEventRunnable {
    fn name(&self) -> &'static str { "SimpleEventRunnable" }

    fn handler(self: Box<SimpleEventRunnable>) {
        let target = self.target.root();
        target.fire_simple_event(&*self.name);
    }
}

pub struct DOMManipulationTask(pub Box<Runnable + Send>);

impl DOMManipulationTask {
    pub fn handle_task(self, script_thread: &ScriptThread) {
        if !self.0.is_cancelled() {
            self.0.main_thread_handler(script_thread);
        }
    }
}
