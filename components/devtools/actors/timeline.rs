/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// TODO: Is this actor still relevant?
#![allow(dead_code)]

use std::cell::RefCell;
use std::error::Error;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg::{DropTimelineMarkers, SetTimelineMarkers};
use devtools_traits::{DevtoolScriptControlMsg, PreciseTime, TimelineMarker, TimelineMarkerType};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::framerate::FramerateActor;
use crate::actors::memory::{MemoryActor, TimelineMemoryReply};
use crate::protocol::JsonPacketStream;
use crate::StreamId;

pub struct TimelineActor {
    name: String,
    script_sender: IpcSender<DevtoolScriptControlMsg>,
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
    registry: Arc<Mutex<ActorRegistry>>,
    start_stamp: PreciseTime,

    framerate_actor: Option<String>,
    memory_actor: Option<String>,
}

#[derive(Serialize)]
struct IsRecordingReply {
    from: String,
    value: bool,
}

#[derive(Serialize)]
struct StartReply {
    from: String,
    value: HighResolutionStamp,
}

#[derive(Serialize)]
struct StopReply {
    from: String,
    value: HighResolutionStamp,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TimelineMarkerReply {
    name: String,
    start: HighResolutionStamp,
    end: HighResolutionStamp,
    stack: Option<Vec<()>>,
    end_stack: Option<Vec<()>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MarkersEmitterReply {
    #[serde(rename = "type")]
    type_: String,
    markers: Vec<TimelineMarkerReply>,
    from: String,
    end_time: HighResolutionStamp,
}

#[derive(Serialize)]
struct MemoryEmitterReply {
    #[serde(rename = "type")]
    type_: String,
    from: String,
    delta: HighResolutionStamp,
    measurement: TimelineMemoryReply,
}

#[derive(Serialize)]
struct FramerateEmitterReply {
    #[serde(rename = "type")]
    type_: String,
    from: String,
    delta: HighResolutionStamp,
    timestamps: Vec<HighResolutionStamp>,
}

/// HighResolutionStamp is struct that contains duration in milliseconds
/// with accuracy to microsecond that shows how much time has passed since
/// actor registry inited
/// analog <https://w3c.github.io/hr-time/#sec-DOMHighResTimeStamp>
pub struct HighResolutionStamp(f64);

impl HighResolutionStamp {
    pub fn new(start_stamp: PreciseTime, time: PreciseTime) -> HighResolutionStamp {
        let duration = start_stamp.to(time).as_micros();
        HighResolutionStamp(duration as f64 / 1000_f64)
    }

    pub fn wrap(time: f64) -> HighResolutionStamp {
        HighResolutionStamp(time)
    }
}

impl Serialize for HighResolutionStamp {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

static DEFAULT_TIMELINE_DATA_PULL_TIMEOUT: u64 = 200; //ms

impl TimelineActor {
    pub fn new(
        name: String,
        pipeline: PipelineId,
        script_sender: IpcSender<DevtoolScriptControlMsg>,
    ) -> TimelineActor {
        let marker_types = vec![TimelineMarkerType::Reflow, TimelineMarkerType::DOMEvent];

        TimelineActor {
            name,
            pipeline,
            marker_types,
            script_sender,
            is_recording: Arc::new(Mutex::new(false)),
            stream: RefCell::new(None),

            framerate_actor: RefCell::new(None),
            memory_actor: RefCell::new(None),
        }
    }

    fn pull_timeline_data(
        &self,
        receiver: IpcReceiver<Option<TimelineMarker>>,
        mut emitter: Emitter,
    ) {
        let is_recording = self.is_recording.clone();

        if !*is_recording.lock().unwrap() {
            return;
        }

        thread::Builder::new()
            .name("PullTimelineData".to_owned())
            .spawn(move || loop {
                if !*is_recording.lock().unwrap() {
                    break;
                }

                let mut markers = vec![];
                while let Ok(Some(marker)) = receiver.try_recv() {
                    markers.push(emitter.marker(marker));
                }
                if emitter.send(markers).is_err() {
                    break;
                }

                thread::sleep(Duration::from_millis(DEFAULT_TIMELINE_DATA_PULL_TIMEOUT));
            })
            .expect("Thread spawning failed");
    }
}

impl Actor for TimelineActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "start" => {
                **self.is_recording.lock().as_mut().unwrap() = true;

                let (tx, rx) = ipc::channel::<Option<TimelineMarker>>().unwrap();
                self.script_sender
                    .send(SetTimelineMarkers(
                        self.pipeline,
                        self.marker_types.clone(),
                        tx,
                    ))
                    .unwrap();

                //TODO: support multiple connections by using root actor's streams instead.
                *self.stream.borrow_mut() = stream.try_clone().ok();

                // init memory actor
                if let Some(with_memory) = msg.get("withMemory") {
                    if let Some(true) = with_memory.as_bool() {
                        *self.memory_actor.borrow_mut() = Some(MemoryActor::create(registry));
                    }
                }

                // init framerate actor
                if let Some(with_ticks) = msg.get("withTicks") {
                    if let Some(true) = with_ticks.as_bool() {
                        let framerate_actor = Some(FramerateActor::create(
                            registry,
                            self.pipeline,
                            self.script_sender.clone(),
                        ));
                        *self.framerate_actor.borrow_mut() = framerate_actor;
                    }
                }

                let emitter = Emitter::new(
                    self.name(),
                    registry.shareable(),
                    registry.start_stamp(),
                    stream.try_clone().unwrap(),
                    self.memory_actor.borrow().clone(),
                    self.framerate_actor.borrow().clone(),
                );

                self.pull_timeline_data(rx, emitter);

                let msg = StartReply {
                    from: self.name(),
                    value: HighResolutionStamp::new(registry.start_stamp(), PreciseTime::now()),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "stop" => {
                let msg = StopReply {
                    from: self.name(),
                    value: HighResolutionStamp::new(registry.start_stamp(), PreciseTime::now()),
                };

                let _ = stream.write_json_packet(&msg);
                self.script_sender
                    .send(DropTimelineMarkers(
                        self.pipeline,
                        self.marker_types.clone(),
                    ))
                    .unwrap();

                //TODO: move this to the cleanup method.
                if let Some(ref actor_name) = *self.framerate_actor.borrow() {
                    registry.drop_actor_later(actor_name.clone());
                }

                if let Some(ref actor_name) = *self.memory_actor.borrow() {
                    registry.drop_actor_later(actor_name.clone());
                }

                **self.is_recording.lock().as_mut().unwrap() = false;
                self.stream.borrow_mut().take();
                ActorMessageStatus::Processed
            },

            "isRecording" => {
                let msg = IsRecordingReply {
                    from: self.name(),
                    value: *self.is_recording.lock().unwrap(),
                };

                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl Emitter {
    pub fn new(
        name: String,
        registry: Arc<Mutex<ActorRegistry>>,
        start_stamp: PreciseTime,
        stream: TcpStream,
        memory_actor_name: Option<String>,
        framerate_actor_name: Option<String>,
    ) -> Emitter {
        Emitter {
            from: name,
            stream,
            registry,
            start_stamp,

            framerate_actor: framerate_actor_name,
            memory_actor: memory_actor_name,
        }
    }

    fn marker(&self, payload: TimelineMarker) -> TimelineMarkerReply {
        TimelineMarkerReply {
            name: payload.name,
            start: HighResolutionStamp::new(self.start_stamp, payload.start_time),
            end: HighResolutionStamp::new(self.start_stamp, payload.end_time),
            stack: payload.start_stack,
            end_stack: payload.end_stack,
        }
    }

    fn send(&mut self, markers: Vec<TimelineMarkerReply>) -> Result<(), Box<dyn Error>> {
        let end_time = PreciseTime::now();
        let reply = MarkersEmitterReply {
            type_: "markers".to_owned(),
            markers,
            from: self.from.clone(),
            end_time: HighResolutionStamp::new(self.start_stamp, end_time),
        };
        self.stream.write_json_packet(&reply)?;

        if let Some(ref actor_name) = self.framerate_actor {
            let mut lock = self.registry.lock();
            let registry = lock.as_mut().unwrap();
            let framerate_actor = registry.find_mut::<FramerateActor>(actor_name);
            let framerate_reply = FramerateEmitterReply {
                type_: "framerate".to_owned(),
                from: framerate_actor.name(),
                delta: HighResolutionStamp::new(self.start_stamp, end_time),
                timestamps: framerate_actor.take_pending_ticks(),
            };
            self.stream.write_json_packet(&framerate_reply)?;
        }

        if let Some(ref actor_name) = self.memory_actor {
            let registry = self.registry.lock().unwrap();
            let memory_actor = registry.find::<MemoryActor>(actor_name);
            let memory_reply = MemoryEmitterReply {
                type_: "memory".to_owned(),
                from: memory_actor.name(),
                delta: HighResolutionStamp::new(self.start_stamp, end_time),
                measurement: memory_actor.measure(),
            };
            self.stream.write_json_packet(&memory_reply)?;
        }

        Ok(())
    }
}
