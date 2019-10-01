/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionAction;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionActionHandler;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionMethods;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionPlaybackState;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::mediametadata::MediaMetadata;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use std::rc::Rc;

#[dom_struct]
pub struct MediaSession {
    reflector_: Reflector,
    /// https://w3c.github.io/mediasession/#dom-mediasession-metadata
    metadata: MutNullableDom<MediaMetadata>,
    /// https://w3c.github.io/mediasession/#dom-mediasession-playbackstate
    playback_state: DomRefCell<MediaSessionPlaybackState>,
}

impl MediaSession {
    fn new_inherited() -> MediaSession {
        MediaSession {
            reflector_: Reflector::new(),
            metadata: Default::default(),
            playback_state: DomRefCell::new(MediaSessionPlaybackState::None),
        }
    }

    pub fn new(global: &Window) -> DomRoot<MediaSession> {
        reflect_dom_object(
            Box::new(MediaSession::new_inherited()),
            global,
            MediaSessionBinding::Wrap,
        )
    }
}

impl MediaSessionMethods for MediaSession {
    fn GetMetadata(&self) -> Option<DomRoot<MediaMetadata>> {
        self.metadata.get()
    }

    fn SetMetadata(&self, value: Option<&MediaMetadata>) {
        self.metadata.set(value);
    }

    /// https://w3c.github.io/mediasession/#dom-mediasession-playbackstate
    fn PlaybackState(&self) -> MediaSessionPlaybackState {
        *self.playback_state.borrow()
    }

    /// https://w3c.github.io/mediasession/#dom-mediasession-playbackstate
    fn SetPlaybackState(&self, value: MediaSessionPlaybackState) {
        *self.playback_state.borrow_mut() = value;
    }

    fn SetActionHandler(
        &self,
        _action: MediaSessionAction,
        _handler: Option<Rc<MediaSessionActionHandler>>,
    ) {
    }
}
