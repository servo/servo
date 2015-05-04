/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::PipelineId;
use rustc_serialize::{json, Encoder, Encodable};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::mem;
use std::net::TcpStream;
use std::thread::sleep_ms;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use time::PreciseTime;

use actor::{Actor, ActorRegistry};
use actors::memory::{MemoryActor, TimelineMemoryReply};
use actors::framerate::FramerateActor;
use devtools_traits::DevtoolScriptControlMsg;
use devtools_traits::DevtoolScriptControlMsg::{SetTimelineMarkers, DropTimelineMarkers};
use devtools_traits::{TimelineMarker, TracingMetadata, TimelineMarkerType};
use protocol::JsonPacketStream;
use util::task;

pub struct TimelineActor {
    name: String,
    script_sender: Sender<DevtoolScriptControlMsg>,
    marker_types: Vec<TimelineMarkerType>,
    pipeline: PipelineId,
    is_recording: Arc<Mutex<bool>>,
    stream: RefCell<Option<TcpStream>>,

    framerate_actor: RefCell<Option<String>>,
    memory_actor: RefCell<Option<String>>,
}

struct Emitter {
    from: String,
    stream: TcpStream,
    markers: Vec<TimelineMarkerReply>,
    registry: Arc<Mutex<ActorRegistry>>,
    start_stamp: PreciseTime,

    framerate_actor: Option<String>,
    memory_actor: Option<String>,
}

#[derive(RustcEncodable)]
struct IsRecordingReply {
    from: String,
    value: bool
}

#[derive(RustcEncodable)]
struct StartReply {
    from: String,
    value: HighResolutionStamp,
}

#[derive(RustcEncodable)]
struct StopReply {
    from: String,
    value: HighResolutionStamp,
}

#[derive(RustcEncodable)]
struct TimelineMarkerReply {
    name: String,
    start: HighResolutionStamp,
    end: HighResolutionStamp,
    stack: Option<Vec<()>>,
    endStack: Option<Vec<()>>,
}

#[derive(RustcEncodable)]
struct MarkersEmitterReply {
    __type__: String,
    markers: Vec<TimelineMarkerReply>,
    from: String,
    endTime: HighResolutionStamp,
}

#[derive(RustcEncodable)]
struct MemoryEmitterReply {
    __type__: String,
    from: String,
    delta: HighResolutionStamp,
    measurement: TimelineMemoryReply,
}

#[derive(RustcEncodable)]
struct FramerateEmitterReply {
    __type__: String,
    from: String,
    delta: HighResolutionStamp,
    timestamps: Vec<u64>,
}

/// HighResolutionStamp is struct that contains duration in milliseconds
/// with accuracy to microsecond that shows how much time has passed since
/// actor registry inited
/// analog https://w3c.github.io/hr-time/#sec-DOMHighResTimeStamp
struct HighResolutionStamp(f64);

impl HighResolutionStamp {
    fn new(start_stamp: PreciseTime, time: PreciseTime) -> HighResolutionStamp {
        let duration = start_stamp.to(time).num_microseconds()
                                  .expect("Too big duration in microseconds");
        HighResolutionStamp(duration as f64 / 1000 as f64)
    }
}

impl Encodable for HighResolutionStamp {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.0.encode(s)
    }
}

static DEFAULT_TIMELINE_DATA_PULL_TIMEOUT: u32 = 200; //ms

impl TimelineActor {
    pub fn new(name: String,
               pipeline: PipelineId,
               script_sender: Sender<DevtoolScriptControlMsg>) -> TimelineActor {

        let marker_types = vec!(TimelineMarkerType::Reflow,
                                TimelineMarkerType::DOMEvent);

        TimelineActor {
            name: name,
            pipeline: pipeline,
            marker_types: marker_types,
            script_sender: script_sender,
            is_recording: Arc::new(Mutex::new(false)),
            stream: RefCell::new(None),

            framerate_actor: RefCell::new(None),
            memory_actor: RefCell::new(None),
        }
    }

