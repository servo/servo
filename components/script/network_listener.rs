/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net_traits::{Action, FetchResponseListener, FetchResponseMsg};
use std::sync::{Arc, Mutex};
use task::{TaskCanceller, TaskOnce};
use task_source::TaskSource;
use task_source::networking::NetworkingTaskSource;

/// An off-thread sink for async network event tasks. All such events are forwarded to
/// a target thread, where they are invoked on the provided context object.
pub struct NetworkListener<Listener: PreInvoke + Send + 'static> {
    pub context: Arc<Mutex<Listener>>,
    pub task_source: NetworkingTaskSource,
    pub canceller: Option<TaskCanceller>,
}

impl<Listener: PreInvoke + Send + 'static> NetworkListener<Listener> {
    pub fn notify<A: Action<Listener> + Send + 'static>(&self, action: A) {
        let task = ListenerTask {
            context: self.context.clone(),
            action: action,
        };
        let result = if let Some(ref canceller) = self.canceller {
            self.task_source.queue_with_canceller(task, canceller)
        } else {
            self.task_source.queue_unconditionally(task)
        };
        if let Err(err) = result {
            warn!("failed to deliver network data: {:?}", err);
        }
    }
}

// helps type inference
impl<Listener: FetchResponseListener + PreInvoke + Send + 'static> NetworkListener<Listener> {
    pub fn notify_fetch(&self, action: FetchResponseMsg) {
        self.notify(action);
    }
}

/// A gating mechanism that runs before invoking the task on the target thread.
/// If the `should_invoke` method returns false, the task is discarded without
/// being invoked.
pub trait PreInvoke {
    fn should_invoke(&self) -> bool {
        true
    }
}

/// A task for moving the async network events between threads.
struct ListenerTask<A: Action<Listener> + Send + 'static, Listener: PreInvoke + Send> {
    context: Arc<Mutex<Listener>>,
    action: A,
}

impl<A, Listener> TaskOnce for ListenerTask<A, Listener>
where
    A: Action<Listener> + Send + 'static,
    Listener: PreInvoke + Send,
{
    fn run_once(self) {
        let mut context = self.context.lock().unwrap();
        if context.should_invoke() {
            self.action.process(&mut *context);
        }
    }
}
