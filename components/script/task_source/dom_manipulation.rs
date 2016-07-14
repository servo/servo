/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::global::GlobalRef;
use dom::bindings::refcounted::Trusted;
use dom::event::{EventBubbles, EventCancelable, EventRunnable, SimpleEventRunnable};
use dom::eventtarget::EventTarget;
use dom::window::Window;
use script_thread::{MainThreadScriptMsg, Runnable, RunnableWrapper, ScriptThread};
use std::result::Result;
use std::sync::mpsc::Sender;
use string_cache::Atom;
use task_source::TaskSource;

#[derive(JSTraceable, Clone)]
pub struct DOMManipulationTaskSource(pub Sender<MainThreadScriptMsg>);

impl TaskSource for DOMManipulationTaskSource {
    fn queue_with_wrapper<T>(&self,
                             msg: Box<T>,
                             wrapper: &RunnableWrapper)
                             -> Result<(), ()>
                             where T: Runnable + Send + 'static {
        let msg = DOMManipulationTask(wrapper.wrap_runnable(msg));
        self.0.send(MainThreadScriptMsg::DOMManipulation(msg)).map_err(|_| ())
    }
}

impl DOMManipulationTaskSource {
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
        let _ = self.queue(runnable, GlobalRef::Window(window));
    }

    pub fn queue_simple_event(&self, target: &EventTarget, name: Atom, window: &Window) {
        let target = Trusted::new(target);
        let runnable = box SimpleEventRunnable {
            target: target,
            name: name,
        };
        let _ = self.queue(runnable, GlobalRef::Window(window));
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
