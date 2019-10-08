/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionAction;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionActionHandler;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionMethods;
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::MediaSessionPlaybackState;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::htmlmediaelement::HTMLMediaElement;
use crate::dom::mediametadata::MediaMetadata;
use crate::dom::window::Window;
use crate::script_thread::ScriptThread;
use dom_struct::dom_struct;
use msg::constellation_msg::TopLevelBrowsingContextId;
use script_traits::{MediaSessionActionType, MediaSessionEvent, ScriptMsg};
use std::collections::HashMap;
use std::rc::Rc;

#[dom_struct]
pub struct MediaSession {
    reflector_: Reflector,
    /// https://w3c.github.io/mediasession/#dom-mediasession-metadata
    metadata: MutNullableDom<MediaMetadata>,
    /// https://w3c.github.io/mediasession/#dom-mediasession-playbackstate
    playback_state: DomRefCell<MediaSessionPlaybackState>,
    /// https://w3c.github.io/mediasession/#supported-media-session-actions
    #[ignore_malloc_size_of = "Rc"]
    action_handlers: DomRefCell<HashMap<MediaSessionActionType, Rc<MediaSessionActionHandler>>>,
    /// The media instance controlled by this media session.
    /// For now only HTMLMediaElements are controlled by media sessions.
    media_instance: MutNullableDom<HTMLMediaElement>,
}

impl MediaSession {
    #[allow(unrooted_must_root)]
    fn new_inherited(browsing_context_id: TopLevelBrowsingContextId) -> MediaSession {
        let media_session = MediaSession {
            reflector_: Reflector::new(),
            metadata: Default::default(),
            playback_state: DomRefCell::new(MediaSessionPlaybackState::None),
            action_handlers: DomRefCell::new(HashMap::new()),
            media_instance: Default::default(),
        };
        ScriptThread::register_media_session(&media_session, browsing_context_id);
        media_session
    }

    pub fn new(window: &Window) -> DomRoot<MediaSession> {
        let browsing_context_id = window.window_proxy().top_level_browsing_context_id();
        reflect_dom_object(
            Box::new(MediaSession::new_inherited(browsing_context_id)),
            window,
            MediaSessionBinding::Wrap,
        )
    }

    pub fn handle_action(&self, action: MediaSessionActionType) {
        if let Some(handler) = self.action_handlers.borrow().get(&action) {
            if handler.Call__(ExceptionHandling::Report).is_err() {
                warn!("Error calling MediaSessionActionHandler callback");
            }
            return;
        }
        // TODO default action.
    }

    pub fn send_event(&self, event: MediaSessionEvent) {
        let global = self.global();
        let browser_id = global
            .as_window()
            .window_proxy()
            .top_level_browsing_context_id();
        let _ = global
            .script_to_constellation_chan()
            .send(ScriptMsg::MediaSessionEventMsg(browser_id, event))
            .unwrap();
    }
}

impl MediaSessionMethods for MediaSession {
    /// https://w3c.github.io/mediasession/#dom-mediasession-metadata
    fn GetMetadata(&self) -> Option<DomRoot<MediaMetadata>> {
        self.metadata.get()
    }

    /// https://w3c.github.io/mediasession/#dom-mediasession-metadata
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

    /// https://w3c.github.io/mediasession/#update-action-handler-algorithm
    fn SetActionHandler(
        &self,
        action: MediaSessionAction,
        handler: Option<Rc<MediaSessionActionHandler>>,
    ) {
        match handler {
            Some(handler) => self
                .action_handlers
                .borrow_mut()
                .insert(action.into(), handler.clone()),
            None => self.action_handlers.borrow_mut().remove(&action.into()),
        };
    }
}

impl Drop for MediaSession {
    fn drop(&mut self) {
        let global = self.global();
        let browsing_context_id = global
            .as_window()
            .window_proxy()
            .top_level_browsing_context_id();
        ScriptThread::remove_media_session(browsing_context_id);
    }
}

impl From<MediaSessionAction> for MediaSessionActionType {
    fn from(action: MediaSessionAction) -> MediaSessionActionType {
        match action {
            MediaSessionAction::Play => MediaSessionActionType::Play,
            MediaSessionAction::Pause => MediaSessionActionType::Pause,
            MediaSessionAction::Seekbackward => MediaSessionActionType::SeekBackward,
            MediaSessionAction::Seekforward => MediaSessionActionType::SeekForward,
            MediaSessionAction::Previoustrack => MediaSessionActionType::PreviousTrack,
            MediaSessionAction::Nexttrack => MediaSessionActionType::NextTrack,
            MediaSessionAction::Skipad => MediaSessionActionType::SkipAd,
            MediaSessionAction::Stop => MediaSessionActionType::Stop,
            MediaSessionAction::Seekto => MediaSessionActionType::SeekTo,
        }
    }
}
