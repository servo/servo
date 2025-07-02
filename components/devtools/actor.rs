/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::mem;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use base::cross_process_instant::CrossProcessInstant;
use base::id::PipelineId;
use log::debug;
use serde_json::{Map, Value, json};

use crate::StreamId;
use crate::protocol::{ClientRequest, JsonPacketStream};

/// Error replies.
///
/// <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#error-packets>
#[derive(Debug)]
pub enum ActorError {
    MissingParameter,
    BadParameterType,
    UnrecognizedPacketType,
    /// Custom errors, not defined in the protocol docs.
    /// This includes send errors, and errors that prevent Servo from sending a reply.
    Internal,
}

impl ActorError {
    pub fn name(&self) -> &'static str {
        match self {
            ActorError::MissingParameter => "missingParameter",
            ActorError::BadParameterType => "badParameterType",
            ActorError::UnrecognizedPacketType => "unrecognizedPacketType",
            // The devtools frontend always checks for specific protocol errors by catching a JS exception `e` whose
            // message contains the error name, and checking `e.message.includes("someErrorName")`. As a result, the
            // only error name we can safely use for custom errors is the empty string, because any other error name we
            // use may be a substring of some upstream error name.
            ActorError::Internal => "",
        }
    }
}

/// A common trait for all devtools actors that encompasses an immutable name
/// and the ability to process messages that are directed to particular actors.
/// TODO: ensure the name is immutable
pub(crate) trait Actor: Any + ActorAsAny {
    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream_id: StreamId,
    ) -> Result<(), ActorError>;
    fn name(&self) -> String;
    fn cleanup(&self, _id: StreamId) {}
}

pub(crate) trait ActorAsAny {
    fn actor_as_any(&self) -> &dyn Any;
    fn actor_as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Actor> ActorAsAny for T {
    fn actor_as_any(&self) -> &dyn Any {
        self
    }
    fn actor_as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A list of known, owned actors.
pub struct ActorRegistry {
    actors: HashMap<String, Box<dyn Actor + Send>>,
    new_actors: RefCell<Vec<Box<dyn Actor + Send>>>,
    old_actors: RefCell<Vec<String>>,
    script_actors: RefCell<HashMap<String, String>>,

    /// Lookup table for SourceActor names associated with a given PipelineId.
    source_actor_names: RefCell<HashMap<PipelineId, Vec<String>>>,

    shareable: Option<Arc<Mutex<ActorRegistry>>>,
    next: Cell<u32>,
    start_stamp: CrossProcessInstant,
}

impl ActorRegistry {
    /// Create an empty registry.
    pub fn new() -> ActorRegistry {
        ActorRegistry {
            actors: HashMap::new(),
            new_actors: RefCell::new(vec![]),
            old_actors: RefCell::new(vec![]),
            script_actors: RefCell::new(HashMap::new()),
            source_actor_names: RefCell::new(HashMap::new()),
            shareable: None,
            next: Cell::new(0),
            start_stamp: CrossProcessInstant::now(),
        }
    }

    pub(crate) fn cleanup(&self, stream_id: StreamId) {
        for actor in self.actors.values() {
            actor.cleanup(stream_id);
        }
    }

    /// Creating shareable registry
    pub fn create_shareable(self) -> Arc<Mutex<ActorRegistry>> {
        if let Some(shareable) = self.shareable {
            return shareable;
        }

        let shareable = Arc::new(Mutex::new(self));
        {
            let mut lock = shareable.lock();
            let registry = lock.as_mut().unwrap();
            registry.shareable = Some(shareable.clone());
        }
        shareable
    }

    /// Get shareable registry through threads
    pub fn shareable(&self) -> Arc<Mutex<ActorRegistry>> {
        self.shareable.as_ref().unwrap().clone()
    }

    /// Get start stamp when registry was started
    pub fn start_stamp(&self) -> CrossProcessInstant {
        self.start_stamp
    }

    pub fn register_script_actor(&self, script_id: String, actor: String) {
        debug!("registering {} ({})", actor, script_id);
        let mut script_actors = self.script_actors.borrow_mut();
        script_actors.insert(script_id, actor);
    }

    pub fn script_to_actor(&self, script_id: String) -> String {
        if script_id.is_empty() {
            return "".to_owned();
        }
        self.script_actors.borrow().get(&script_id).unwrap().clone()
    }

    pub fn script_actor_registered(&self, script_id: String) -> bool {
        self.script_actors.borrow().contains_key(&script_id)
    }

    pub fn actor_to_script(&self, actor: String) -> String {
        for (key, value) in &*self.script_actors.borrow() {
            debug!("checking {}", value);
            if *value == actor {
                return key.to_owned();
            }
        }
        panic!("couldn't find actor named {}", actor)
    }

    /// Create a unique name based on a monotonically increasing suffix
    pub fn new_name(&self, prefix: &str) -> String {
        let suffix = self.next.get();
        self.next.set(suffix + 1);
        format!("{}{}", prefix, suffix)
    }

    /// Add an actor to the registry of known actors that can receive messages.
    pub(crate) fn register(&mut self, actor: Box<dyn Actor + Send>) {
        self.actors.insert(actor.name(), actor);
    }

    pub(crate) fn register_later(&self, actor: Box<dyn Actor + Send>) {
        let mut actors = self.new_actors.borrow_mut();
        actors.push(actor);
    }

    /// Find an actor by registered name
    pub fn find<'a, T: Any>(&'a self, name: &str) -> &'a T {
        let actor = self.actors.get(name).unwrap();
        actor.actor_as_any().downcast_ref::<T>().unwrap()
    }