    fn pull_timeline_data(&self, receiver: Receiver<TimelineMarker>, mut emitter: Emitter) {
        let is_recording = self.is_recording.clone();

        if !*is_recording.lock().unwrap() {
            return;
        }

        /// Select root(with depth 0) TimelineMarker pair (IntervalStart + IntervalEnd)
        /// from queue and add marker to emitter
        /// Return true if closed (IntervalStart + IntervalEnd) pair was founded
        fn group(queue: &mut VecDeque<TimelineMarker>, depth: usize,
                 start_payload: Option<TimelineMarker>, emitter: &mut Emitter) -> bool {

            if let Some(start_payload) = start_payload {
                if start_payload.metadata != TracingMetadata::IntervalStart {
                    panic!("Start payload doesn't have metadata IntervalStart");
                }

                if let Some(end_payload) = queue.pop_front() {
                    match end_payload.metadata {
                        TracingMetadata::IntervalEnd => {
                            if depth == 0 {
                                // Emit TimelineMarkerReply, pair was found
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
                } else {
                    queue.push_front(start_payload);
                }
            }

            false
        }

        task::spawn_named("PullTimelineMarkers".to_string(), move || {
            let mut queues = HashMap::new();
            queues.insert("Reflow".to_string(), VecDeque::new());
            queues.insert("DOMEvent".to_string(), VecDeque::new());

            loop {
                if !*is_recording.lock().unwrap() {
                    break;
                }

                // Creating queues by marker.name
                loop {
                    match receiver.try_recv() {
                        Ok(marker) => {
                            if let Some(list) = queues.get_mut(&marker.name) {
                                list.push_back(marker);
                            }
                        }

                        Err(_) => break
                    }
                }

                // Emit all markers
                for (_, queue) in queues.iter_mut() {
                    let start_payload = queue.pop_front();
                    group(queue, 0, start_payload, &mut emitter);
                }
                emitter.send();

                sleep_ms(DEFAULT_TIMELINE_DATA_PULL_TIMEOUT);
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
                **self.is_recording.lock().as_mut().unwrap() = true;

                let (tx, rx) = channel::<TimelineMarker>();
                self.script_sender.send(SetTimelineMarkers(self.pipeline, self.marker_types.clone(), tx)).unwrap();

                *self.stream.borrow_mut() = stream.try_clone().ok();

                // init memory actor
                if let Some(with_memory) = msg.get("withMemory") {
                    if let Some(true) = with_memory.as_boolean() {
                        *self.memory_actor.borrow_mut() = Some(MemoryActor::create(registry));
                    }
                }

                // init framerate actor
                if let Some(with_ticks) = msg.get("withTicks") {
                    if let Some(true) = with_ticks.as_boolean() {
                        *self.framerate_actor.borrow_mut() = Some(FramerateActor::create(registry));
                    }
                }

                let emitter = Emitter::new(self.name(), registry.get_shareable(),
                                           registry.get_start_stamp(),
                                           stream.try_clone().unwrap(),
                                           self.memory_actor.borrow().clone(),
                                           self.framerate_actor.borrow().clone());

                self.pull_timeline_data(rx, emitter);

                let msg = StartReply {
                    from: self.name(),
                    value: HighResolutionStamp::new(registry.get_start_stamp(),
                                                    PreciseTime::now()),
                };
                stream.write_json_packet(&msg);
                true
            }

            "stop" => {
                let msg = StopReply {
                    from: self.name(),
                    value: HighResolutionStamp::new(registry.get_start_stamp(),
                                                    PreciseTime::now()),
                };

                stream.write_json_packet(&msg);
                self.script_sender.send(DropTimelineMarkers(self.pipeline, self.marker_types.clone())).unwrap();

                if let Some(ref actor_name) = *self.framerate_actor.borrow() {
                    registry.drop_actor_later(actor_name.clone());
                }

                if let Some(ref actor_name) = *self.memory_actor.borrow() {
                    registry.drop_actor_later(actor_name.clone());
                }

                **self.is_recording.lock().as_mut().unwrap() = false;
                self.stream.borrow_mut().take();
                true
            }

            "isRecording" => {
                let msg = IsRecordingReply {
                    from: self.name(),
                    value: self.is_recording.lock().unwrap().clone()
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
    pub fn new(name: String,
               registry: Arc<Mutex<ActorRegistry>>,
               start_stamp: PreciseTime,
               stream: TcpStream,
               memory_actor_name: Option<String>,
               framerate_actor_name: Option<String>) -> Emitter {

        Emitter {
            from: name,
            stream: stream,
            markers: Vec::new(),
            registry: registry,
            start_stamp: start_stamp,

            framerate_actor: framerate_actor_name,
            memory_actor: memory_actor_name,
        }
    }

    fn add_marker(&mut self, start_payload: TimelineMarker, end_payload: TimelineMarker) -> () {
        self.markers.push(TimelineMarkerReply {
            name: start_payload.name,
            start: HighResolutionStamp::new(self.start_stamp, start_payload.time),
            end: HighResolutionStamp::new(self.start_stamp, end_payload.time),
            stack: start_payload.stack,
            endStack: end_payload.stack,
        });
    }

    fn send(&mut self) -> () {
        let end_time = PreciseTime::now();
        let reply = MarkersEmitterReply {
            __type__: "markers".to_string(),
            markers: mem::replace(&mut self.markers, Vec::new()),
            from: self.from.clone(),
            endTime: HighResolutionStamp::new(self.start_stamp, end_time),
        };
        self.stream.write_json_packet(&reply);

        if let Some(ref actor_name) = self.framerate_actor {
            let mut lock = self.registry.lock();
            let registry = lock.as_mut().unwrap();
            let mut framerate_actor = registry.find_mut::<FramerateActor>(actor_name);
            let framerateReply = FramerateEmitterReply {
                __type__: "framerate".to_string(),
                from: framerate_actor.name(),
                delta: HighResolutionStamp::new(self.start_stamp, end_time),
                timestamps: framerate_actor.take_pending_ticks(),
            };
            self.stream.write_json_packet(&framerateReply);
        }

        if let Some(ref actor_name) = self.memory_actor {
            let registry = self.registry.lock().unwrap();
            let memory_actor = registry.find::<MemoryActor>(actor_name);
            let memoryReply = MemoryEmitterReply {
                __type__: "memory".to_string(),
                from: memory_actor.name(),
                delta: HighResolutionStamp::new(self.start_stamp, end_time),
                measurement: memory_actor.measure(),
            };
            self.stream.write_json_packet(&memoryReply);
        }
    }
}
