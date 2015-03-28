use msg::constellation_msg::PipelineId;
use rustc_serialize::json;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::net::TcpStream;
use std::old_io::timer::sleep;
use std::sync::Arc;
use std::thread;
use std::time::duration::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use time::precise_time_ns;

use actor::{Actor, ActorRegistry};
use actors::memory::{MemoryActor, TimelineMemoryReply};
use actors::framerate::FramerateActor;
use devtools_traits::DevtoolScriptControlMsg;
use devtools_traits::DevtoolScriptControlMsg::{SetTimelineMarker, DropTimelineMarker};
use devtools_traits::{TimelineMarker, TracingMetadata, TimelineMarkerType};
use protocol::JsonPacketStream;

pub struct TimelineActor {
    name: String,
    script_sender: Sender<DevtoolScriptControlMsg>,
    marker_types: Vec<TimelineMarkerType>,
    pipeline: PipelineId,
    isRecording: Arc<AtomicBool>,
    stream: RefCell<Option<TcpStream>>,
}

struct Emitter {
    from: String,
    stream: RefCell<TcpStream>,
    framerateActor: RefCell<Option<FramerateActor>>,
    markers: RefCell<Vec<TimelineMarkerReply>>,
    memoryActor: RefCell<Option<MemoryActor>>,
}

#[derive(RustcEncodable)]
struct IsRecordingReply {
    from: String,
    value: bool
}

#[derive(RustcEncodable)]
struct StartReply {
    from: String,
    value: u64
}

#[derive(RustcEncodable)]
struct StopReply {
    from: String,
    value: u64
}

#[derive(RustcEncodable)]
struct TimelineMarkerReply {
    name: String,
    start: u64,
    end: u64,
    stack: Option<Vec<uint>>,
    endStack: Option<Vec<uint>>,
}

#[derive(RustcEncodable)]
struct MarkersEmitterReply {
    __type__: String,
    markers: RefCell<Vec<TimelineMarkerReply>>,
    from: String,
    endTime: u64,
}

#[derive(RustcEncodable)]
struct MemoryEmitterReply {
    __type__: String,
    from: String,
    delta: u64,
    measurement: TimelineMemoryReply,
}

#[derive(RustcEncodable)]
struct FramerateEmitterReply {
    __type__: String,
    from: String,
    delta: u64,
    timestamps: Vec<u64>,
}

static DEFAULT_TIMELINE_DATA_PULL_TIMEOUT: uint = 200; //ms

impl TimelineActor {
    pub fn new(name: String,
               pipeline: PipelineId,
               script_sender: Sender<DevtoolScriptControlMsg>) -> TimelineActor {

        let mut marker_types = vec!(TimelineMarkerType::Reflow,
                                    TimelineMarkerType::DOMEvent);

        TimelineActor {
            name: name,
            pipeline: pipeline,
            marker_types: marker_types,
            script_sender: script_sender,
            isRecording: Arc::new(AtomicBool::new(false)),
            stream: RefCell::new(None),
        }
    }

    fn pullTimelineData(&self, receiver: Receiver<TimelineMarker>, emitter: Emitter) {
        let isRecording = self.isRecording.clone();

        if !isRecording.load(Ordering::Relaxed) {
            return;
        }

        /// Select root(with depth 0) TimelineMarker pair (IntervalStart + IntervalEnd)
        /// from queue and add marker to emitter
        /// Return true if closed (IntervalStart + IntervalEnd) pair was founded
        fn group(queue: &mut VecDeque<TimelineMarker>, depth: uint,
                        start_payload: Option<TimelineMarker>, emitter: &Emitter) -> bool {

            match start_payload {
                Some(start_payload) => {
                    if start_payload.metadata != TracingMetadata::IntervalStart {
                        panic!("Start payload doesn't have metadata IntervalStart");
                    }
                    match queue.pop_front() {
                        Some(end_payload) => {
                            match end_payload.metadata {
                                TracingMetadata::IntervalEnd => {
                                    if depth == 0 {
                                        // Emit TimelineMarkerReply, pair was founded
                                        emitter.add_marker(start_payload, end_payload);
                                    }
                                    return true;
                                }
                                TracingMetadata::IntervalStart => {
                                    if group(queue, depth + 1, Some(end_payload), emitter) {
                                        return group(queue, depth, Some(start_payload), emitter);
                                    } else {
                                        queue.push_front(start_payload);
                                    }
                                }
                                _ => panic!("Unknown tracingMetadata")
                            }
                        },
                        None => {
                            queue.push_front(start_payload);
                        },
                    }
                },
                None => (),
            };
            false
        }

        thread::spawn(move || {
            let mut queues = HashMap::new();
            queues.insert("Reflow".to_string(), VecDeque::new());
            queues.insert("DOMEvent".to_string(), VecDeque::new());

            loop {
                if !isRecording.load(Ordering::Relaxed) {
                    break;
                }

                // Creating queues by marker.name
                loop {
                    match receiver.try_recv() {
                        Ok(marker) => {
                            match queues.get_mut(&marker.name) {
                                Some(list) => list.push_back(marker),
                                None => ()
                            }
                        }

                        Err(TryRecvError) => break
                    }
                }

                // Emit all markers
                for (name, queue) in queues.iter_mut() {
                    let start_payload = queue.pop_front();
                    group(queue, 0, start_payload, &emitter);
                }
                emitter.send();

                sleep(Duration::milliseconds(DEFAULT_TIMELINE_DATA_PULL_TIMEOUT as i64));
            }
        });
    }
}

