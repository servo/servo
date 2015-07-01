/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::callback::ExceptionHandling::Report;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::global::global_object_for_js_object;
use dom::bindings::utils::Reflectable;

use dom::window::ScriptHelpers;

use script_task::{ScriptChan, ScriptMsg, TimerSource};
use horribly_inefficient_timers;

use util::task::spawn_named;
use util::str::DOMString;

use js::jsapi::{RootedValue, HandleValue, Heap};
use js::jsval::{JSVal, UndefinedValue};

use std::borrow::ToOwned;
use std::cell::Cell;
use std::cmp;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::sync::mpsc::Select;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::default::Default;

#[derive(JSTraceable, PartialEq, Eq, Copy, Clone)]
pub struct TimerId(i32);

#[derive(JSTraceable)]
#[privatize]
struct TimerHandle {
    handle: TimerId,
    data: TimerData,
    control_chan: Option<Sender<TimerControlMsg>>,
}

#[derive(JSTraceable, Clone)]
pub enum TimerCallback {
    StringTimerCallback(DOMString),
    FunctionTimerCallback(Rc<Function>)
}

impl Hash for TimerId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let TimerId(id) = *self;
        id.hash(state);
    }
}

impl TimerHandle {
    fn cancel(&mut self) {
        self.control_chan.as_ref().map(|chan| chan.send(TimerControlMsg::Cancel).ok());
    }
    fn suspend(&mut self) {
        self.control_chan.as_ref().map(|chan| chan.send(TimerControlMsg::Suspend).ok());
    }
    fn resume(&mut self) {
        self.control_chan.as_ref().map(|chan| chan.send(TimerControlMsg::Resume).ok());
    }
}

#[derive(JSTraceable)]
#[privatize]
pub struct TimerManager {
    active_timers: DOMRefCell<HashMap<TimerId, TimerHandle>>,
    next_timer_handle: Cell<i32>,
}


impl Drop for TimerManager {
    fn drop(&mut self) {
        for (_, timer_handle) in self.active_timers.borrow_mut().iter_mut() {
            timer_handle.cancel();
        }
    }
}

// Enum allowing more descriptive values for the is_interval field
#[derive(JSTraceable, PartialEq, Copy, Clone)]
pub enum IsInterval {
    Interval,
    NonInterval,
}

// Messages sent control timers from script task
#[derive(JSTraceable, PartialEq, Copy, Clone, Debug)]
pub enum TimerControlMsg {
    Cancel,
    Suspend,
    Resume
}

// Holder for the various JS values associated with setTimeout
// (ie. function value to invoke and all arguments to pass
//      to the function when calling it)
// TODO: Handle rooting during fire_timer when movable GC is turned on
#[derive(JSTraceable)]
#[privatize]
struct TimerData {
    is_interval: IsInterval,
    callback: TimerCallback,
    args: Vec<Heap<JSVal>>
}

impl TimerManager {
    pub fn new() -> TimerManager {
        TimerManager {
            active_timers: DOMRefCell::new(HashMap::new()),
            next_timer_handle: Cell::new(0)
        }
    }

    pub fn suspend(&self) {
        for (_, timer_handle) in self.active_timers.borrow_mut().iter_mut() {
            timer_handle.suspend();
        }
    }
    pub fn resume(&self) {
        for (_, timer_handle) in self.active_timers.borrow_mut().iter_mut() {
            timer_handle.resume();
        }
    }

