/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::WrapperCache;
use dom::bindings::window;

use layout_interface::ReflowForScriptQuery;
use script_task::{ExitMsg, FireTimerMsg, Page, ScriptChan};
use servo_msg::compositor_msg::ScriptListener;

use std::comm;
use std::comm::Chan;
use std::libc;
use std::int;
use std::io;
use std::ptr;
use js::jsapi::JSVal;
use extra::timer;
use extra::uv_global_loop;

pub enum TimerControlMsg {
    TimerMessage_Fire(~TimerData),
    TimerMessage_Close,
    TimerMessage_TriggerExit //XXXjdm this is just a quick hack to talk to the script task
}

//FIXME If we're going to store the page, find a way to do so safely.
pub struct Window {
    page: *mut Page,
    script_chan: ScriptChan,
    compositor: @ScriptListener,
    wrapper: WrapperCache,
    timer_chan: Chan<TimerControlMsg>,
}

#[unsafe_destructor]
impl Drop for Window {
    fn drop(&self) {
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
impl Window {
    pub fn alert(&self, s: &str) {
        // Right now, just print to the console
        io::println(fmt!("ALERT: %s", s));
    }

    pub fn close(&self) {
        self.timer_chan.send(TimerMessage_TriggerExit);
    }

    pub fn setTimeout(&self, timeout: int, argc: libc::c_uint, argv: *JSVal) {
        let timeout = int::max(0, timeout) as uint;

        // Post a delayed message to the per-window timer task; it will dispatch it
        // to the relevant script handler that will deal with it.
        timer::delayed_send(&uv_global_loop::get(),
                            timeout,
                            &self.timer_chan,
                            TimerMessage_Fire(~TimerData(argc, argv)));
    }

    pub fn content_changed(&self) {
        unsafe {
            (*self.page).reflow_all(ReflowForScriptQuery, self.script_chan.clone(), self.compositor);
        }
    }

    pub fn new(page: *mut Page, script_chan: ScriptChan, compositor: @ScriptListener)
               -> @mut Window {
        let script_chan_clone = script_chan.clone();
        let win = @mut Window {
            page: page,
            script_chan: script_chan,
            compositor: compositor,
            wrapper: WrapperCache::new(),
            timer_chan: {
                let (timer_port, timer_chan) = comm::stream::<TimerControlMsg>();
                do spawn {
                    loop {
                        match timer_port.recv() {
                            TimerMessage_Close => break,
                            TimerMessage_Fire(td) => unsafe {script_chan_clone.chan.send(FireTimerMsg((*page).id.clone(), td))},
                            TimerMessage_TriggerExit => script_chan_clone.chan.send(ExitMsg),
                        }
                    }
                }
                timer_chan
            },
        };

        unsafe {
            // TODO(tkuehn): This just grabs the top-level page. Need to handle subframes.
            let compartment = (*page).js_info.get_ref().js_compartment;
            window::create(compartment, win);
        }
        win
    }
}

