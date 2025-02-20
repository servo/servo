/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

use net_traits::{
    Action, BoxedFetchCallback, FetchResponseListener, FetchResponseMsg, ResourceFetchTiming,
    ResourceTimingType,
};
use servo_url::ServoUrl;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceentry::PerformanceEntry;
use crate::dom::performanceresourcetiming::{InitiatorType, PerformanceResourceTiming};
use crate::script_runtime::CanGc;
use crate::task::TaskOnce;
use crate::task_source::SendableTaskSource;

/// An off-thread sink for async network event tasks. All such events are forwarded to
/// a target thread, where they are invoked on the provided context object.
pub(crate) struct NetworkListener<Listener: PreInvoke + Send + 'static> {
    pub(crate) context: Arc<Mutex<Listener>>,
    pub(crate) task_source: SendableTaskSource,
}

pub(crate) trait ResourceTimingListener {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl);
    fn resource_timing_global(&self) -> DomRoot<GlobalScope>;
}

pub(crate) fn submit_timing<T: ResourceTimingListener + FetchResponseListener>(
    listener: &T,
    can_gc: CanGc,
) {
    if listener.resource_timing().timing_type != ResourceTimingType::Resource {
        warn!(
            "Submitting non-resource ({:?}) timing as resource",
            listener.resource_timing().timing_type
        );
        return;
    }

    let (initiator_type, url) = listener.resource_timing_information();
    if initiator_type == InitiatorType::Other {
        warn!("Ignoring InitiatorType::Other resource {:?}", url);
        return;
    }

    submit_timing_data(
        &listener.resource_timing_global(),
        url,
        initiator_type,
        listener.resource_timing(),
        can_gc,
    );
}

pub(crate) fn submit_timing_data(
    global: &GlobalScope,
    url: ServoUrl,
    initiator_type: InitiatorType,
    resource_timing: &ResourceFetchTiming,
    can_gc: CanGc,
) {
    let performance_entry =
        PerformanceResourceTiming::new(global, url, initiator_type, None, resource_timing, can_gc);
    global
        .performance()
        .queue_entry(performance_entry.upcast::<PerformanceEntry>(), can_gc);
}

impl<Listener: PreInvoke + Send + 'static> NetworkListener<Listener> {
    pub(crate) fn notify<A: Action<Listener> + Send + 'static>(&mut self, action: A) {
        self.task_source.queue(ListenerTask {
            context: self.context.clone(),
            action,
        });
    }
}

// helps type inference
impl<Listener: FetchResponseListener + PreInvoke + Send + 'static> NetworkListener<Listener> {
    pub(crate) fn notify_fetch(&mut self, action: FetchResponseMsg) {
        self.notify(action);
    }

    pub(crate) fn into_callback(mut self) -> BoxedFetchCallback {
        Box::new(move |response_msg| self.notify_fetch(response_msg))
    }
}

/// A gating mechanism that runs before invoking the task on the target thread.
/// If the `should_invoke` method returns false, the task is discarded without
/// being invoked.
pub(crate) trait PreInvoke {
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
