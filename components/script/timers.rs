/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::callback::ExceptionHandling::Report;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::js::JSRef;
use dom::bindings::utils::Reflectable;

use dom::window::ScriptHelpers;

use script_task::{ScriptChan, ScriptMsg, TimerSource};
use horribly_inefficient_timers;

use util::task::spawn_named;
use util::str::DOMString;

use js::jsval::JSVal;

use std::borrow::ToOwned;
use std::cell::Cell;
use std::cmp;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::sync::mpsc::Select;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq)]
#[jstraceable]
#[derive(Copy, Clone)]
pub struct TimerId(i32);

#[jstraceable]
#[privatize]
struct TimerHandle {
    handle: TimerId,
    data: TimerData,
    control_chan: Option<Sender<TimerControlMsg>>,
}

#[jstraceable]
#[derive(Clone)]
pub enum TimerCallback {
    StringTimerCallback(DOMString),
    FunctionTimerCallback(Function)
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

#[jstraceable]
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
#[jstraceable]
#[derive(PartialEq, Copy, Clone)]
pub enum IsInterval {
    Interval,
    NonInterval,
}

// Messages sent control timers from script task
#[jstraceable]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TimerControlMsg {
    Cancel,
    Suspend,
    Resume
}

// Holder for the various JS values associated with setTimeout
// (ie. function value to invoke and all arguments to pass
//      to the function when calling it)
// TODO: Handle rooting during fire_timer when movable GC is turned on
#[jstraceable]
#[privatize]
#[derive(Clone)]
struct TimerData {
    is_interval: IsInterval,
    callback: TimerCallback,
    args: Vec<JSVal>
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
                                  arguments: Vec<JSVal>,
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
                args: arguments
            }
        };
        self.active_timers.borrow_mut().insert(timer_id, timer);
        handle
    }

    pub fn clear_timeout_or_interval(&self, handle: i32) {
        let mut timer_handle = self.active_timers.borrow_mut().remove(&TimerId(handle));
        match timer_handle {
            Some(ref mut handle) => handle.cancel(),
            None => {}
        }
    }

    pub fn fire_timer<T: Reflectable>(&self, timer_id: TimerId, this: JSRef<T>) {

        let data = match self.active_timers.borrow().get(&timer_id) {
            None => return,
            Some(timer_handle) => timer_handle.data.clone(),
        };

        // TODO: Must handle rooting of funval and args when movable GC is turned on
        match data.callback {
            TimerCallback::FunctionTimerCallback(function) => {
                let _ = function.Call_(this, data.args, Report);
            }
            TimerCallback::StringTimerCallback(code_str) => {
                this.evaluate_js_on_global_with_result(&code_str);
            }
        };

        if data.is_interval == IsInterval::NonInterval {
            self.active_timers.borrow_mut().remove(&timer_id);
        }
    }
}

