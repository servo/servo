use content::content_task::{ControlMsg, Timer, ExitMsg};
use dom::bindings::utils::WrapperCache;
use js::jsapi::{JSVal, JSObject};
use util::task::spawn_listener;

use core::comm::{Port, Chan};
use std::timer;
use std::uv_global_loop;

pub enum TimerControlMsg {
    TimerMessage_Fire(~TimerData),
    TimerMessage_Close,
    TimerMessage_TriggerExit //XXXjdm this is just a quick hack to talk to the content task
}

pub struct Window {
    timer_chan: Chan<TimerControlMsg>,
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
        // to the relevant content handler that will deal with it.
        timer::delayed_send(&uv_global_loop::get(),
                            timeout,
                            &self.timer_chan,
                            TimerMessage_Fire(~TimerData(argc, argv)));
    }
}

pub fn Window(content_chan: comm::SharedChan<ControlMsg>) -> Window {
        
    Window {
        wrapper: WrapperCache::new(),
        timer_chan: do spawn_listener |timer_port: Port<TimerControlMsg>| {
            loop {
                match timer_port.recv() {
                    TimerMessage_Close => break,
                    TimerMessage_Fire(td) => {
                        content_chan.send(Timer(td));
                    }
                    TimerMessage_TriggerExit => content_chan.send(ExitMsg)
                }
            }
        }
    }
}
