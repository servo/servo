use comm::{Port, Chan};
use content::content_task::{ControlMsg, Timer, ExitMsg};
use js::jsapi::jsval;
use dvec::DVec;

enum TimerControlMsg {
    TimerMessage_Fire(~TimerData),
    TimerMessage_Close,
    TimerMessage_TriggerExit //XXXjdm this is just a quick hack to talk to the content task
}

struct Window {
    timer_chan: Chan<TimerControlMsg>,

    drop {
        self.timer_chan.send(TimerMessage_Close);
    }
}

// Holder for the various JS values associated with setTimeout
// (ie. function value to invoke and all arguments to pass
//      to the function when calling it)
struct TimerData {
    funval: jsval,
    args: DVec<jsval>,
}

fn TimerData(argc: libc::c_uint, argv: *jsval) -> TimerData unsafe {
    let data = TimerData {
        funval : *argv,
        args : DVec(),
    };

    let mut i = 2;
    while i < argc as uint {
        data.args.push(*ptr::offset(argv, i));
        i += 1;
    };

    data
}

impl Window {
    fn alert(s: &str) {
        // Right now, just print to the console
        io::println(#fmt("ALERT: %s", s));
    }

    fn close() {
        self.timer_chan.send(TimerMessage_TriggerExit);
    }

    fn setTimeout(&self, timeout: int, argc: libc::c_uint, argv: *jsval) {
        let timeout = int::max(0, timeout) as uint;

        // Post a delayed message to the per-window timer task; it will dispatch it
        // to the relevant content handler that will deal with it.
        std::timer::delayed_send(std::uv_global_loop::get(),
                                 timeout, self.timer_chan,
                                 TimerMessage_Fire(~TimerData(argc, argv)));
    }
}

fn Window(content_port: Port<ControlMsg>) -> Window {
    let content_chan = Chan(content_port);
        
    Window {
        timer_chan: do task::spawn_listener |timer_port: Port<TimerControlMsg>| {
            loop {
                match timer_port.recv() {
                    TimerMessage_Close => break,
                    TimerMessage_Fire(td) => {
                        content_chan.send(Timer(copy td));
                    }
                    TimerMessage_TriggerExit => content_chan.send(ExitMsg)
                }
            }
        }
    }
}
