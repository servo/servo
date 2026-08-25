/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::any::{Any, type_name};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use log::{debug, warn};
use malloc_size_of::MallocSizeOf;
use malloc_size_of_derive::MallocSizeOf;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
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
        format!("/workerTarget{suffix}")
    } else {
        format!("{base}{suffix}")
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

impl<T: Actor> From<Arc<T>> for DowncastableActorArc<T> {
    fn from(actor: Arc<T>) -> Self {
        Self {
            actor,
            _phantom: PhantomData,
        }
    }
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

impl MallocSizeOf for RegisteredActor {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        self.0.size_of(ops)
    }
}

// Debug-only guard to prevent deadlocks. Panics if `ActorRegistry::write` is called
// reentrantly on the same thread before the previous write guard is dropped.
#[cfg(debug_assertions)]
thread_local! {
    static REENTRANCY_GUARD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

struct WriteGuard<'a>(RwLockWriteGuard<'a, ActorRegistryInner>);

impl<'a> Deref for WriteGuard<'a> {
    type Target = ActorRegistryInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for WriteGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> Drop for WriteGuard<'a> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        REENTRANCY_GUARD.with(|cell| cell.set(false));
    }
}

#[derive(Default, MallocSizeOf)]
struct ActorRegistryInner {
    actors: HashSet<RegisteredActor>,
    script_to_actor: HashMap<String, String>,
    actor_to_script: HashMap<String, String>,
    source_actor_names: HashMap<PipelineId, Vec<String>>,
    inline_source_content: HashMap<PipelineId, String>,
}

#[derive(Default, MallocSizeOf)]
pub(crate) struct ActorRegistry(RwLock<ActorRegistryInner>);

impl ActorRegistry {
    fn read(&self) -> RwLockReadGuard<'_, ActorRegistryInner> {
        self.0.read()
    }

    fn write(&self) -> WriteGuard<'_> {
        #[cfg(debug_assertions)]
        REENTRANCY_GUARD.with(|cell| {
            assert!(
                !cell.get(),
                "Reentrant write operation detected on the same thread for ActorRegistry"
            );
            cell.set(true);
        });
        WriteGuard(self.0.write())
    }
}

impl ActorRegistry {
    /// Add an actor to the registry of known actors that can receive messages.
    pub(crate) fn register<T: Actor>(&self, actor: T) -> Arc<T> {
        let actor = Arc::new(actor);
        self.write().actors.insert(RegisteredActor(actor.clone()));
        actor
    }

    /// Find an actor by registered name
    pub(crate) fn find<T: Actor>(&self, name: &str) -> DowncastableActorArc<T> {
        let actor = self
            .read()
            .actors
            .get(name)
            .expect("Should never look for a nonexistent actor")
            .0
            .clone();
        DowncastableActorArc {
            actor,
            _phantom: PhantomData,
        }
    }

    /// Find an actor by registered name and return its serialization
    pub(crate) fn encode<T: ActorEncode<S>, S: Serialize>(&self, name: &str) -> S {
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
        let Some(actor_name) = msg.get("to").and_then(|value| value.as_str()) else {
            warn!("Received unexpected message: {msg:?}");
            return Err(());
        };

        let actor = self
            .read()
            .actors
            .get(actor_name)
            .map(|registered_actor| registered_actor.0.clone());
        match actor {
            None => {
                // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#packets>
                let _ = stream
                    .write_json_packet(&json!({ "from": actor_name, "error": "noSuchActor" }));
            },
            Some(actor) => {
                let Some(msg_type) = msg.get("type").and_then(|value| value.as_str()) else {
                    let _ = stream.write_json_packet(
                        &json!({ "from": actor_name, "error": "missingParameter" }),
                    );
                    return Ok(());
                };
                if let Err(error) = ClientRequest::handle(stream.clone(), actor_name, |req| {
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

    pub(crate) fn register_script_actor(&self, script_id: String, actor: String) {
        debug!("Registering {actor} ({script_id})");
        let mut lock = self.write();
        lock.script_to_actor
            .insert(script_id.clone(), actor.clone());
        lock.actor_to_script.insert(actor, script_id);
    }

    pub(crate) fn script_to_actor(&self, script_id: &str) -> String {
        if script_id.is_empty() {
            return String::new();
        }
        self.read()
            .script_to_actor
            .get(script_id)
            .unwrap_or_else(|| panic!("No actor for script id {script_id}"))
            .clone()
    }

    pub(crate) fn script_actor_registered(&self, script_id: &str) -> bool {
        self.read().script_to_actor.contains_key(script_id)
    }

    pub(crate) fn actor_to_script(&self, actor: String) -> String {
        self.read()
            .actor_to_script
            .get(&actor)
            .unwrap_or_else(|| panic!("No script id for actor {actor}"))
            .clone()
    }

    pub(crate) fn register_source_actor(&self, pipeline_id: PipelineId, actor_name: &str) {
        self.write()
            .source_actor_names
            .entry(pipeline_id)
            .or_default()
            .push(actor_name.to_owned());
    }

    pub(crate) fn source_actor_names_for_pipeline(&self, pipeline_id: PipelineId) -> Vec<String> {
        self.read()
            .source_actor_names
            .get(&pipeline_id)
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn set_inline_source_content(&self, pipeline_id: PipelineId, content: String) {
        assert!(
            self.write()
                .inline_source_content
                .insert(pipeline_id, content)
                .is_none()
        );
    }

    pub(crate) fn inline_source_content(&self, pipeline_id: PipelineId) -> Option<String> {
        self.read().inline_source_content.get(&pipeline_id).cloned()
    }

    /// This is a no-op, we don't currently handle removals from the actor registry
    pub(crate) fn remove(&self, _name: String) {}

    pub(crate) fn cleanup(&self, stream_id: StreamId) {
        let actors = self.read().actors.clone();
        for actor in actors {
            actor.0.cleanup(stream_id);
        }
    }
}
