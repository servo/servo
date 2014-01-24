use std::comm::{Port, SharedChan};
use render_task::Msg;

pub struct RenderChan<T> {
    chan: SharedChan<Msg<T>>,
}

impl<T: Send> Clone for RenderChan<T> {
    fn clone(&self) -> RenderChan<T> {
        RenderChan {
            chan: self.chan.clone(),
        }
    }
}

impl<T: Send> RenderChan<T> {
    pub fn new() -> (Port<Msg<T>>, RenderChan<T>) {
        let (port, chan) = SharedChan::new();
        let render_chan = RenderChan {
            chan: chan,
        };
        (port, render_chan)
    }

    pub fn send(&self, msg: Msg<T>) {
        assert!(self.try_send(msg), "RenderChan.send: render port closed")
    }

    pub fn try_send(&self, msg: Msg<T>) -> bool {
        self.chan.try_send(msg)
    }
}
