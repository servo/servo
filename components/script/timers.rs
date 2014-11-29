/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::callback::ReportExceptions;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::js::JSRef;
use dom::bindings::utils::Reflectable;

use script_task::{FireTimerMsg, ScriptChan};
use script_task::{TimerSource, FromWindow, FromWorker};

use servo_util::task::spawn_named;

use js::jsval::JSVal;

use std::cell::Cell;
use std::cmp;
use std::collections::HashMap;
use std::comm::{channel, Sender};
use std::comm::Select;
use std::hash::{Hash, sip};
use std::io::timer::Timer;
use std::time::duration::Duration;

#[deriving(PartialEq, Eq)]
#[jstraceable]
pub struct TimerId(i32);

#[jstraceable]
#[privatize]
struct TimerHandle {
    handle: TimerId,
    data: TimerData,
    cancel_chan: Option<Sender<()>>,
}

impl Hash for TimerId {
    fn hash(&self, state: &mut sip::SipState) {
        let TimerId(id) = *self;
        id.hash(state);
    }
}

impl TimerHandle {
    fn cancel(&mut self) {
        self.cancel_chan.as_ref().map(|chan| chan.send_opt(()).ok());
    }
}

#[jstraceable]
#[privatize]
pub struct TimerManager {
    active_timers: DOMRefCell<HashMap<TimerId, TimerHandle>>,
    next_timer_handle: Cell<i32>,
}


#[unsafe_destructor]
impl Drop for TimerManager {
    fn drop(&mut self) {
        for (_, timer_handle) in self.active_timers.borrow_mut().iter_mut() {
            timer_handle.cancel();
        }
    }
}

// Enum allowing more descriptive values for the is_interval field
#[jstraceable]
#[deriving(PartialEq, Clone)]
pub enum IsInterval {
    Interval,
    NonInterval,
}

// Holder for the various JS values associated with setTimeout
// (ie. function value to invoke and all arguments to pass
//      to the function when calling it)
// TODO: Handle rooting during fire_timer when movable GC is turned on
#[jstraceable]
#[privatize]
#[deriving(Clone)]
struct TimerData {
    is_interval: IsInterval,
    funval: Function,
    args: Vec<JSVal>
}

impl TimerManager {
    pub fn new() -> TimerManager {
        TimerManager {
            active_timers: DOMRefCell::new(HashMap::new()),
            next_timer_handle: Cell::new(0)
        }
    }

    pub fn set_timeout_or_interval(&self,
                                  callback: Function,
                                  arguments: Vec<JSVal>,
                                  timeout: i32,
                                  is_interval: IsInterval,
                                  source: TimerSource,
                                  script_chan: ScriptChan)
                                  -> i32 {
        let timeout = cmp::max(0, timeout) as u64;
        let handle = self.next_timer_handle.get();
        self.next_timer_handle.set(handle + 1);

        // Spawn a new timer task; it will dispatch the FireTimerMsg
        // to the relevant script handler that will deal with it.
        let tm = Timer::new().unwrap();
        let (cancel_chan, cancel_port) = channel();
        let spawn_name = match source {
            FromWindow(_) if is_interval == IsInterval::Interval => "Window:SetInterval",
            FromWorker if is_interval == IsInterval::Interval => "Worker:SetInterval",
            FromWindow(_) => "Window:SetTimeout",
            FromWorker => "Worker:SetTimeout",
        };
        spawn_named(spawn_name, proc() {
            let mut tm = tm;
            let duration = Duration::milliseconds(timeout as i64);
            let timeout_port = if is_interval == IsInterval::Interval {
                tm.periodic(duration)
            } else {
                tm.oneshot(duration)
            };
            let cancel_port = cancel_port;

            let select = Select::new();
            let mut timeout_handle = select.handle(&timeout_port);
            unsafe { timeout_handle.add() };
            let mut cancel_handle = select.handle(&cancel_port);
            unsafe { cancel_handle.add() };

            loop {
                let id = select.wait();
                if id == timeout_handle.id() {
                    timeout_port.recv();
                    let ScriptChan(ref chan) = script_chan;
                    chan.send(FireTimerMsg(source, TimerId(handle)));
                    if is_interval == IsInterval::NonInterval {
                        break;
                    }
                } else if id == cancel_handle.id() {
                    break;
                }
            }
        });
        let timer_id = TimerId(handle);
        let timer = TimerHandle {
            handle: timer_id,
            cancel_chan: Some(cancel_chan),
            data: TimerData {
                is_interval: is_interval,
                funval: callback,
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
            None => { }
        }
    }

    pub fn fire_timer<T: Reflectable>(&self, timer_id: TimerId, this: JSRef<T>) {

        let data = match self.active_timers.borrow().get(&timer_id) {
            None => return,
            Some(timer_handle) => timer_handle.data.clone(),
        };

        // TODO: Must handle rooting of funval and args when movable GC is turned on
        let _ = data.funval.Call_(this, data.args, ReportExceptions);

        if data.is_interval == IsInterval::NonInterval {
            self.active_timers.borrow_mut().remove(&timer_id);
        }
    }
}

