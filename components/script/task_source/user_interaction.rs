/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::refcounted::Trusted;
use dom::event::{EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use script_thread::MainThreadScriptMsg;
use std::result::Result;
use std::sync::mpsc::Sender;
use string_cache::Atom;
use task_source::TaskSource;

#[derive(JSTraceable, Clone)]
pub struct UserInteractionTaskSource(pub Sender<MainThreadScriptMsg>);

impl TaskSource<UserInteractionTask> for UserInteractionTaskSource {
    fn queue(&self, msg: UserInteractionTask) -> Result<(), ()> {
        self.0.send(MainThreadScriptMsg::UserInteraction(msg)).map_err(|_| ())
    }
}

impl UserInteractionTaskSource {
    pub fn queue_event(&self,
                   target: &EventTarget,
                   name: Atom,
                   bubbles: EventBubbles,
                   cancelable: EventCancelable) {
        let target = Trusted::new(target);
        let _ = self.0.send(MainThreadScriptMsg::UserInteraction(UserInteractionTask::FireEvent(
            target, name, bubbles, cancelable)));
    }
}

pub enum UserInteractionTask {
    // https://dom.spec.whatwg.org/#concept-event-fire
    FireEvent(Trusted<EventTarget>, Atom, EventBubbles, EventCancelable),
}

impl UserInteractionTask {
    pub fn handle_task(self) {
        use self::UserInteractionTask::*;

        match self {
            FireEvent(element, name, bubbles, cancelable) => {
                let target = element.root();
                target.fire_event(&*name, bubbles, cancelable);
            }
        }
    }
}