impl Actor for TimelineActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &str,
                      msg: &json::Object,
                      stream: &mut TcpStream) -> Result<bool, ()> {
        Ok(match msg_type {
            "start" => {
                self.isRecording.store(true, Ordering::Relaxed);

                let (tx, rx) = channel::<TimelineMarker>();
                for marker_type in self.marker_types.iter() {
                    self.script_sender.send(SetTimelineMarker(self.pipeline, marker_type.clone(), tx.clone()));
                }

                *self.stream.borrow_mut() = stream.try_clone().ok();
                let emitter = Emitter {
                    from: self.name(),
                    stream: RefCell::new(stream.try_clone().ok().unwrap()),
                    framerateActor: RefCell::new(None),
                    markers: RefCell::new(Vec::new()),
                    memoryActor: RefCell::new(None),
                };

                // init memoryActor
                if msg.contains_key("withMemory") {
                    *emitter.memoryActor.borrow_mut() = Some(MemoryActor::new(registry));
                }

                // init framerateActor
                if msg.contains_key("withTicks") {
                    let framerateActor = FramerateActor::new(registry);
                    framerateActor.startRecording();
                    *emitter.framerateActor.borrow_mut() = Some(framerateActor);
                }

                self.pullTimelineData(rx, emitter);

                let msg = StartReply {
                    from: self.name(),
                    value: precise_time_ns(),
                };
                stream.write_json_packet(&msg);
                true
            }

            "stop" => {
                let msg = StopReply {
                    from: self.name(),
                    value: precise_time_ns()
                };

                stream.write_json_packet(&msg);
                for marker_type in self.marker_types.iter() {
                    self.script_sender.send(DropTimelineMarker(self.pipeline, marker_type.clone()));
                }

                self.isRecording.store(false, Ordering::Relaxed);
                *self.stream.borrow_mut() = None;
                true
            }

            "isRecording" => {
                let msg = IsRecordingReply {
                    from: self.name(),
                    value: self.isRecording.load(Ordering::Relaxed)
                };

                stream.write_json_packet(&msg);
                true
            }

            _ => {
                false
            }
        })
    }
}

impl Emitter {
    fn add_marker(&self, start_payload: TimelineMarker, end_payload: TimelineMarker) -> () {
        self.markers.borrow_mut().push(TimelineMarkerReply {
            name: start_payload.name,
            start: start_payload.time,
            end: end_payload.time,
            stack: start_payload.stack,
            endStack: end_payload.stack,
        });
    }

    fn send(&self) -> () {
        let endTime = precise_time_ns();
        let reply = MarkersEmitterReply {
            __type__: "markers".to_string(),
            markers: RefCell::new(Vec::new()),
            from: self.from.clone(),
            endTime: endTime,
        };

        let mut markers = self.markers.borrow_mut();
        for marker in markers.drain() {
            reply.markers.borrow_mut().push(marker);
        }
        self.stream.borrow_mut().write_json_packet(&reply);

        match *self.framerateActor.borrow() {
            Some(ref framerateActor) => {
                let framerateReply = FramerateEmitterReply {
                    __type__: "framerate".to_string(),
                    from: self.from.clone(),
                    delta: endTime,
                    timestamps: framerateActor.get_pending_ticks(),
                };
                self.stream.borrow_mut().write_json_packet(&framerateReply);
            }
            None => ()
        }

        match *self.memoryActor.borrow() {
            Some(ref memoryActor) => {
                let memoryReply = MemoryEmitterReply {
                    __type__: "memory".to_string(),
                    from: self.from.clone(),
                    delta: endTime,
                    measurement: memoryActor.measure(),
                };
                self.stream.borrow_mut().write_json_packet(&memoryReply);
            }
            None => ()
        }
    }
}

impl Drop for Emitter {
    fn drop(&mut self) {
        match *self.framerateActor.borrow() {
            Some(ref framerateActor) =>  framerateActor.stopRecording(),
            None => ()
        }
    }
}
