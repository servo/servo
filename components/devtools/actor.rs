/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// General actor system infrastructure.

use std::any::{AnyPrivate, AnyRefExt, AnyMutRefExt};
use std::collections::hashmap::HashMap;
use std::cell::{Cell, RefCell};
use std::intrinsics::TypeId;
use std::io::TcpStream;
use std::mem::{transmute, transmute_copy, replace};
use std::raw::TraitObject;
use serialize::json;

/// A common trait for all devtools actors that encompasses an immutable name
/// and the ability to process messages that are directed to particular actors.
/// TODO: ensure the name is immutable
pub trait Actor: AnyPrivate {
    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &String,
                      msg: &json::JsonObject,
                      stream: &mut TcpStream) -> bool;
    fn name(&self) -> String;
}

impl<'a> AnyMutRefExt<'a> for &'a mut Actor + 'a {
    fn downcast_mut<T: 'static>(self) -> Option<&'a mut T> {
        if self.is::<T>() {
            unsafe {
                // Get the raw representation of the trait object
                let to: TraitObject = transmute_copy(&self);

                // Extract the data pointer
                Some(transmute(to.data))
            }
        } else {
            None
        }
    }
}

impl<'a> AnyRefExt<'a> for &'a Actor + 'a {
    fn is<T: 'static>(self) -> bool {
        // This implementation is only needed so long as there's a Rust bug that
        // prevents downcast_ref from giving realistic return values.
        let t = TypeId::of::<T>();
        let boxed = self.get_type_id();
        t == boxed
    }

    fn downcast_ref<T: 'static>(self) -> Option<&'a T> {
        if self.is::<T>() {
            unsafe {
                // Get the raw representation of the trait object
                let to: TraitObject = transmute_copy(&self);

                // Extract the data pointer
                Some(transmute(to.data))
            }
        } else {
            None
        }
    }
}

/// A list of known, owned actors.
pub struct ActorRegistry {
    actors: HashMap<String, Box<Actor+Send+Sized>>,
    new_actors: RefCell<Vec<Box<Actor+Send+Sized>>>,
    script_actors: RefCell<HashMap<String, String>>,
    next: Cell<u32>,
}

impl ActorRegistry {
    /// Create an empty registry.
    pub fn new() -> ActorRegistry {
        ActorRegistry {
            actors: HashMap::new(),
            new_actors: RefCell::new(vec!()),
            script_actors: RefCell::new(HashMap::new()),
            next: Cell::new(0),
        }
    }

    pub fn register_script_actor(&self, script_id: String, actor: String) {
        println!("registering {:s} ({:s})", actor.as_slice(), script_id.as_slice());
        let mut script_actors = self.script_actors.borrow_mut();
        script_actors.insert(script_id, actor);
    }

    pub fn script_to_actor(&self, script_id: String) -> String {
        if script_id.as_slice() == "" {
            return "".to_string();
        }
        self.script_actors.borrow().find(&script_id).unwrap().to_string()
    }

    pub fn script_actor_registered(&self, script_id: String) -> bool {
        self.script_actors.borrow().contains_key(&script_id)
    }

    pub fn actor_to_script(&self, actor: String) -> String {
        for (key, value) in self.script_actors.borrow().iter() {
            println!("checking {:s}", value.as_slice());
            if value.as_slice() == actor.as_slice() {
                return key.to_string();
            }
        }
        fail!("couldn't find actor named {:s}", actor)
    }

    /// Create a unique name based on a monotonically increasing suffix
    pub fn new_name(&self, prefix: &str) -> String {
        let suffix = self.next.get();
        self.next.set(suffix + 1);
        format!("{:s}{:u}", prefix, suffix)
    }

    /// Add an actor to the registry of known actors that can receive messages.
    pub fn register(&mut self, actor: Box<Actor+Send+Sized>) {
        self.actors.insert(actor.name().to_string(), actor);
    }

    pub fn register_later(&self, actor: Box<Actor+Send+Sized>) {
        let mut actors = self.new_actors.borrow_mut();
        actors.push(actor);
    }

    /// Find an actor by registered name
    pub fn find<'a, T: 'static>(&'a self, name: &str) -> &'a T {
        //FIXME: Rust bug forces us to implement bogus Any for Actor since downcast_ref currently
        //       fails for unknown reasons.
        /*let actor: &Actor+Send+Sized = *self.actors.find(&name.to_string()).unwrap();
        (actor as &Any).downcast_ref::<T>().unwrap()*/
        self.actors.find(&name.to_string()).unwrap().downcast_ref::<T>().unwrap()
    }

    /// Find an actor by registered name
    pub fn find_mut<'a, T: 'static>(&'a mut self, name: &str) -> &'a mut T {
        //FIXME: Rust bug forces us to implement bogus Any for Actor since downcast_ref currently
        //       fails for unknown reasons.
        /*let actor: &mut Actor+Send+Sized = *self.actors.find_mut(&name.to_string()).unwrap();
        (actor as &mut Any).downcast_mut::<T>().unwrap()*/
        self.actors.find_mut(&name.to_string()).unwrap().downcast_mut::<T>().unwrap()
    }

    /// Attempt to process a message as directed by its `to` property. If the actor is not
    /// found or does not indicate that it knew how to process the message, ignore the failure.
    pub fn handle_message(&mut self, msg: &json::JsonObject, stream: &mut TcpStream) {
        let to = msg.find(&"to".to_string()).unwrap().as_string().unwrap();
        match self.actors.find(&to.to_string()) {
            None => println!("message received for unknown actor \"{:s}\"", to),
            Some(actor) => {
                let msg_type = msg.find(&"type".to_string()).unwrap().as_string().unwrap();
                if !actor.handle_message(self, &msg_type.to_string(), msg, stream) {
                    println!("unexpected message type \"{:s}\" found for actor \"{:s}\"",
                             msg_type, to);
                }
            }
        }
        let new_actors = replace(&mut *self.new_actors.borrow_mut(), vec!());
        for actor in new_actors.into_iter() {
            self.actors.insert(actor.name().to_string(), actor);
        }
    }
}
