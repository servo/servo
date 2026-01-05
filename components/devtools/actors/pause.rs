/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::actor::Actor;

// TODO: Remove once the actor is used.
#[expect(dead_code)]
/// Referenced by `ThreadActor` when replying to `interupt` messages.
/// <https://searchfox.org/firefox-main/source/devtools/server/actors/thread.js#1699>
pub struct PauseActor {
    pub name: String,
}

impl Actor for PauseActor {
    fn name(&self) -> String {
        self.name.clone()
    }
}
