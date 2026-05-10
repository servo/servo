/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;

use crate::actor::{Actor, ActorRegistry};

/// Referenced by `ThreadActor` when replying to `interupt` messages.
/// <https://searchfox.org/firefox-main/source/devtools/server/actors/thread.js#1699>
#[derive(MallocSizeOf)]
pub(crate) struct PauseActor {
    name: String,
}

impl Actor for PauseActor {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl PauseActor {
    pub fn register(registry: &ActorRegistry) -> String {
        let name = registry.new_name::<Self>();
        let actor = Self { name: name.clone() };
        registry.register::<Self>(actor);
        name
    }
}
