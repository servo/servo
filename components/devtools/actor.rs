/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::any::{Any, type_name};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use atomic_refcell::AtomicRefCell;
use log::{debug, warn};
use malloc_size_of::MallocSizeOf;
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Value, json};
use servo_base::id::PipelineId;

use crate::StreamId;
use crate::protocol::{ClientRequest, DevtoolsConnection, JsonPacketStream};

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

/// Create a name prefix for each actor type, without any counter suffix.
pub(crate) fn base_name<T: ?Sized>() -> &'static str {
    let prefix = type_name::<T>();
    prefix.split("::").last().unwrap_or(prefix)
}

/// Create a unique actor name based on the type and a monotonically increasing suffix.
pub(crate) fn new_actor_name<T: ?Sized>() -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let suffix = COUNTER.fetch_add(1, Ordering::Relaxed);
    let base = base_name::<T>();

    // Firefox DevTools client requires "/workerTarget" in actor name to recognize workers
    // <https://searchfox.org/firefox-main/source/devtools/client/fronts/watcher.js#65>
    if base.contains("WorkerTarget") {
        format!("/workerTarget{}", suffix)
    } else {
        format!("{}{}", base, suffix)
    }
}

/// A common trait for all devtools actors that encompasses an immutable name
/// and the ability to process messages that are directed to particular actors.
pub(crate) trait Actor: Any + ActorAsAny + Send + Sync + MallocSizeOf {
    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream_id: StreamId,
    ) -> Result<(), ActorError> {
        let _ = (request, registry, msg_type, msg, stream_id);
        Err(ActorError::UnrecognizedPacketType)
    }
    fn name(&self) -> &str;
    fn cleanup(&self, _id: StreamId) {}
}

pub(crate) trait ActorAsAny {
    fn actor_as_any(&self) -> &dyn Any;
}

impl<T: Actor> ActorAsAny for T {
    fn actor_as_any(&self) -> &dyn Any {
        self
    }
}

pub(crate) trait ActorEncode<T: Serialize>: Actor {
    fn encode(&self, registry: &ActorRegistry) -> T;
}

/// Return value of `ActorRegistry::find` that allows seamless downcasting
/// from `dyn Actor` to the concrete actor type.
pub(crate) struct DowncastableActorArc<T> {
    actor: Arc<dyn Actor>,
    _phantom: PhantomData<T>,
}

impl<T: 'static> std::ops::Deref for DowncastableActorArc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.actor
            .actor_as_any()
            .downcast_ref::<T>()
            .unwrap_or_else(|| {
                panic!(
                    "Failed to downcast {} to type {}",
                    self.actor.name(),
                    type_name::<T>()
                )
            })
    }
}

#[derive(Clone)]
struct RegisteredActor(Arc<dyn Actor>);

impl PartialEq for RegisteredActor {
    fn eq(&self, other: &Self) -> bool {
        self.0.name() == other.0.name()
    }
}

impl Eq for RegisteredActor {}

impl Hash for RegisteredActor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.name().hash(state);
    }
}

impl Borrow<str> for RegisteredActor {
    fn borrow(&self) -> &str {
        self.0.name()
    }
}

#[derive(Default)]
struct ActorRegistryType(AtomicRefCell<HashSet<RegisteredActor>>);

impl MallocSizeOf for ActorRegistryType {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        self.0
            .borrow()
            .iter()
            .map(|wrapper| wrapper.0.size_of(ops))
            .sum()
    }
}

/// A list of known, owned actors.
#[derive(Default, MallocSizeOf)]
pub(crate) struct ActorRegistry {
    actors: ActorRegistryType,
    script_actors: AtomicRefCell<HashMap<String, String>>,
    /// Lookup table for SourceActor names associated with a given PipelineId.
    source_actor_names: AtomicRefCell<HashMap<PipelineId, Vec<String>>>,
    /// Lookup table for inline source content associated with a given PipelineId.
    inline_source_content: AtomicRefCell<HashMap<PipelineId, String>>,
}

impl ActorRegistry {
    pub(crate) fn cleanup(&self, stream_id: StreamId) {
        let actors: Vec<Arc<dyn Actor>> = {
            let guard = self.actors.0.borrow();
            guard.iter().map(|r| r.0.clone()).collect()
        };

        for actor in actors {
            actor.cleanup(stream_id);
        }
    }

