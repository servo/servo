/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionAction;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionActionHandler;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionMethods;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionPlaybackState;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::mediametadata::MediaMetadata;
use crate::dom::window::Window;
use crate::script_thread::ScriptThread;
use dom_struct::dom_struct;
use msg::constellation_msg::PipelineId;
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
    #[allow(unrooted_must_root)]
    fn new_inherited(pipeline_id: PipelineId) -> MediaSession {
        let media_session = MediaSession {
            reflector_: Reflector::new(),
            metadata: Default::default(),
            playback_state: DomRefCell::new(MediaSessionPlaybackState::None),
        };
        ScriptThread::register_media_session(&media_session, pipeline_id);
        media_session
    }

    pub fn new(window: &Window) -> DomRoot<MediaSession> {
        let pipeline_id = window
            .pipeline_id()
            .expect("Cannot create MediaSession without a PipelineId");
        reflect_dom_object(
            Box::new(MediaSession::new_inherited(pipeline_id)),
            window,
            MediaSessionBinding::Wrap,
        )
    }
}

impl MediaSessionMethods for MediaSession {
    fn GetMetadata(&self) -> Option<DomRoot<MediaMetadata>> {
        self.metadata.get()
    }

    fn SetMetadata(&self, metadata: Option<&MediaMetadata>) {
        if let Some(ref metadata) = metadata {
            metadata.set_session(self);
        }
        self.metadata.set(metadata);
    }

    /// https://w3c.github.io/mediasession/#dom-mediasession-playbackstate
    fn PlaybackState(&self) -> MediaSessionPlaybackState {
        *self.playback_state.borrow()
    }

    /// https://w3c.github.io/mediasession/#dom-mediasession-playbackstate
    fn SetPlaybackState(&self, state: MediaSessionPlaybackState) {
        *self.playback_state.borrow_mut() = state;
    }

    fn SetActionHandler(
        &self,
        _action: MediaSessionAction,
        _handler: Option<Rc<MediaSessionActionHandler>>,
    ) {
    }
}

impl Drop for MediaSession {
    fn drop(&mut self) {
        let global = self.global();
        let pipeline_id = global
            .as_window()
            .pipeline_id()
            .expect("No PipelineId while dropping MediaSession");
        ScriptThread::remove_media_session(pipeline_id);
    }
}
