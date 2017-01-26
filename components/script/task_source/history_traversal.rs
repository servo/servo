/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::globalscope::GlobalScope;
use dom::window::Window;
use ipc_channel::ipc;
use msg::constellation_msg::TraversalDirection;
use script_thread::{MainThreadScriptMsg, Runnable, RunnableWrapper, ScriptThread};
use script_traits::ScriptMsg as ConstellationMsg;
use std::sync::mpsc::Sender;
use task_source::TaskSource;

#[derive(JSTraceable, Clone)]
pub struct HistoryTraversalTaskSource(pub Sender<MainThreadScriptMsg>);

impl TaskSource for HistoryTraversalTaskSource {
    fn queue_with_wrapper<T>(&self,
                             msg: Box<T>,
                             wrapper: &RunnableWrapper)
                             -> Result<(), ()>
                             where T: Runnable + Send + 'static {
        let msg = HistoryTraversalTask(wrapper.wrap_runnable(msg));
        self.0.send(MainThreadScriptMsg::HistoryTraversal(msg)).map_err(|_| ())
    }
}

impl HistoryTraversalTaskSource {
    pub fn queue_history_traversal(&self, window: &Window, direction: TraversalDirection) {
        let trusted_window = Trusted::new(window);
        let runnable = box HistoryTraversalRunnable {
            window: trusted_window,
            direction: direction,
        };
        let _ = self.queue(runnable, window.upcast());
    }
}

pub struct HistoryTraversalTask(pub Box<Runnable + Send>);

impl HistoryTraversalTask {
    pub fn handle_task(self, script_thread: &ScriptThread) {
        if !self.0.is_cancelled() {
            self.0.main_thread_handler(script_thread);
        }
    }
}

struct HistoryTraversalRunnable {
    window: Trusted<Window>,
    direction: TraversalDirection,
}

impl Runnable for HistoryTraversalRunnable {
    fn name(&self) -> &'static str { "HistoryTraversalRunnable" }

    fn handler(self: Box<HistoryTraversalRunnable>) {
        let window = self.window.root();
        let global_scope = window.upcast::<GlobalScope>();
        let pipeline = global_scope.pipeline_id();
        let (sender, recv) = ipc::channel().expect("Failed to create ipc channel");
        let msg = ConstellationMsg::TraverseHistory(Some(pipeline), self.direction, sender);
        let _ = global_scope.constellation_chan().send(msg);
        let _ =  recv.recv();
    }
}
