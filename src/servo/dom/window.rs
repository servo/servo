use comm::{Port, Chan};
use content::content_task::{ControlMsg, Timer};

enum TimerControlMsg {
    TimerMessage_Fire(~dom::bindings::window::TimerData),
    TimerMessage_Close
}

struct Window {
    timer_chan: Chan<TimerControlMsg>,

    drop {
        self.timer_chan.send(TimerMessage_Close);
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
                }
            }
        }
    }
}
