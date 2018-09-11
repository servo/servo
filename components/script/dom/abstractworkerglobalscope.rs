/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::DevtoolScriptControlMsg;
use dom::abstractworker::WorkerScriptMsg;
use dom::bindings::conversions::DerivedFrom;
use dom::bindings::reflector::DomObject;
use dom::dedicatedworkerglobalscope::{AutoWorkerReset, DedicatedWorkerScriptMsg};
use dom::globalscope::GlobalScope;
use dom::worker::TrustedWorkerAddress;
use dom::workerglobalscope::WorkerGlobalScope;
use script_runtime::{ScriptChan, CommonScriptMsg, ScriptPort};
use std::sync::mpsc::{Receiver, Select, Sender};
use task_queue::{QueuedTaskConversion, TaskQueue};

/// A ScriptChan that can be cloned freely and will silently send a TrustedWorkerAddress with
/// common event loop messages. While this SendableWorkerScriptChan is alive, the associated
/// Worker object will remain alive.
#[derive(Clone, JSTraceable)]
pub struct SendableWorkerScriptChan {
    pub sender: Sender<DedicatedWorkerScriptMsg>,
    pub worker: TrustedWorkerAddress,
}

impl ScriptChan for SendableWorkerScriptChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        let msg = DedicatedWorkerScriptMsg::CommonWorker(self.worker.clone(), WorkerScriptMsg::Common(msg));
        self.sender.send(msg).map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        Box::new(SendableWorkerScriptChan {
            sender: self.sender.clone(),
            worker: self.worker.clone(),
        })
    }
}

/// A ScriptChan that can be cloned freely and will silently send a TrustedWorkerAddress with
/// worker event loop messages. While this SendableWorkerScriptChan is alive, the associated
/// Worker object will remain alive.
#[derive(Clone, JSTraceable)]
pub struct WorkerThreadWorkerChan {
    pub sender: Sender<DedicatedWorkerScriptMsg>,
    pub worker: TrustedWorkerAddress,
}

impl ScriptChan for WorkerThreadWorkerChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        let msg = DedicatedWorkerScriptMsg::CommonWorker(self.worker.clone(), WorkerScriptMsg::Common(msg));
        self.sender
            .send(msg)
            .map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        Box::new(WorkerThreadWorkerChan {
            sender: self.sender.clone(),
            worker: self.worker.clone(),
        })
    }
}

impl ScriptPort for Receiver<DedicatedWorkerScriptMsg> {
    fn recv(&self) -> Result<CommonScriptMsg, ()> {
        let common_msg = match self.recv() {
            Ok(DedicatedWorkerScriptMsg::CommonWorker(_worker, common_msg)) => common_msg,
            Err(_) => return Err(()),
            Ok(DedicatedWorkerScriptMsg::WakeUp) => panic!("unexpected worker event message!")
        };
        match common_msg {
            WorkerScriptMsg::Common(script_msg) => Ok(script_msg),
            WorkerScriptMsg::DOMMessage(_) => panic!("unexpected worker event message!"),
        }
    }
}

pub trait WorkerEventLoopMethods {
    type TimerMsg: Send;
    type WorkerMsg: QueuedTaskConversion + Send;
    type Event;
    fn timer_event_port(&self) -> &Receiver<Self::TimerMsg>;
    fn task_queue(&self) -> &TaskQueue<Self::WorkerMsg>;
    fn handle_event(&self, event: Self::Event);
    fn handle_worker_post_event(&self, worker: &TrustedWorkerAddress) -> Option<AutoWorkerReset>;
    fn from_worker_msg(&self, msg: Self::WorkerMsg) -> Self::Event;
    fn from_timer_msg(&self, msg: Self::TimerMsg) -> Self::Event;
    fn from_devtools_msg(&self, msg: DevtoolScriptControlMsg) -> Self::Event;
}

#[allow(unsafe_code)]
// https://html.spec.whatwg.org/multipage/#worker-event-loop
pub fn run_worker_event_loop<T, TimerMsg, WorkerMsg, Event>(worker_scope: &T,
                                                            worker: Option<&TrustedWorkerAddress>)
where
    TimerMsg: Send,
    WorkerMsg: QueuedTaskConversion + Send,
    T: WorkerEventLoopMethods<TimerMsg = TimerMsg,  WorkerMsg = WorkerMsg, Event = Event>
    + DerivedFrom<WorkerGlobalScope> + DerivedFrom<GlobalScope>
    + DomObject {
    let scope = worker_scope.upcast::<WorkerGlobalScope>();
    let timer_event_port = worker_scope.timer_event_port();
    let devtools_port = scope.from_devtools_receiver();
    let task_queue = worker_scope.task_queue();
    let sel = Select::new();
    let mut worker_handle = sel.handle(task_queue.select());
    let mut timer_event_handle = sel.handle(timer_event_port);
    let mut devtools_handle = sel.handle(devtools_port);
    unsafe {
        worker_handle.add();
        timer_event_handle.add();
        if scope.from_devtools_sender().is_some() {
            devtools_handle.add();
        }
    }
    let ret = sel.wait();
    let event = {
        if ret == worker_handle.id() {
            task_queue.take_tasks();
            worker_scope.from_worker_msg(task_queue.recv().unwrap())
        } else if ret == timer_event_handle.id() {
            worker_scope.from_timer_msg(timer_event_port.recv().unwrap())
        } else if ret == devtools_handle.id() {
            worker_scope.from_devtools_msg(devtools_port.recv().unwrap())
        } else {
            panic!("unexpected select result!")
        }
    };
    let mut sequential = vec![];
    sequential.push(event);
    // https://html.spec.whatwg.org/multipage/#worker-event-loop
    // Once the WorkerGlobalScope's closing flag is set to true,
    // the event loop's task queues must discard any further tasks
    // that would be added to them
    // (tasks already on the queue are unaffected except where otherwise specified).
    while !scope.is_closing() {
        // Batch all events that are ready.
        // The task queue will throttle non-priority tasks if necessary.
        match task_queue.try_recv() {
            Err(_) => match timer_event_port.try_recv() {
                Err(_) => match devtools_port.try_recv() {
                    Err(_) => break,
                    Ok(ev) => sequential.push(worker_scope.from_devtools_msg(ev)),
                },
                Ok(ev) => sequential.push(worker_scope.from_timer_msg(ev)),
            },
            Ok(ev) => sequential.push(worker_scope.from_worker_msg(ev)),
        }
    }
    // Step 3
    for event in sequential {
        worker_scope.handle_event(event);
        // Step 6
        let _ar = match worker {
            Some(worker) => worker_scope.handle_worker_post_event(worker),
            None => None
        };
        worker_scope.upcast::<GlobalScope>().perform_a_microtask_checkpoint();
    }
}
