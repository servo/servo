/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script_thread::{MainThreadScriptMsg, Runnable};
use std::sync::mpsc::Sender;
use task_source::TaskSource;

#[derive(JSTraceable)]
pub struct HistoryTraversalTaskSource(pub Sender<MainThreadScriptMsg>);

impl TaskSource<HistoryTraversalTask> for HistoryTraversalTaskSource {
    fn queue(&self, msg: HistoryTraversalTask) -> Result<(), ()> {
        self.0.send(MainThreadScriptMsg::HistoryTraversal(msg)).map_err(|_| ())
    }
}

impl HistoryTraversalTaskSource {
    pub fn clone(&self) -> Box<TaskSource<HistoryTraversalTask> + Send> {
        box HistoryTraversalTaskSource((&self.0).clone())
    }
}


pub enum HistoryTraversalTask {
    FireNavigationEvent(Box<Runnable + Send>),
}

impl HistoryTraversalTask {
    pub fn handle_task(self) {
        use self::HistoryTraversalTask::*;

        match self {
            FireNavigationEvent(runnable) => runnable.handler()
        }
    }
}
