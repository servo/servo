/*!
A little class that rate limits the number of resize events sent to the content task
based on how fast content dispatches those events. It waits until each event is handled
before sending the next. If the window is resized multiple times before an event is handled
then some events will never be sent.
*/

use dom::event::{Event, ResizeEvent};

pub struct ResizeRateLimiter {
    /// The channel we send resize events on
    /* priv */ dom_event_chan: pipes::SharedChan<Event>,
    /// The port we are waiting on for a response to the last resize event
    /* priv */ mut last_response_port: Option<pipes::Port<()>>,
    /// The next window resize event we should fire
    /* priv */ mut next_resize_event: Option<(uint, uint)>
}

pub fn ResizeRateLimiter(dom_event_chan: pipes::SharedChan<Event>) -> ResizeRateLimiter {
    ResizeRateLimiter {
        dom_event_chan: move dom_event_chan,
        last_response_port: None,
        next_resize_event: None
    }
}

impl ResizeRateLimiter {
    fn window_resized(width: uint, height: uint) {
        match self.last_response_port {
            None => {
                assert self.next_resize_event.is_none();
                self.send_event(width, height);
            }
            Some(*) => {
                if self.last_response_port.get_ref().peek() {
                    self.send_event(width, height);
                    self.next_resize_event = None;
                } else {
                    if self.next_resize_event.is_some() {
                        warn!("osmain: content can't keep up. skipping resize event");
                    }
                    self.next_resize_event = Some((width, height));
                }
            }
        }
    }

    fn check_resize_response() {
        match self.next_resize_event {
            Some((copy width, copy height)) => {
                assert self.last_response_port.is_some();
                if self.last_response_port.get_ref().peek() {
                    self.send_event(width, height);
                    self.next_resize_event = None;
                }
            }
            None => ()
        }
    }

    priv fn send_event(width: uint, height: uint) {
        let (port, chan) = pipes::stream();
        self.dom_event_chan.send(ResizeEvent(width, height, move chan));
        self.last_response_port = Some(move port);
    }
}
