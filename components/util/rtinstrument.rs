/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use opts;
use std::any::Any;
#[cfg(not(test))]
use std::io::File;
//use std::mem;
//use std::raw;
use std::rt::Runtime;
use std::rt::local::Local;
//use std::rt::rtio;
use std::rt::task::{Task, TaskOpts, BlockedTask};
use std_time;
use std::sync::Mutex;
#[cfg(not(test))]
use serialize::json;

#[deriving(Encodable)]
pub enum Event {
    Spawn,
    Schedule,
    Unschedule,
    Death,
}

#[deriving(Encodable)]
pub struct Message {
    timestamp: u64,
    event: Event,
}

#[deriving(Encodable)]
pub struct TaskStats {
    pub name: String,
    pub messages: Vec<Message>,
    pub task_id: uint,
}

struct InstrumentedRuntime {
    inner: Option<Box<Runtime + Send>>,
    messages: Vec<Message>,
}

#[deriving(Encodable)]
pub struct GlobalState {
    task_stats: Vec<TaskStats>,
}

#[cfg(not(test))]
pub fn teardown() {
    if opts::get().profile_tasks {
        let state = GLOBAL_STATE.lock();
        let result = json::encode(&*state);
        let path = Path::new("thread_trace.json");
        let mut file = File::create(&path).unwrap();
        file.write_str(result.as_slice()).unwrap();
    }
}

impl GlobalState {
    fn new() -> GlobalState {
        GlobalState {
            task_stats: vec!(),
        }
    }
}

lazy_static! {
    pub static ref GLOBAL_STATE: Mutex<GlobalState> = Mutex::new(GlobalState::new());
}

/// Instrument all code run inside the specific block, returning a vector of all
/// messages which occurred.
pub fn instrument(f: proc()) {
    if opts::get().profile_tasks {
        install();
        f();
        let rt = uninstall();
        let task_id = rt.task_id();
        let name = {
            let task = Local::borrow(None::<Task>);
            match task.name {
                Some(ref name) => name.to_string(),
                None => "unknown".into_string(),
            }
        };
        let stats = TaskStats {
            name: name,
            messages: rt.messages,
            task_id: task_id,
        };
        let mut state = GLOBAL_STATE.lock();
        state.task_stats.push(stats);
    } else {
        f();
    }
}

/// Installs an instrumented runtime which will append to the given vector of
/// messages.
///
/// The instrumented runtime is installed into the current task.
fn install() {
    let mut task = Local::borrow(None::<Task>);
    let rt = task.take_runtime();
    let mut new_rt = box InstrumentedRuntime {
        inner: Some(rt),
        messages: vec!(),
    };
    new_rt.log(Event::Spawn);
    task.put_runtime(new_rt);
}

/// Uninstalls the runtime from the current task, returning the instrumented
/// runtime.
fn uninstall() -> InstrumentedRuntime {
    let mut task = Local::borrow(None::<Task>);
    let mut rt = task.maybe_take_runtime::<InstrumentedRuntime>().unwrap();
    rt.log(Event::Death);
    task.put_runtime(rt.inner.take().unwrap());
    *rt
}

impl InstrumentedRuntime {
    /// Puts this runtime back into the local task
    fn put(mut self: Box<InstrumentedRuntime>, event: Option<Event>) {
        assert!(self.inner.is_none());

        let mut task: Box<Task> = Local::take();
        let rt = task.take_runtime();
        self.inner = Some(rt);
        match event {
            Some(event) => self.log(event),
            None => {}
        }
        task.put_runtime(self);
        Local::put(task);
    }

    /// Logs a message into this runtime
    fn log(&mut self, event: Event) {
        self.messages.push(Message {
            timestamp: std_time::precise_time_ns(),
            event: event,
        });
    }

    fn task_id(&self) -> uint { self as *const _ as uint }
}

impl Runtime for InstrumentedRuntime {
    fn stack_guard(&self) -> Option<uint> {
        self.inner.as_ref().unwrap().stack_guard()
    }

    fn yield_now(mut self: Box<InstrumentedRuntime>, cur_task: Box<Task>) {
        self.inner.take().unwrap().yield_now(cur_task);
        self.put(None)
    }

    fn maybe_yield(mut self: Box<InstrumentedRuntime>, cur_task: Box<Task>) {
        self.inner.take().unwrap().maybe_yield(cur_task);
        self.put(None)
    }

    fn deschedule(mut self: Box<InstrumentedRuntime>, times: uint, cur_task: Box<Task>,
                  f: |BlockedTask| -> Result<(), BlockedTask>) {
        self.log(Event::Unschedule);
        self.inner.take().unwrap().deschedule(times, cur_task, f);
        self.put(Some(Event::Schedule));
    }

    fn reawaken(mut self: Box<InstrumentedRuntime>, to_wake: Box<Task>) {
        self.inner.take().unwrap().reawaken(to_wake);
        self.put(None);
    }

    fn spawn_sibling(mut self: Box<InstrumentedRuntime>,
                     cur_task: Box<Task>,
                     opts: TaskOpts,
                     f: proc():Send) {
        // Be sure to install an instrumented runtime for the spawned sibling by
        // specifying a new runtime.
        self.inner.take().unwrap().spawn_sibling(cur_task, opts, proc() {
            install();
            f();
            drop(uninstall());
        });
        self.put(None)
    }

    fn stack_bounds(&self) -> (uint, uint) { self.inner.as_ref().unwrap().stack_bounds() }
    fn can_block(&self) -> bool { self.inner.as_ref().unwrap().can_block() }
    fn wrap(self: Box<InstrumentedRuntime>) -> Box<Any+'static> { self as Box<Any> }
}
