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

use util::task::spawn_named;
use util::str::DOMString;

use js::jsval::JSVal;

use std::borrow::ToOwned;
use std::cell::Cell;
use std::cmp;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::sync::mpsc::Select;
use std::hash::{Hash, Hasher, Writer};
use std::old_io::timer::Timer;
use std::time::duration::Duration;

#[derive(PartialEq, Eq)]
#[jstraceable]
#[derive(Copy)]
pub struct TimerId(i32);

#[jstraceable]
#[privatize]
struct TimerHandle {
    handle: TimerId,
    data: TimerData,
    cancel_chan: Option<Sender<()>>,
}

#[jstraceable]
#[derive(Clone)]
pub enum TimerCallback {
    StringTimerCallback(DOMString),
    FunctionTimerCallback(Function)
}

impl<H: Writer + Hasher> Hash<H> for TimerId {
    fn hash(&self, state: &mut H) {
        let TimerId(id) = *self;
        id.hash(state);
    }
}

impl TimerHandle {
    fn cancel(&mut self) {
        self.cancel_chan.as_ref().map(|chan| chan.send(()).ok());
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
#[derive(PartialEq, Copy, Clone)]
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

    #[allow(unsafe_blocks)]
    pub fn set_timeout_or_interval(&self,
                                  callback: TimerCallback,
                                  arguments: Vec<JSVal>,
                                  timeout: i32,
                                  is_interval: IsInterval,
                                  source: TimerSource,
                                  script_chan: Box<ScriptChan+Send>)
                                  -> i32 {
        let timeout = cmp::max(0, timeout) as u64;
        let handle = self.next_timer_handle.get();
        self.next_timer_handle.set(handle + 1);

        // Spawn a new timer task; it will dispatch the `ScriptMsg::FireTimer`
        // to the relevant script handler that will deal with it.
        let tm = Timer::new().unwrap();
        let (cancel_chan, cancel_port) = channel();
        let spawn_name = match source {
            TimerSource::FromWindow(_) if is_interval == IsInterval::Interval => "Window:SetInterval",
            TimerSource::FromWorker if is_interval == IsInterval::Interval => "Worker:SetInterval",
            TimerSource::FromWindow(_) => "Window:SetTimeout",
            TimerSource::FromWorker => "Worker:SetTimeout",
        }.to_owned();
        spawn_named(spawn_name, move || {
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
                    timeout_port.recv().unwrap();
                    script_chan.send(ScriptMsg::FireTimer(source, TimerId(handle)));
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
                this.evaluate_js_on_global_with_result(code_str.as_slice());
            }
        };

        if data.is_interval == IsInterval::NonInterval {
            self.active_timers.borrow_mut().remove(&timer_id);
        }
    }
}