    #[allow(unsafe_code)]
    pub fn set_timeout_or_interval(&self,
                                  callback: TimerCallback,
                                  arguments: Vec<HandleValue>,
                                  timeout: i32,
                                  is_interval: IsInterval,
                                  source: TimerSource,
                                  script_chan: Box<ScriptChan+Send>)
                                  -> i32 {
        let duration_ms = cmp::max(0, timeout) as u32;
        let handle = self.next_timer_handle.get();
        self.next_timer_handle.set(handle + 1);

        // Spawn a new timer task; it will dispatch the `ScriptMsg::FireTimer`
        // to the relevant script handler that will deal with it.
        let (control_chan, control_port) = channel();
        let spawn_name = match source {
            TimerSource::FromWindow(_) if is_interval == IsInterval::Interval => "Window:SetInterval",
            TimerSource::FromWorker if is_interval == IsInterval::Interval => "Worker:SetInterval",
            TimerSource::FromWindow(_) => "Window:SetTimeout",
            TimerSource::FromWorker => "Worker:SetTimeout",
        }.to_owned();
        spawn_named(spawn_name, move || {
            let timeout_port = if is_interval == IsInterval::Interval {
                horribly_inefficient_timers::periodic(duration_ms)
            } else {
                horribly_inefficient_timers::oneshot(duration_ms)
            };
            let control_port = control_port;

            let select = Select::new();
            let mut timeout_handle = select.handle(&timeout_port);
            unsafe { timeout_handle.add() };
            let mut control_handle = select.handle(&control_port);
            unsafe { control_handle.add() };

            loop {
                let id = select.wait();

                if id == timeout_handle.id() {
                    timeout_port.recv().unwrap();
                    if script_chan.send(ScriptMsg::FireTimer(source, TimerId(handle))).is_err() {
                        break;
                    }

                    if is_interval == IsInterval::NonInterval {
                        break;
                    }
                } else if id == control_handle.id() {;
                    match control_port.recv().unwrap() {
                        TimerControlMsg::Suspend => {
                            let msg = control_port.recv().unwrap();
                            match msg {
                                TimerControlMsg::Suspend => panic!("Nothing to suspend!"),
                                TimerControlMsg::Resume => {},
                                TimerControlMsg::Cancel => {
                                    break;
                                },
                            }
                            },
                        TimerControlMsg::Resume => panic!("Nothing to resume!"),
                        TimerControlMsg::Cancel => {
                            break;
                        }
                    }
                }
            }
        });
        let timer_id = TimerId(handle);
        let timer = TimerHandle {
            handle: timer_id,
            control_chan: Some(control_chan),
            data: TimerData {
                is_interval: is_interval,
                callback: callback,
                args: Vec::with_capacity(arguments.len())
            }
        };
        self.active_timers.borrow_mut().insert(timer_id, timer);

        // This is a bit complicated, but this ensures that the vector's
        // buffer isn't reallocated (and moved) after setting the Heap values
        let mut timers = self.active_timers.borrow_mut();
        let mut timer = timers.get_mut(&timer_id).unwrap();
        for _ in 0..arguments.len() {
            timer.data.args.push(Heap::default());
        }
        for i in 0..arguments.len() {
            timer.data.args.get_mut(i).unwrap().set(arguments[i].get());
        }
        handle
    }

    pub fn clear_timeout_or_interval(&self, handle: i32) {
        let mut timer_handle = self.active_timers.borrow_mut().remove(&TimerId(handle));
        match timer_handle {
            Some(ref mut handle) => handle.cancel(),
            None => {}
        }
    }

    pub fn fire_timer<T: Reflectable>(&self, timer_id: TimerId, this: &T) {

        let (is_interval, callback, args): (IsInterval, TimerCallback, Vec<JSVal>) =
            match self.active_timers.borrow().get(&timer_id) {
                Some(timer_handle) =>
                    (timer_handle.data.is_interval,
                     timer_handle.data.callback.clone(),
                     timer_handle.data.args.iter().map(|arg| arg.get()).collect()),
                None => return,
            };

        match callback {
            TimerCallback::FunctionTimerCallback(function) => {
                let arg_handles = args.iter().by_ref().map(|arg| HandleValue { ptr: arg }).collect();
                let _ = function.Call_(this, arg_handles, Report);
            }
            TimerCallback::StringTimerCallback(code_str) => {
                let proxy = this.reflector().get_jsobject();
                let cx = global_object_for_js_object(proxy.get()).r().get_cx();
                let mut rval = RootedValue::new(cx, UndefinedValue());
                this.evaluate_js_on_global_with_result(&code_str, rval.handle_mut());
            }
        }

        if is_interval == IsInterval::NonInterval {
            self.active_timers.borrow_mut().remove(&timer_id);
        }
    }
}

