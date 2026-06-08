/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use malloc_size_of_derive::MallocSizeOf;

use crate::actor::{Actor, ActorRegistry, new_actor_name};

#[derive(MallocSizeOf)]
pub(crate) struct AccessibleWalkerActor {
    name: String,
}

impl AccessibleWalkerActor {
    pub fn register(registry: &ActorRegistry) -> Arc<Self> {
        let name = new_actor_name::<Self>();
        let actor = Self { name };
        registry.register::<Self>(actor)
    }
}

impl Actor for AccessibleWalkerActor {
    fn name(&self) -> &str {
        &self.name
    }
}
