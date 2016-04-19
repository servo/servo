/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::refcounted::Trusted;
use dom::event::{EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use script_runtime::{CommonScriptMsg, ScriptChan};
use script_thread::MainThreadScriptMsg;
use std::result::Result;
use std::sync::mpsc::Sender;
use string_cache::Atom;
use task_source::TaskSource;

#[derive(JSTraceable)]
pub struct UserInteractionTaskSource(pub Sender<MainThreadScriptMsg>);

impl TaskSource<UserInteractionTask> for UserInteractionTaskSource {
    fn queue(&self, msg: UserInteractionTask) -> Result<(), ()> {
        self.0.send(MainThreadScriptMsg::UserInteraction(msg)).map_err(|_| ())
    }

    fn clone(&self) -> Box<TaskSource<UserInteractionTask> + Send> {
        box UserInteractionTaskSource((&self.0).clone())
    }
}

pub enum UserInteractionTask {
    // https://dom.spec.whatwg.org/#concept-event-fire
    FireEvent(Atom, Trusted<EventTarget>, EventBubbles, EventCancelable),
    // https://html.spec.whatwg.org/multipage/#fire-a-simple-event
    FireSimpleEvent(Atom, Trusted<EventTarget>)
}

impl UserInteractionTask {
    pub fn handle_task(self) {
        use self::UserInteractionTask::*;

        match self {
            FireEvent(name, element, bubbles, cancelable) => {
                let target = element.root();
                target.fire_event(&*name, bubbles, cancelable);
            }
            FireSimpleEvent(name, element) => {
                let target = element.root();
                target.fire_simple_event(&*name);
            }
        }
    }
}
