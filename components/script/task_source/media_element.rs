/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::result::Result;

use base::id::PipelineId;
use crossbeam_channel::Sender;
use servo_atoms::Atom;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::event::SimpleEventTask;
use crate::dom::eventtarget::EventTarget;
use crate::dom::window::Window;
use crate::script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use crate::script_thread::MainThreadScriptMsg;
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_source::{TaskSource, TaskSourceName};

#[derive(Clone, JSTraceable)]
pub struct MediaElementTaskSource(
    #[no_trace] pub Sender<MainThreadScriptMsg>,
    #[no_trace] pub PipelineId,
);

impl fmt::Debug for MediaElementTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MediaElementTaskSource(...)")
    }
}

impl TaskSource for MediaElementTaskSource {
    const NAME: TaskSourceName = TaskSourceName::MediaElement;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = MainThreadScriptMsg::Common(CommonScriptMsg::Task(
            ScriptThreadEventCategory::ScriptEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            MediaElementTaskSource::NAME,
        ));
        self.0.send(msg).map_err(|_| ())
    }
}

impl MediaElementTaskSource {
    pub fn queue_simple_event(&self, target: &EventTarget, name: Atom, window: &Window) {
        let target = Trusted::new(target);
        let _ = self.queue(SimpleEventTask { target, name }, window.upcast());
    }
}
