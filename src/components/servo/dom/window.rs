/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::WrapperCache;
use dom::bindings::window;
use scripting::script_task::{ExitMsg, FireTimerMsg, ScriptMsg, ScriptContext};
use layout::layout_task::MatchSelectorsDamage;
use util::task::spawn_listener;

use core::comm::{Port, Chan, SharedChan};
use js::jsapi::JSVal;
use std::timer;
use std::uv_global_loop;

pub enum TimerControlMsg {
    TimerMessage_Fire(~TimerData),
    TimerMessage_Close,
    TimerMessage_TriggerExit //XXXjdm this is just a quick hack to talk to the script task
}

//FIXME If we're going to store the script task, find a way to do so safely. Currently it's
//      only used for querying layout from arbitrary script.
pub struct Window {
    timer_chan: Chan<TimerControlMsg>,
    script_chan: SharedChan<ScriptMsg>,
    script_context: *mut ScriptContext,
    wrapper: WrapperCache
}

impl Drop for Window {
    fn finalize(&self) {
        self.timer_chan.send(TimerMessage_Close);
    }
}

// Holder for the various JS values associated with setTimeout
// (ie. function value to invoke and all arguments to pass
//      to the function when calling it)
pub struct TimerData {
    funval: JSVal,
    args: ~[JSVal],
}

pub fn TimerData(argc: libc::c_uint, argv: *JSVal) -> TimerData {
    unsafe {
        let mut args = ~[];

        let mut i = 2;
        while i < argc as uint {
            args.push(*ptr::offset(argv, i));
            i += 1;
        };

        TimerData {
            funval : *argv,
            args : args,
        }
    }
}

// FIXME: delayed_send shouldn't require Copy
#[allow(non_implicitly_copyable_typarams)]
pub impl Window {
    fn alert(&self, s: &str) {
        // Right now, just print to the console
        io::println(fmt!("ALERT: %s", s));
    }

    fn close(&self) {
        self.timer_chan.send(TimerMessage_TriggerExit);
    }

    fn setTimeout(&self, timeout: int, argc: libc::c_uint, argv: *JSVal) {
        let timeout = int::max(0, timeout) as uint;

        // Post a delayed message to the per-window timer task; it will dispatch it
        // to the relevant script handler that will deal with it.
        timer::delayed_send(&uv_global_loop::get(),
                            timeout,
                            &self.timer_chan,
                            TimerMessage_Fire(~TimerData(argc, argv)));
    }

    fn content_changed(&self) {
        unsafe {
            (*self.script_context).trigger_relayout(MatchSelectorsDamage);
        }
    }

    pub fn new(script_chan: SharedChan<ScriptMsg>, script_context: *mut ScriptContext)
               -> @mut Window {
        let script_chan_copy = script_chan.clone();
        let win = @mut Window {
            wrapper: WrapperCache::new(),
            script_chan: script_chan,
            timer_chan: {
                do spawn_listener |timer_port: Port<TimerControlMsg>| {
                    loop {
                        match timer_port.recv() {
                            TimerMessage_Close => break,
                            TimerMessage_Fire(td) => script_chan_copy.send(FireTimerMsg(td)),
                            TimerMessage_TriggerExit => script_chan_copy.send(ExitMsg),
                        }
                    }
                }
            },
            script_context: script_context,
        };

        unsafe {
            let compartment = (*script_context).js_compartment;
            window::create(compartment, win);
        }
        win
    }
}

