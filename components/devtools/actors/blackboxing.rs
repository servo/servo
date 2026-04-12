/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;

use crate::ActorMsg;
use crate::actor::{Actor, ActorEncode, ActorRegistry};

#[derive(MallocSizeOf)]
pub(crate) struct BlackboxingActor {
    name: String,
}

impl Actor for BlackboxingActor {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl BlackboxingActor {
    pub fn register(registry: &ActorRegistry) -> String {
        let name = registry.new_name::<Self>();
        let actor = Self { name: name.clone() };
        registry.register::<Self>(actor);
        name
    }
}

impl ActorEncode<ActorMsg> for BlackboxingActor {
    fn encode(&self, _: &ActorRegistry) -> ActorMsg {
        ActorMsg { actor: self.name() }
    }
}