    /// Find an actor by registered name
    pub fn find_mut<'a, T: Any>(&'a mut self, name: &str) -> &'a mut T {
        let actor = self.actors.get_mut(name).unwrap();
        actor.actor_as_any_mut().downcast_mut::<T>().unwrap()
    }

    /// Attempt to process a message as directed by its `to` property. If the actor is not found, does not support the
    /// message, or failed to handle the message, send an error reply instead.
    pub(crate) fn handle_message(
        &mut self,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        stream_id: StreamId,
    ) -> Result<(), ()> {
        let to = match msg.get("to") {
            Some(to) => to.as_str().unwrap(),
            None => {
                log::warn!("Received unexpected message: {:?}", msg);
                return Err(());
            },
        };

        match self.actors.get(to) {
            None => {
                // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#packets>
                let msg = json!({ "from": to, "error": "noSuchActor" });
                let _ = stream.write_json_packet(&msg);
            },
            Some(actor) => {
                let msg_type = msg.get("type").unwrap().as_str().unwrap();
                if let Err(error) = ClientRequest::handle(stream, to, |req| {
                    actor.handle_message(req, self, msg_type, msg, stream_id)
                }) {
                    // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#error-packets>
                    let _ = stream.write_json_packet(&json!({
                        "from": actor.name(), "error": error.name()
                    }));
                }
            },
        }
        let new_actors = mem::take(&mut *self.new_actors.borrow_mut());
        for actor in new_actors.into_iter() {
            self.actors.insert(actor.name().to_owned(), actor);
        }

        let old_actors = mem::take(&mut *self.old_actors.borrow_mut());
        for name in old_actors {
            self.drop_actor(name);
        }
        Ok(())
    }

    pub fn drop_actor(&mut self, name: String) {
        self.actors.remove(&name);
    }

    pub fn drop_actor_later(&self, name: String) {
        let mut actors = self.old_actors.borrow_mut();
        actors.push(name);
    }

    pub fn register_source_actor(&self, pipeline_id: PipelineId, actor_name: &str) {
        self.source_actor_names
            .borrow_mut()
            .entry(pipeline_id)
            .or_default()
            .push(actor_name.to_owned());
    }

    pub fn source_actor_names_for_pipeline(&mut self, pipeline_id: PipelineId) -> Vec<String> {
        if let Some(source_actor_names) = self.source_actor_names.borrow_mut().get(&pipeline_id) {
            return source_actor_names.clone();
        }

        vec![]
    }
}
