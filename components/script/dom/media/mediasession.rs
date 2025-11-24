/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use constellation_traits::ScriptToConstellationMessage;
use dom_struct::dom_struct;
use embedder_traits::{
    MediaMetadata as EmbedderMediaMetadata, MediaPositionState as EmbedderMediaPositionState,
    MediaSessionActionType, MediaSessionEvent,
};
use rustc_hash::FxBuildHasher;

use crate::conversions::Convert;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementMethods;
use crate::dom::bindings::codegen::Bindings::MediaMetadataBinding::{
    MediaMetadataInit, MediaMetadataMethods,
};
use crate::dom::bindings::codegen::Bindings::MediaSessionBinding::{
    MediaPositionState, MediaSessionAction, MediaSessionActionHandler, MediaSessionMethods,
    MediaSessionPlaybackState,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::HashMapTracedValues;
use crate::dom::html::htmlmediaelement::HTMLMediaElement;
use crate::dom::media::mediametadata::MediaMetadata;
use crate::dom::window::Window;
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MediaSession {
    reflector_: Reflector,
    /// <https://w3c.github.io/mediasession/#dom-mediasession-metadata>
    #[ignore_malloc_size_of = "defined in embedder_traits"]
    #[no_trace]
    metadata: DomRefCell<Option<EmbedderMediaMetadata>>,
    /// <https://w3c.github.io/mediasession/#dom-mediasession-playbackstate>
    playback_state: DomRefCell<MediaSessionPlaybackState>,
    /// <https://w3c.github.io/mediasession/#supported-media-session-actions>
    #[conditional_malloc_size_of]
    action_handlers: DomRefCell<
        HashMapTracedValues<MediaSessionActionType, Rc<MediaSessionActionHandler>, FxBuildHasher>,
    >,
    /// The media instance controlled by this media session.
    /// For now only HTMLMediaElements are controlled by media sessions.
    media_instance: MutNullableDom<HTMLMediaElement>,
}

impl MediaSession {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited() -> MediaSession {
        MediaSession {
            reflector_: Reflector::new(),
            metadata: DomRefCell::new(None),
            playback_state: DomRefCell::new(MediaSessionPlaybackState::None),
            action_handlers: DomRefCell::new(HashMapTracedValues::new_fx()),
            media_instance: Default::default(),
        }
    }

    pub(crate) fn new(window: &Window, can_gc: CanGc) -> DomRoot<MediaSession> {
        reflect_dom_object(Box::new(MediaSession::new_inherited()), window, can_gc)
    }

    pub(crate) fn register_media_instance(&self, media_instance: &HTMLMediaElement) {
        self.media_instance.set(Some(media_instance));
    }

    pub(crate) fn handle_action(&self, action: MediaSessionActionType, can_gc: CanGc) {
        debug!("Handle media session action {:?}", action);

        if let Some(handler) = self.action_handlers.borrow().get(&action) {
            if handler.Call__(ExceptionHandling::Report, can_gc).is_err() {
                warn!("Error calling MediaSessionActionHandler callback");
            }
            return;
        }

        // Default action.
        if let Some(media) = self.media_instance.get() {
            match action {
                MediaSessionActionType::Play => {
                    let realm = enter_realm(self);
                    media.Play(InRealm::Entered(&realm), can_gc);
                },
                MediaSessionActionType::Pause => {
                    media.Pause(can_gc);
                },
                MediaSessionActionType::SeekBackward => {},
                MediaSessionActionType::SeekForward => {},
                MediaSessionActionType::PreviousTrack => {},
                MediaSessionActionType::NextTrack => {},
                MediaSessionActionType::SkipAd => {},
                MediaSessionActionType::Stop => {},
                MediaSessionActionType::SeekTo => {},
            }
        }
    }

    pub(crate) fn send_event(&self, event: MediaSessionEvent) {
        let global = self.global();
        let window = global.as_window();
        let pipeline_id = window.pipeline_id();
        window.send_to_constellation(ScriptToConstellationMessage::MediaSessionEvent(
            pipeline_id,
            event,
        ));
    }

    pub(crate) fn update_title(&self, title: String) {
        let mut metadata = self.metadata.borrow_mut();
        if let Some(ref mut metadata) = *metadata {
            // We only update the title with the data provided by the media
            // player and iff the user did not provide a title.
            if !metadata.title.is_empty() {
                return;
            }
            metadata.title = title;
        } else {
            *metadata = Some(EmbedderMediaMetadata::new(title));
        }
        self.send_event(MediaSessionEvent::SetMetadata(
            metadata.as_ref().unwrap().clone(),
        ));
    }
}

impl MediaSessionMethods<crate::DomTypeHolder> for MediaSession {
    /// <https://w3c.github.io/mediasession/#dom-mediasession-metadata>
    fn GetMetadata(&self, can_gc: CanGc) -> Option<DomRoot<MediaMetadata>> {
        if let Some(ref metadata) = *self.metadata.borrow() {
            let mut init = MediaMetadataInit::empty();
            init.title = DOMString::from_string(metadata.title.clone());
            init.artist = DOMString::from_string(metadata.artist.clone());
            init.album = DOMString::from_string(metadata.album.clone());
            let global = self.global();
            Some(MediaMetadata::new(global.as_window(), &init, can_gc))
        } else {
            None
        }
    }

    /// <https://w3c.github.io/mediasession/#dom-mediasession-metadata>
    fn SetMetadata(&self, metadata: Option<&MediaMetadata>) {
        if let Some(metadata) = metadata {
            metadata.set_session(self);
        }

        let global = self.global();
        let window = global.as_window();
        let _metadata = match metadata {
            Some(m) => {
                let title = if m.Title().is_empty() {
                    window.get_url().into_string()
                } else {
                    m.Title().into()
                };
                EmbedderMediaMetadata {
                    title,
                    artist: m.Artist().into(),
                    album: m.Album().into(),
                }
            },
            None => EmbedderMediaMetadata::new(window.get_url().into_string()),
        };

        *self.metadata.borrow_mut() = Some(_metadata.clone());

        self.send_event(MediaSessionEvent::SetMetadata(_metadata));
    }

    /// <https://w3c.github.io/mediasession/#dom-mediasession-playbackstate>
    fn PlaybackState(&self) -> MediaSessionPlaybackState {
        *self.playback_state.borrow()
    }

    /// <https://w3c.github.io/mediasession/#dom-mediasession-playbackstate>
    fn SetPlaybackState(&self, state: MediaSessionPlaybackState) {
        *self.playback_state.borrow_mut() = state;
    }

    /// <https://w3c.github.io/mediasession/#update-action-handler-algorithm>
    fn SetActionHandler(
        &self,
        action: MediaSessionAction,
        handler: Option<Rc<MediaSessionActionHandler>>,
    ) {
        match handler {
            Some(handler) => self
                .action_handlers
                .borrow_mut()
                .insert(action.convert(), handler.clone()),
            None => self.action_handlers.borrow_mut().remove(&action.convert()),
        };
    }

    /// <https://w3c.github.io/mediasession/#dom-mediasession-setpositionstate>
    fn SetPositionState(&self, state: &MediaPositionState) -> Fallible<()> {
        // If state is an empty dictionary, clear the position state and abort these steps.
        if state.duration.is_none() && state.position.is_none() && state.playbackRate.is_none() {
            let position_state = EmbedderMediaPositionState::new(0.0, 0.0, 0.0);
            self.send_event(MediaSessionEvent::SetPositionState(position_state));

            return Ok(());
        }

        let duration = if let Some(state_duration) = state.duration {
            // If state’s duration is negative or NaN, throw a TypeError.
            if state_duration < 0.0 || state_duration.is_nan() {
                return Err(Error::Type("Duration is negative or NaN".to_owned()));
            }
            state_duration
        } else {
            // If state’s duration is not present, throw a TypeError.
            return Err(Error::Type("Duration is not present".to_owned()));
        };

        let position = if let Some(state_position) = state.position {
            // If state’s position is negative or greater than duration, throw a TypeError.
            if *state_position < 0.0 || *state_position > duration {
                return Err(Error::Type(
                    "Position is negative or greater than duration".to_owned(),
                ));
            }
            *state_position
        } else {
            // If state’s position is not present, set it to zero.
            0.0
        };

        let playback_rate = if let Some(state_playback_rate) = state.playbackRate {
            // If state’s playbackRate is zero, throw a TypeError.
            if *state_playback_rate == 0.0 {
                return Err(Error::Type("Playback rate is zero".to_owned()));
            }
            *state_playback_rate
        } else {
            // If state’s playbackRate is not present, set it to 1.0.
            1.0
        };

        // Update the position state and last position updated time.
        let position_state = EmbedderMediaPositionState::new(duration, playback_rate, position);
        self.send_event(MediaSessionEvent::SetPositionState(position_state));

        Ok(())
    }
}

impl Convert<MediaSessionActionType> for MediaSessionAction {
    fn convert(self) -> MediaSessionActionType {
        match self {
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
