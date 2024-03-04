/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::mem;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use devtools_traits::PreciseTime;
use log::{debug, warn};
use serde_json::{Map, Value};

/// General actor system infrastructure.
use crate::StreamId;

#[derive(PartialEq)]
pub enum ActorMessageStatus {
    Processed,
    Ignored,
}

/// A common trait for all devtools actors that encompasses an immutable name
/// and the ability to process messages that are directed to particular actors.
/// TODO: ensure the name is immutable
pub(crate) trait Actor: Any + ActorAsAny {
    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        id: StreamId,
    ) -> Result<ActorMessageStatus, ()>;
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
    shareable: Option<Arc<Mutex<ActorRegistry>>>,
    next: Cell<u32>,
    start_stamp: PreciseTime,
}

impl ActorRegistry {
    /// Create an empty registry.
    pub fn new() -> ActorRegistry {
        ActorRegistry {
            actors: HashMap::new(),
            new_actors: RefCell::new(vec![]),
            old_actors: RefCell::new(vec![]),
            script_actors: RefCell::new(HashMap::new()),
            shareable: None,
            next: Cell::new(0),
            start_stamp: PreciseTime::now(),
        }
    }

    pub(crate) fn cleanup(&self, id: StreamId) {
        for actor in self.actors.values() {
            actor.cleanup(id);
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
    pub fn start_stamp(&self) -> PreciseTime {
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

    /// Attempt to process a message as directed by its `to` property. If the actor is not
    /// found or does not indicate that it knew how to process the message, ignore the failure.
    pub(crate) fn handle_message(
        &mut self,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        id: StreamId,
    ) -> Result<(), ()> {
        let to = match msg.get("to") {
            Some(to) => to.as_str().unwrap(),
            None => {
                warn!("Received unexpected message: {:?}", msg);
                return Err(());
            },
        };

        match self.actors.get(to) {
            None => debug!("message received for unknown actor \"{}\"", to),
            Some(actor) => {
                let msg_type = msg.get("type").unwrap().as_str().unwrap();
                if actor.handle_message(self, msg_type, msg, stream, id)? !=
                    ActorMessageStatus::Processed
                {
                    debug!(
                        "unexpected message type \"{}\" found for actor \"{}\"",
                        msg_type, to
                    );
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
}
