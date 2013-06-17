/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A little class that rate limits the number of resize events sent to the script task
/// based on how fast script dispatches those events. It waits until each event is handled
/// before sending the next. If the window is resized multiple times before an event is handled
/// then some events will never be sent.

use core::comm::{Port};
use script::dom::event::ResizeEvent;
use script::script_task::{ScriptChan, ScriptMsg, SendEventMsg};

pub struct ResizeRateLimiter {
    /// The channel we send resize events on
    priv script_chan: ScriptChan,
    /// The port we are waiting on for a response to the last resize event
    priv last_response_port: Option<Port<()>>,
    /// The next window resize event we should fire
    priv next_resize_event: Option<(uint, uint)>
}

pub fn ResizeRateLimiter(script_chan: ScriptChan) -> ResizeRateLimiter {
    ResizeRateLimiter {
        script_chan: script_chan,
        last_response_port: None,
        next_resize_event: None
    }
}

pub impl ResizeRateLimiter {
    fn window_resized(&mut self, width: uint, height: uint) {
        match self.last_response_port {
            None => {
                assert!(self.next_resize_event.is_none());
                self.send_event(width, height);
            }
            Some(*) => {
                if self.last_response_port.get_ref().peek() {
                    self.send_event(width, height);
                    self.next_resize_event = None;
                } else {
                    if self.next_resize_event.is_some() {
                        warn!("osmain: script task can't keep up. skipping resize event");
                    }
                    self.next_resize_event = Some((width, height));
                }
            }
        }
    }

    fn check_resize_response(&mut self) {
        match self.next_resize_event {
            Some((copy width, copy height)) => {
                assert!(self.last_response_port.is_some());
                if self.last_response_port.get_ref().peek() {
                    self.send_event(width, height);
                    self.next_resize_event = None;
                }
            }
            None => ()
        }
    }

    priv fn send_event(&mut self, width: uint, height: uint) {
        let (port, chan) = comm::stream();
        self.script_chan.send(SendEventMsg(ResizeEvent(width, height, chan)));
        self.last_response_port = Some(port);
    }
}
