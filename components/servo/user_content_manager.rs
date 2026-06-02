/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use constellation_traits::{EmbedderToConstellationMessage, UserContentManagerAction};
use embedder_traits::user_contents::{UserContentManagerId, UserScript, UserStyleSheet};

use crate::Servo;

/// The [`UserContentManager`] allows embedders to inject content (scripts, styles) during
/// into the pages loaded within the `WebView`. The same `UserContentManager` can be
/// shared among multiple `WebView`s. Any updates to the `UserContentManager` will
/// take effect only after the page is reloaded.
#[derive(Clone)]
pub struct UserContentManager {
    pub(crate) id: UserContentManagerId,
    pub(crate) servo: Servo,
}

impl UserContentManager {
    pub fn new(servo: &Servo) -> Self {
        Self {
            id: UserContentManagerId::next(),
            servo: servo.clone(),
        }
    }

    pub(crate) fn id(&self) -> UserContentManagerId {
        self.id
    }

    pub fn add_script(&self, user_script: Rc<UserScript>) {
        self.servo.constellation_proxy().send(
            EmbedderToConstellationMessage::UserContentManagerAction(
                self.id,
                UserContentManagerAction::AddUserScript((*user_script).clone()),
            ),
        );
    }

    pub fn remove_script(&self, user_script: Rc<UserScript>) {
        self.servo.constellation_proxy().send(
            EmbedderToConstellationMessage::UserContentManagerAction(
                self.id,
                UserContentManagerAction::RemoveUserScript(user_script.id()),
            ),
        );
    }

    pub fn add_stylesheet(&self, user_stylesheet: Rc<UserStyleSheet>) {
        self.servo.constellation_proxy().send(
            EmbedderToConstellationMessage::UserContentManagerAction(
                self.id,
                UserContentManagerAction::AddUserStyleSheet((*user_stylesheet).clone()),
            ),
        );
    }

    pub fn remove_stylesheet(&self, user_stylesheet: Rc<UserStyleSheet>) {
        self.servo.constellation_proxy().send(
            EmbedderToConstellationMessage::UserContentManagerAction(
                self.id,
                UserContentManagerAction::RemoveUserStyleSheet(user_stylesheet.id()),
            ),
        );
    }
}

impl Drop for UserContentManager {
    fn drop(&mut self) {
        self.servo.constellation_proxy().send(
            EmbedderToConstellationMessage::UserContentManagerAction(
                self.id,
                UserContentManagerAction::DestroyUserContentManager,
            ),
        );
    }
}