    pub fn register_script_actor(&self, script_id: String, actor: String) {
        debug!("registering {} ({})", actor, script_id);
        let mut script_actors = self.script_actors.borrow_mut();
        script_actors.insert(script_id, actor);
    }

    pub fn script_to_actor(&self, script_id: &str) -> String {
        if script_id.is_empty() {
            return "".to_owned();
        }
        self.script_actors.borrow().get(script_id).unwrap().clone()
    }

    pub fn script_actor_registered(&self, script_id: &str) -> bool {
        self.script_actors.borrow().contains_key(script_id)
    }

    pub fn actor_to_script(&self, actor: String) -> String {
        for (key, value) in &*self.script_actors.borrow() {
            if *value == actor {
                return key.to_owned();
            }
        }
        panic!("couldn't find actor named {}", actor)
    }

    /// Add an actor to the registry of known actors that can receive messages.
    pub(crate) fn register<T: Actor>(&self, actor: T) {
        let mut guard = self.actors.0.borrow_mut();
        guard.insert(RegisteredActor(Arc::new(actor)));
    }

    /// Find an actor by registered name
    pub fn find<T: Actor>(&self, name: &str) -> DowncastableActorArc<T> {
        let actor = {
            let guard = self.actors.0.borrow();
            guard
                .get(name)
                .expect("Should never look for a nonexistent actor")
                .0
                .clone()
        }; // Read guard is dropped here!
        DowncastableActorArc {
            actor,
            _phantom: PhantomData,
        }
    }

    /// Find an actor by registered name and return its serialization
    pub fn encode<T: ActorEncode<S>, S: Serialize>(&self, name: &str) -> S {
        self.find::<T>(name).encode(self)
    }

    /// Attempt to process a message as directed by its `to` property. If the actor is not found, does not support the
    /// message, or failed to handle the message, send an error reply instead.
    pub(crate) fn handle_message(
        &self,
        msg: &Map<String, Value>,
        stream: &mut DevtoolsConnection,
        stream_id: StreamId,
    ) -> Result<(), ()> {
        let to = match msg.get("to") {
            Some(to) => to.as_str().unwrap(),
            None => {
                log::warn!("Received unexpected message: {:?}", msg);
                return Err(());
            },
        };

        let actor = {
            let actors_map = self.actors.0.borrow();
            actors_map.get(to).map(|r| r.0.clone())
        };
        match actor {
            None => {
                // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#packets>
                let msg = json!({ "from": to, "error": "noSuchActor" });
                let _ = stream.write_json_packet(&msg);
            },
            Some(actor) => {
                let msg_type = msg.get("type").unwrap().as_str().unwrap();
                if let Err(error) = ClientRequest::handle(stream.clone(), to, |req| {
                    actor.handle_message(req, self, msg_type, msg, stream_id)
                }) {
                    // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#error-packets>
                    let error = json!({
                        "from": actor.name(), "error": error.name()
                    });
                    warn!("Sending devtools protocol error: error={error:?} request={msg:?}");
                    let _ = stream.write_json_packet(&error);
                }
            },
        }
        Ok(())
    }

    pub fn remove(&self, name: &str) {
        let mut guard = self.actors.0.borrow_mut();
        guard.remove(name);
    }

    pub fn register_source_actor(&self, pipeline_id: PipelineId, actor_name: &str) {
        self.source_actor_names
            .borrow_mut()
            .entry(pipeline_id)
            .or_default()
            .push(actor_name.to_owned());
    }

    pub fn source_actor_names_for_pipeline(&self, pipeline_id: PipelineId) -> Vec<String> {
        self.source_actor_names
            .borrow_mut()
            .get(&pipeline_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_inline_source_content(&self, pipeline_id: PipelineId, content: String) {
        assert!(
            self.inline_source_content
                .borrow_mut()
                .insert(pipeline_id, content)
                .is_none()
        );
    }

    pub fn inline_source_content(&self, pipeline_id: PipelineId) -> Option<String> {
        self.inline_source_content
            .borrow()
            .get(&pipeline_id)
            .cloned()
    }
}
