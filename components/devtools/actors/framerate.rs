use rustc_serialize::json;
use std::cell::RefCell;
use std::mem;
use std::net::TcpStream;
use time::precise_time_ns;

use actor::{Actor, ActorRegistry};
use protocol::JsonPacketStream;

pub struct FramerateActor {
    name: String,
    startTime: RefCell<Option<u64>>,
    isRecording: RefCell<bool>,
    ticks: RefCell<Vec<u64>>,
}

impl Actor for FramerateActor {
    fn name(&self) -> String {
        self.name.clone()
    }


    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &str,
                      msg: &json::Object,
                      stream: &mut TcpStream) -> Result<bool, ()> {
        Ok(true)
    }
}

impl FramerateActor {
    pub fn new(registry: &ActorRegistry) -> FramerateActor {
        FramerateActor {
            name: registry.new_name("framerate"),
            startTime: RefCell::new(None),
            isRecording: RefCell::new(false),
            ticks: RefCell::new(Vec::new()),
        }
    }

    pub fn startRecording(&self) {
        let currentTime = precise_time_ns();
        *self.isRecording.borrow_mut() = true;
        *self.startTime.borrow_mut() = Some(currentTime);

        //TODO: request animation frame
    }

    pub fn stopRecording(&self) {
        if !self.isRecording() {
            return;
        }
        *self.isRecording.borrow_mut() = false;
        *self.startTime.borrow_mut() = None;
    }

    pub fn isRecording(&self) -> bool {
        *self.isRecording.borrow()
    }

    // callback on request animation frame
    pub fn on_refresh_driver_tick(&self) {
        if !self.isRecording() {
            return;
        }
        //TODO: request animation frame

        let startTime = self.startTime.borrow().unwrap();
        self.ticks.borrow_mut().push(startTime - precise_time_ns());
    }

    pub fn get_pending_ticks(&self) -> Vec<u64> {
        let mut ticks = self.ticks.borrow_mut();
        mem::replace(&mut *ticks, Vec::new())
    }
}
