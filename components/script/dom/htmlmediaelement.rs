/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{f64, mem};

use dom_struct::dom_struct;
use embedder_traits::resources::{self, Resource as EmbedderResource};
use embedder_traits::{MediaPositionState, MediaSessionEvent, MediaSessionPlaybackState};
use euclid::default::Size2D;
use headers::{ContentLength, ContentRange, HeaderMapExt};
use html5ever::{LocalName, Prefix, local_name, namespace_url, ns};
use http::StatusCode;
use http::header::{self, HeaderMap, HeaderValue};
use ipc_channel::ipc::{self, IpcSharedMemory, channel};
use ipc_channel::router::ROUTER;
use js::jsapi::JSAutoRealm;
use media::{GLPlayerMsg, GLPlayerMsgForward, WindowGLContext};
use net_traits::request::{Destination, RequestId};
use net_traits::{
    FetchMetadata, FetchResponseListener, Metadata, NetworkError, ResourceFetchTiming,
    ResourceTimingType,
};
use pixels::Image;
use script_bindings::codegen::InheritTypes::{
    ElementTypeId, HTMLElementTypeId, HTMLMediaElementTypeId, NodeTypeId,
};
use script_layout_interface::MediaFrame;
use servo_config::pref;
use servo_media::player::audio::AudioRenderer;
use servo_media::player::video::{VideoFrame, VideoFrameRenderer};
use servo_media::player::{PlaybackState, Player, PlayerError, PlayerEvent, SeekLock, StreamType};
use servo_media::{ClientContextId, ServoMedia, SupportsMediaType};
use servo_url::ServoUrl;
use webrender_api::{
    ExternalImageData, ExternalImageId, ExternalImageType, ImageBufferKind, ImageDescriptor,
    ImageDescriptorFlags, ImageFormat, ImageKey,
};
use webrender_traits::{CrossProcessCompositorApi, ImageUpdate, SerializableImageData};

use crate::document_loader::{LoadBlocker, LoadType};
use crate::dom::attr::Attr;
use crate::dom::audiotrack::AudioTrack;
use crate::dom::audiotracklist::AudioTrackList;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::HTMLMediaElementBinding::{
    CanPlayTypeResult, HTMLMediaElementConstants, HTMLMediaElementMethods,
};
use crate::dom::bindings::codegen::Bindings::HTMLSourceElementBinding::HTMLSourceElementMethods;
use crate::dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorConstants::*;
use crate::dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorMethods;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::Navigator_Binding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::codegen::Bindings::TextTrackBinding::{TextTrackKind, TextTrackMode};
use crate::dom::bindings::codegen::Bindings::URLBinding::URLMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::{
    MediaStreamOrBlob, VideoTrackOrAudioTrackOrTextTrack,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::blob::Blob;
use crate::dom::document::Document;
use crate::dom::element::{
    AttributeMutation, Element, ElementCreator, cors_setting_for_element,
    reflect_cross_origin_attribute, set_cross_origin_attribute,
};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlscriptelement::HTMLScriptElement;
use crate::dom::htmlsourceelement::HTMLSourceElement;
use crate::dom::htmlstyleelement::HTMLStyleElement;
use crate::dom::htmlvideoelement::HTMLVideoElement;
use crate::dom::mediaerror::MediaError;
use crate::dom::mediafragmentparser::MediaFragmentParser;
use crate::dom::mediastream::MediaStream;
use crate::dom::node::{Node, NodeDamage, NodeTraits, UnbindContext};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::shadowroot::IsUserAgentWidget;
use crate::dom::texttrack::TextTrack;
use crate::dom::texttracklist::TextTrackList;
use crate::dom::timeranges::{TimeRanges, TimeRangesContainer};
use crate::dom::trackevent::TrackEvent;
use crate::dom::url::URL;
use crate::dom::videotrack::VideoTrack;
use crate::dom::videotracklist::VideoTrackList;
use crate::dom::virtualmethods::VirtualMethods;
use crate::fetch::{FetchCanceller, create_a_potential_cors_request};
use crate::microtask::{Microtask, MicrotaskRunnable};
use crate::network_listener::{self, PreInvoke, ResourceTimingListener};
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;

#[derive(PartialEq)]
enum FrameStatus {
    Locked,
    Unlocked,
}

struct FrameHolder(FrameStatus, VideoFrame);

impl FrameHolder {
    fn new(frame: VideoFrame) -> FrameHolder {
        FrameHolder(FrameStatus::Unlocked, frame)
    }

    fn lock(&mut self) {
        if self.0 == FrameStatus::Unlocked {
            self.0 = FrameStatus::Locked;
        };
    }

    fn unlock(&mut self) {
        if self.0 == FrameStatus::Locked {
            self.0 = FrameStatus::Unlocked;
        };
    }

    fn set(&mut self, new_frame: VideoFrame) {
        if self.0 == FrameStatus::Unlocked {
            self.1 = new_frame
        };
    }

    fn get(&self) -> (u32, Size2D<i32>, usize) {
        if self.0 == FrameStatus::Locked {
            (
                self.1.get_texture_id(),
                Size2D::new(self.1.get_width(), self.1.get_height()),
                0,
            )
        } else {
            unreachable!();
        }
    }

    fn get_frame(&self) -> VideoFrame {
        self.1.clone()
    }
}

pub(crate) struct MediaFrameRenderer {
    player_id: Option<u64>,
    compositor_api: CrossProcessCompositorApi,
    current_frame: Option<MediaFrame>,
    old_frame: Option<ImageKey>,
    very_old_frame: Option<ImageKey>,
    current_frame_holder: Option<FrameHolder>,
    show_poster: bool,
}

impl MediaFrameRenderer {
    fn new(compositor_api: CrossProcessCompositorApi) -> Self {
        Self {
            player_id: None,
            compositor_api,
            current_frame: None,
            old_frame: None,
            very_old_frame: None,
            current_frame_holder: None,
            show_poster: false,
        }
    }

    fn render_poster_frame(&mut self, image: Arc<Image>) {
        if let Some(image_key) = image.id {
            self.current_frame = Some(MediaFrame {
                image_key,
                width: image.width as i32,
                height: image.height as i32,
            });
            self.show_poster = true;
        }
    }
}

impl VideoFrameRenderer for MediaFrameRenderer {
    fn render(&mut self, frame: VideoFrame) {
        // Don't render new frames if the poster should be shown
        if self.show_poster {
            return;
        }

        let mut updates = vec![];

        if let Some(old_image_key) = mem::replace(&mut self.very_old_frame, self.old_frame.take()) {
            updates.push(ImageUpdate::DeleteImage(old_image_key));
        }

        let descriptor = ImageDescriptor::new(
            frame.get_width(),
            frame.get_height(),
            ImageFormat::BGRA8,
            ImageDescriptorFlags::empty(),
        );

        match &mut self.current_frame {
            Some(current_frame)
                if current_frame.width == frame.get_width() &&
                    current_frame.height == frame.get_height() =>
            {
                if !frame.is_gl_texture() {
                    updates.push(ImageUpdate::UpdateImage(
                        current_frame.image_key,
                        descriptor,
                        SerializableImageData::Raw(IpcSharedMemory::from_bytes(&frame.get_data())),
                    ));
                }

                self.current_frame_holder
                    .get_or_insert_with(|| FrameHolder::new(frame.clone()))
                    .set(frame);

                if let Some(old_image_key) = self.old_frame.take() {
                    updates.push(ImageUpdate::DeleteImage(old_image_key));
                }
            },
            Some(current_frame) => {
                self.old_frame = Some(current_frame.image_key);

                let Some(new_image_key) = self.compositor_api.generate_image_key() else {
                    return;
                };

                /* update current_frame */
                current_frame.image_key = new_image_key;
                current_frame.width = frame.get_width();
                current_frame.height = frame.get_height();

                let image_data = if frame.is_gl_texture() && self.player_id.is_some() {
                    let texture_target = if frame.is_external_oes() {
                        ImageBufferKind::TextureExternal
                    } else {
                        ImageBufferKind::Texture2D
                    };

                    SerializableImageData::External(ExternalImageData {
                        id: ExternalImageId(self.player_id.unwrap()),
                        channel_index: 0,
                        image_type: ExternalImageType::TextureHandle(texture_target),
                        normalized_uvs: false,
                    })
                } else {
                    SerializableImageData::Raw(IpcSharedMemory::from_bytes(&frame.get_data()))
                };

                self.current_frame_holder
                    .get_or_insert_with(|| FrameHolder::new(frame.clone()))
                    .set(frame);

                updates.push(ImageUpdate::AddImage(new_image_key, descriptor, image_data));
            },
            None => {
                let Some(image_key) = self.compositor_api.generate_image_key() else {
                    return;
                };

                self.current_frame = Some(MediaFrame {
                    image_key,
                    width: frame.get_width(),
                    height: frame.get_height(),
                });

                let image_data = if frame.is_gl_texture() && self.player_id.is_some() {
                    let texture_target = if frame.is_external_oes() {
                        ImageBufferKind::TextureExternal
                    } else {
                        ImageBufferKind::Texture2D
                    };

                    SerializableImageData::External(ExternalImageData {
                        id: ExternalImageId(self.player_id.unwrap()),
                        channel_index: 0,
                        image_type: ExternalImageType::TextureHandle(texture_target),
                        normalized_uvs: false,
                    })
                } else {
                    SerializableImageData::Raw(IpcSharedMemory::from_bytes(&frame.get_data()))
                };

                self.current_frame_holder = Some(FrameHolder::new(frame));

                updates.push(ImageUpdate::AddImage(image_key, descriptor, image_data));
            },
        }
        self.compositor_api.update_images(updates);
    }
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
enum SrcObject {
    MediaStream(Dom<MediaStream>),
    Blob(Dom<Blob>),
}

impl From<MediaStreamOrBlob> for SrcObject {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn from(src_object: MediaStreamOrBlob) -> SrcObject {
        match src_object {
            MediaStreamOrBlob::Blob(blob) => SrcObject::Blob(Dom::from_ref(&*blob)),
            MediaStreamOrBlob::MediaStream(stream) => {
                SrcObject::MediaStream(Dom::from_ref(&*stream))
            },
        }
    }
}

#[dom_struct]
#[allow(non_snake_case)]
pub(crate) struct HTMLMediaElement {
    htmlelement: HTMLElement,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-networkstate>
    network_state: Cell<NetworkState>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-readystate>
    ready_state: Cell<ReadyState>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-srcobject>
    src_object: DomRefCell<Option<SrcObject>>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-currentsrc>
    current_src: DomRefCell<String>,
    /// Incremented whenever tasks associated with this element are cancelled.
    generation_id: Cell<u32>,
    /// <https://html.spec.whatwg.org/multipage/#fire-loadeddata>
    ///
    /// Reset to false every time the load algorithm is invoked.
    fired_loadeddata_event: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-error>
    error: MutNullableDom<MediaError>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-paused>
    paused: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-defaultplaybackrate>
    defaultPlaybackRate: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-playbackrate>
    playbackRate: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#attr-media-autoplay>
    autoplaying: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#delaying-the-load-event-flag>
    delaying_the_load_event_flag: DomRefCell<Option<LoadBlocker>>,
    /// <https://html.spec.whatwg.org/multipage/#list-of-pending-play-promises>
    #[ignore_malloc_size_of = "promises are hard"]
    pending_play_promises: DomRefCell<Vec<Rc<Promise>>>,
    /// Play promises which are soon to be fulfilled by a queued task.
    #[allow(clippy::type_complexity)]
    #[ignore_malloc_size_of = "promises are hard"]
    in_flight_play_promises_queue: DomRefCell<VecDeque<(Box<[Rc<Promise>]>, ErrorResult)>>,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    player: DomRefCell<Option<Arc<Mutex<dyn Player>>>>,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    video_renderer: Arc<Mutex<MediaFrameRenderer>>,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    audio_renderer: DomRefCell<Option<Arc<Mutex<dyn AudioRenderer>>>>,
    /// <https://html.spec.whatwg.org/multipage/#show-poster-flag>
    show_poster: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-duration>
    duration: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#official-playback-position>
    playback_position: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#default-playback-start-position>
    default_playback_start_position: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-volume>
    volume: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-seeking>
    seeking: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-muted>
    muted: Cell<bool>,
    /// URL of the media resource, if any.
    #[no_trace]
    resource_url: DomRefCell<Option<ServoUrl>>,
    /// URL of the media resource, if the resource is set through the src_object attribute and it
    /// is a blob.
    #[no_trace]
    blob_url: DomRefCell<Option<ServoUrl>>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-played>
    #[ignore_malloc_size_of = "Rc"]
    played: DomRefCell<TimeRangesContainer>,
    // https://html.spec.whatwg.org/multipage/#dom-media-audiotracks
    audio_tracks_list: MutNullableDom<AudioTrackList>,
    // https://html.spec.whatwg.org/multipage/#dom-media-videotracks
    video_tracks_list: MutNullableDom<VideoTrackList>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-texttracks>
    text_tracks_list: MutNullableDom<TextTrackList>,
    /// Time of last timeupdate notification.
    #[ignore_malloc_size_of = "Defined in std::time"]
    next_timeupdate_event: Cell<Instant>,
    /// Latest fetch request context.
    current_fetch_context: DomRefCell<Option<HTMLMediaElementFetchContext>>,
    /// Player Id reported the player thread
    id: Cell<u64>,
    /// Media controls id.
    /// In order to workaround the lack of privileged JS context, we secure the
    /// the access to the "privileged" document.servoGetMediaControls(id) API by
    /// keeping a whitelist of media controls identifiers.
    media_controls_id: DomRefCell<Option<String>>,
    #[ignore_malloc_size_of = "Defined in other crates"]
    #[no_trace]
    player_context: WindowGLContext,
}

/// <https://html.spec.whatwg.org/multipage/#dom-media-networkstate>
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
#[repr(u8)]
pub(crate) enum NetworkState {
    Empty = HTMLMediaElementConstants::NETWORK_EMPTY as u8,
    Idle = HTMLMediaElementConstants::NETWORK_IDLE as u8,
    Loading = HTMLMediaElementConstants::NETWORK_LOADING as u8,
    NoSource = HTMLMediaElementConstants::NETWORK_NO_SOURCE as u8,
}

/// <https://html.spec.whatwg.org/multipage/#dom-media-readystate>
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq, PartialOrd)]
#[repr(u8)]
#[allow(clippy::enum_variant_names)] // Clippy warning silenced here because these names are from the specification.
pub(crate) enum ReadyState {
    HaveNothing = HTMLMediaElementConstants::HAVE_NOTHING as u8,
    HaveMetadata = HTMLMediaElementConstants::HAVE_METADATA as u8,
    HaveCurrentData = HTMLMediaElementConstants::HAVE_CURRENT_DATA as u8,
    HaveFutureData = HTMLMediaElementConstants::HAVE_FUTURE_DATA as u8,
    HaveEnoughData = HTMLMediaElementConstants::HAVE_ENOUGH_DATA as u8,
}

impl HTMLMediaElement {
    pub(crate) fn new_inherited(
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> Self {
        Self {
            htmlelement: HTMLElement::new_inherited(tag_name, prefix, document),
            network_state: Cell::new(NetworkState::Empty),
            ready_state: Cell::new(ReadyState::HaveNothing),
            src_object: Default::default(),
            current_src: DomRefCell::new("".to_owned()),
            generation_id: Cell::new(0),
            fired_loadeddata_event: Cell::new(false),
            error: Default::default(),
            paused: Cell::new(true),
            defaultPlaybackRate: Cell::new(1.0),
            playbackRate: Cell::new(1.0),
            muted: Cell::new(false),
            // FIXME(nox): Why is this initialised to true?
            autoplaying: Cell::new(true),
            delaying_the_load_event_flag: Default::default(),
            pending_play_promises: Default::default(),
            in_flight_play_promises_queue: Default::default(),
            player: Default::default(),
            video_renderer: Arc::new(Mutex::new(MediaFrameRenderer::new(
                document.window().compositor_api().clone(),
            ))),
            audio_renderer: Default::default(),
            show_poster: Cell::new(true),
            duration: Cell::new(f64::NAN),
            playback_position: Cell::new(0.),
            default_playback_start_position: Cell::new(0.),
            volume: Cell::new(1.0),
            seeking: Cell::new(false),
            resource_url: DomRefCell::new(None),
            blob_url: DomRefCell::new(None),
            played: DomRefCell::new(TimeRangesContainer::default()),
            audio_tracks_list: Default::default(),
            video_tracks_list: Default::default(),
            text_tracks_list: Default::default(),
            next_timeupdate_event: Cell::new(Instant::now() + Duration::from_millis(250)),
            current_fetch_context: DomRefCell::new(None),
            id: Cell::new(0),
            media_controls_id: DomRefCell::new(None),
            player_context: document.window().get_player_context(),
        }
    }

    pub(crate) fn get_ready_state(&self) -> ReadyState {
        self.ready_state.get()
    }

    fn media_type_id(&self) -> HTMLMediaElementTypeId {
        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLMediaElement(media_type_id),
            )) => media_type_id,
            _ => unreachable!(),
        }
    }

    fn play_media(&self) {
        if let Some(ref player) = *self.player.borrow() {
            if let Err(e) = player.lock().unwrap().set_rate(self.playbackRate.get()) {
                warn!("Could not set the playback rate {:?}", e);
            }
            if let Err(e) = player.lock().unwrap().play() {
                warn!("Could not play media {:?}", e);
            }
        }
    }

    /// Marks that element as delaying the load event or not.
    ///
    /// Nothing happens if the element was already delaying the load event and
    /// we pass true to that method again.
    ///
    /// <https://html.spec.whatwg.org/multipage/#delaying-the-load-event-flag>
    pub(crate) fn delay_load_event(&self, delay: bool, can_gc: CanGc) {
        let blocker = &self.delaying_the_load_event_flag;
        if delay && blocker.borrow().is_none() {
            *blocker.borrow_mut() = Some(LoadBlocker::new(&self.owner_document(), LoadType::Media));
        } else if !delay && blocker.borrow().is_some() {
            LoadBlocker::terminate(blocker, can_gc);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#time-marches-on>
    fn time_marches_on(&self) {
        // Step 6.
        if Instant::now() > self.next_timeupdate_event.get() {
            self.owner_global()
                .task_manager()
                .media_element_task_source()
                .queue_simple_event(self.upcast(), atom!("timeupdate"));
            self.next_timeupdate_event
                .set(Instant::now() + Duration::from_millis(350));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#internal-pause-steps>
    fn internal_pause_steps(&self) {
        // Step 1.
        self.autoplaying.set(false);

        // Step 2.
        if !self.Paused() {
            // Step 2.1.
            self.paused.set(true);

            // Step 2.2.
            self.take_pending_play_promises(Err(Error::Abort));

            // Step 2.3.
            let this = Trusted::new(self);
            let generation_id = self.generation_id.get();
            self.owner_global()
                .task_manager()
                .media_element_task_source()
                .queue(task!(internal_pause_steps: move || {
                    let this = this.root();
                    if generation_id != this.generation_id.get() {
                        return;
                    }

                    this.fulfill_in_flight_play_promises(|| {
                        // Step 2.3.1.
                        this.upcast::<EventTarget>().fire_event(atom!("timeupdate"), CanGc::note());

                        // Step 2.3.2.
                        this.upcast::<EventTarget>().fire_event(atom!("pause"), CanGc::note());

                        if let Some(ref player) = *this.player.borrow() {
                            if let Err(e) = player.lock().unwrap().pause() {
                                eprintln!("Could not pause player {:?}", e);
                            }
                        }

                        // Step 2.3.3.
                        // Done after running this closure in
                        // `fulfill_in_flight_play_promises`.
                    });
                }));

            // Step 2.4.
            // FIXME(nox): Set the official playback position to the current
            // playback position.
        }
    }
    // https://html.spec.whatwg.org/multipage/#allowed-to-play
    fn is_allowed_to_play(&self) -> bool {
        true
    }

    // https://html.spec.whatwg.org/multipage/#notify-about-playing
    fn notify_about_playing(&self) {
        // Step 1.
        self.take_pending_play_promises(Ok(()));

        // Step 2.
        let this = Trusted::new(self);
        let generation_id = self.generation_id.get();
        self.owner_global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(notify_about_playing: move || {
                let this = this.root();
                if generation_id != this.generation_id.get() {
                    return;
                }

                this.fulfill_in_flight_play_promises(|| {
                    // Step 2.1.
                    this.upcast::<EventTarget>().fire_event(atom!("playing"), CanGc::note());
                    this.play_media();

                    // Step 2.2.
                    // Done after running this closure in
                    // `fulfill_in_flight_play_promises`.
                });

            }));
    }

    // https://html.spec.whatwg.org/multipage/#ready-states
    fn change_ready_state(&self, ready_state: ReadyState) {
        let old_ready_state = self.ready_state.get();
        self.ready_state.set(ready_state);

        if self.network_state.get() == NetworkState::Empty {
            return;
        }

        if old_ready_state == ready_state {
            return;
        }

        let owner_global = self.owner_global();
        let task_manager = owner_global.task_manager();
        let task_source = task_manager.media_element_task_source();

        // Step 1.
        match (old_ready_state, ready_state) {
            (ReadyState::HaveNothing, ReadyState::HaveMetadata) => {
                task_source.queue_simple_event(self.upcast(), atom!("loadedmetadata"));
                // No other steps are applicable in this case.
                return;
            },
            (ReadyState::HaveMetadata, new) if new >= ReadyState::HaveCurrentData => {
                if !self.fired_loadeddata_event.get() {
                    self.fired_loadeddata_event.set(true);
                    let this = Trusted::new(self);
                    task_source.queue(task!(media_reached_current_data: move || {
                        let this = this.root();
                        this.upcast::<EventTarget>().fire_event(atom!("loadeddata"), CanGc::note());
                        this.delay_load_event(false, CanGc::note());
                    }));
                }

                // Steps for the transition from HaveMetadata to HaveCurrentData
                // or HaveFutureData also apply here, as per the next match
                // expression.
            },
            (ReadyState::HaveFutureData, new) if new <= ReadyState::HaveCurrentData => {
                // FIXME(nox): Queue a task to fire timeupdate and waiting
                // events if the conditions call from the spec are met.

                // No other steps are applicable in this case.
                return;
            },

            _ => (),
        }

        if old_ready_state <= ReadyState::HaveCurrentData &&
            ready_state >= ReadyState::HaveFutureData
        {
            task_source.queue_simple_event(self.upcast(), atom!("canplay"));

            if !self.Paused() {
                self.notify_about_playing();
            }
        }

        if ready_state == ReadyState::HaveEnoughData {
            // TODO: Check sandboxed automatic features browsing context flag.
            // FIXME(nox): I have no idea what this TODO is about.

            // FIXME(nox): Review this block.
            if self.autoplaying.get() && self.Paused() && self.Autoplay() {
                // Step 1
                self.paused.set(false);
                // Step 2
                if self.show_poster.get() {
                    self.set_show_poster(false);
                    self.time_marches_on();
                }
                // Step 3
                task_source.queue_simple_event(self.upcast(), atom!("play"));
                // Step 4
                self.notify_about_playing();
                // Step 5
                self.autoplaying.set(false);
            }

            // FIXME(nox): According to the spec, this should come *before* the
            // "play" event.
            task_source.queue_simple_event(self.upcast(), atom!("canplaythrough"));
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn invoke_resource_selection_algorithm(&self, can_gc: CanGc) {
        // Step 1.
        self.network_state.set(NetworkState::NoSource);

        // Step 2.
        self.set_show_poster(true);

        // Step 3.
        self.delay_load_event(true, can_gc);

        // Step 4.
        // If the resource selection mode in the synchronous section is
        // "attribute", the URL of the resource to fetch is relative to the
        // media element's node document when the src attribute was last
        // changed, which is why we need to pass the base URL in the task
        // right here.
        let doc = self.owner_document();
        let task = MediaElementMicrotask::ResourceSelection {
            elem: DomRoot::from_ref(self),
            generation_id: self.generation_id.get(),
            base_url: doc.base_url(),
        };

        // FIXME(nox): This will later call the resource_selection_algorithm_sync
        // method from below, if microtasks were trait objects, we would be able
        // to put the code directly in this method, without the boilerplate
        // indirections.
        ScriptThread::await_stable_state(Microtask::MediaElement(task));
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn resource_selection_algorithm_sync(&self, base_url: ServoUrl, can_gc: CanGc) {
        // Step 5.
        // FIXME(ferjm): Implement blocked_on_parser logic
        // https://html.spec.whatwg.org/multipage/#blocked-on-parser
        // FIXME(nox): Maybe populate the list of pending text tracks.

        // Step 6.
        enum Mode {
            Object,
            Attribute(String),
            Children(DomRoot<HTMLSourceElement>),
        }
        fn mode(media: &HTMLMediaElement) -> Option<Mode> {
            if media.src_object.borrow().is_some() {
                return Some(Mode::Object);
            }
            if let Some(attr) = media
                .upcast::<Element>()
                .get_attribute(&ns!(), &local_name!("src"))
            {
                return Some(Mode::Attribute(attr.Value().into()));
            }
            let source_child_element = media
                .upcast::<Node>()
                .children()
                .filter_map(DomRoot::downcast::<HTMLSourceElement>)
                .next();
            if let Some(element) = source_child_element {
                return Some(Mode::Children(element));
            }
            None
        }
        let mode = if let Some(mode) = mode(self) {
            mode
        } else {
            self.network_state.set(NetworkState::Empty);
            // https://github.com/whatwg/html/issues/3065
            self.delay_load_event(false, can_gc);
            return;
        };

        // Step 7.
        self.network_state.set(NetworkState::Loading);

        // Step 8.
        self.owner_global()
            .task_manager()
            .media_element_task_source()
            .queue_simple_event(self.upcast(), atom!("loadstart"));

        // Step 9.
        match mode {
            // Step 9.obj.
            Mode::Object => {
                // Step 9.obj.1.
                "".clone_into(&mut self.current_src.borrow_mut());

                // Step 9.obj.2.
                // FIXME(nox): The rest of the steps should be ran in parallel.

                // Step 9.obj.3.
                // Note that the resource fetch algorithm itself takes care
                // of the cleanup in case of failure itself.
                self.resource_fetch_algorithm(Resource::Object);
            },
            Mode::Attribute(src) => {
                // Step 9.attr.1.
                if src.is_empty() {
                    self.queue_dedicated_media_source_failure_steps();
                    return;
                }

                // Step 9.attr.2.
                let url_record = match base_url.join(&src) {
                    Ok(url) => url,
                    Err(_) => {
                        self.queue_dedicated_media_source_failure_steps();
                        return;
                    },
                };

                // Step 9.attr.3.
                *self.current_src.borrow_mut() = url_record.as_str().into();

                // Step 9.attr.4.
                // Note that the resource fetch algorithm itself takes care
                // of the cleanup in case of failure itself.
                self.resource_fetch_algorithm(Resource::Url(url_record));
            },
            // Step 9.children.
            Mode::Children(source) => {
                // This is only a partial implementation
                // FIXME: https://github.com/servo/servo/issues/21481
                let src = source.Src();
                // Step 9.attr.2.
                if src.is_empty() {
                    source
                        .upcast::<EventTarget>()
                        .fire_event(atom!("error"), can_gc);
                    self.queue_dedicated_media_source_failure_steps();
                    return;
                }
                // Step 9.attr.3.
                let url_record = match base_url.join(&src) {
                    Ok(url) => url,
                    Err(_) => {
                        source
                            .upcast::<EventTarget>()
                            .fire_event(atom!("error"), can_gc);
                        self.queue_dedicated_media_source_failure_steps();
                        return;
                    },
                };
                // Step 9.attr.8.
                self.resource_fetch_algorithm(Resource::Url(url_record));
            },
        }
    }

    fn fetch_request(&self, offset: Option<u64>, seek_lock: Option<SeekLock>) {
        if self.resource_url.borrow().is_none() && self.blob_url.borrow().is_none() {
            eprintln!("Missing request url");
            self.queue_dedicated_media_source_failure_steps();
            return;
        }

        let document = self.owner_document();
        let destination = match self.media_type_id() {
            HTMLMediaElementTypeId::HTMLAudioElement => Destination::Audio,
            HTMLMediaElementTypeId::HTMLVideoElement => Destination::Video,
        };
        let mut headers = HeaderMap::new();
        // FIXME(eijebong): Use typed headers once we have a constructor for the range header
        headers.insert(
            header::RANGE,
            HeaderValue::from_str(&format!("bytes={}-", offset.unwrap_or(0))).unwrap(),
        );
        let url = match self.resource_url.borrow().as_ref() {
            Some(url) => url.clone(),
            None => self.blob_url.borrow().as_ref().unwrap().clone(),
        };

        let cors_setting = cors_setting_for_element(self.upcast());
        let request = create_a_potential_cors_request(
            Some(document.webview_id()),
            url.clone(),
            destination,
            cors_setting,
            None,
            self.global().get_referrer(),
            document.insecure_requests_policy(),
            document.has_trustworthy_ancestor_or_current_origin(),
        )
        .headers(headers)
        .origin(document.origin().immutable().clone())
        .pipeline_id(Some(self.global().pipeline_id()))
        .referrer_policy(document.get_referrer_policy());

        let mut current_fetch_context = self.current_fetch_context.borrow_mut();
        if let Some(ref mut current_fetch_context) = *current_fetch_context {
            current_fetch_context.cancel(CancelReason::Overridden);
        }

        *current_fetch_context = Some(HTMLMediaElementFetchContext::new(request.id));
        let listener =
            HTMLMediaElementFetchListener::new(self, url.clone(), offset.unwrap_or(0), seek_lock);

        self.owner_document().fetch_background(request, listener);
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-resource
    fn resource_fetch_algorithm(&self, resource: Resource) {
        if let Err(e) = self.setup_media_player(&resource) {
            eprintln!("Setup media player error {:?}", e);
            self.queue_dedicated_media_source_failure_steps();
            return;
        }

        // Steps 1-2.
        // Unapplicable, the `resource` variable already conveys which mode
        // is in use.

        // Step 3.
        // FIXME(nox): Remove all media-resource-specific text tracks.

        // Step 4.
        match resource {
            Resource::Url(url) => {
                // Step 4.remote.1.
                if self.Preload() == "none" && !self.autoplaying.get() {
                    // Step 4.remote.1.1.
                    self.network_state.set(NetworkState::Idle);

                    // Step 4.remote.1.2.
                    let owner_global = self.owner_global();
                    let task_manager = owner_global.task_manager();
                    let task_source = task_manager.media_element_task_source();
                    task_source.queue_simple_event(self.upcast(), atom!("suspend"));

                    // Step 4.remote.1.3.
                    let this = Trusted::new(self);
                    task_source.queue(task!(set_media_delay_load_event_flag_to_false: move || {
                        this.root().delay_load_event(false, CanGc::note());
                    }));

                    // Steps 4.remote.1.4.
                    // FIXME(nox): Somehow we should wait for the task from previous
                    // step to be ran before continuing.

                    // Steps 4.remote.1.5-4.remote.1.7.
                    // FIXME(nox): Wait for an implementation-defined event and
                    // then continue with the normal set of steps instead of just
                    // returning.
                    return;
                }

                // Step 4.remote.2.
                *self.resource_url.borrow_mut() = Some(url);
                self.fetch_request(None, None);
            },
            Resource::Object => {
                if let Some(ref src_object) = *self.src_object.borrow() {
                    match src_object {
                        SrcObject::Blob(blob) => {
                            let blob_url = URL::CreateObjectURL(&self.global(), blob);
                            *self.blob_url.borrow_mut() =
                                Some(ServoUrl::parse(&blob_url).expect("infallible"));
                            self.fetch_request(None, None);
                        },
                        SrcObject::MediaStream(stream) => {
                            let tracks = &*stream.get_tracks();
                            for (pos, track) in tracks.iter().enumerate() {
                                if self
                                    .player
                                    .borrow()
                                    .as_ref()
                                    .unwrap()
                                    .lock()
                                    .unwrap()
                                    .set_stream(&track.id(), pos == tracks.len() - 1)
                                    .is_err()
                                {
                                    self.queue_dedicated_media_source_failure_steps();
                                }
                            }
                        },
                    }
                }
            },
        }
    }

    /// Queues a task to run the [dedicated media source failure steps][steps].
    ///
    /// [steps]: https://html.spec.whatwg.org/multipage/#dedicated-media-source-failure-steps
    fn queue_dedicated_media_source_failure_steps(&self) {
        let this = Trusted::new(self);
        let generation_id = self.generation_id.get();
        self.take_pending_play_promises(Err(Error::NotSupported));
        self.owner_global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(dedicated_media_source_failure_steps: move || {
                let this = this.root();
                if generation_id != this.generation_id.get() {
                    return;
                }

                this.fulfill_in_flight_play_promises(|| {
                    // Step 1.
                    this.error.set(Some(&*MediaError::new(
                        &this.owner_window(),
                        MEDIA_ERR_SRC_NOT_SUPPORTED, CanGc::note())));

                    // Step 2.
                    this.AudioTracks().clear();
                    this.VideoTracks().clear();

                    // Step 3.
                    this.network_state.set(NetworkState::NoSource);

                    // Step 4.
                    this.set_show_poster(true);

                    // Step 5.
                    this.upcast::<EventTarget>().fire_event(atom!("error"), CanGc::note());

                    if let Some(ref player) = *this.player.borrow() {
                        if let Err(e) = player.lock().unwrap().stop() {
                            eprintln!("Could not stop player {:?}", e);
                        }
                    }

                    // Step 6.
                    // Done after running this closure in
                    // `fulfill_in_flight_play_promises`.
                });

                // Step 7.
                this.delay_load_event(false, CanGc::note());
            }));
    }

    fn queue_ratechange_event(&self) {
        self.owner_global()
            .task_manager()
            .media_element_task_source()
            .queue_simple_event(self.upcast(), atom!("ratechange"));
    }

    fn in_error_state(&self) -> bool {
        self.error.get().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#potentially-playing>
    fn is_potentially_playing(&self) -> bool {
        !self.paused.get() &&
            !self.Ended() &&
            self.error.get().is_none() &&
            !self.is_blocked_media_element()
    }

    // https://html.spec.whatwg.org/multipage/#blocked-media-element
    fn is_blocked_media_element(&self) -> bool {
        self.ready_state.get() <= ReadyState::HaveCurrentData ||
            self.is_paused_for_user_interaction() ||
            self.is_paused_for_in_band_content()
    }

    // https://html.spec.whatwg.org/multipage/#paused-for-user-interaction
    fn is_paused_for_user_interaction(&self) -> bool {
        // FIXME: we will likely be able to fill this placeholder once (if) we
        //        implement the MediaSession API.
        false
    }

    // https://html.spec.whatwg.org/multipage/#paused-for-in-band-content
    fn is_paused_for_in_band_content(&self) -> bool {
        // FIXME: we will likely be able to fill this placeholder once (if) we
        //        implement https://github.com/servo/servo/issues/22314
        false
    }

    // https://html.spec.whatwg.org/multipage/#media-element-load-algorithm
    fn media_element_load_algorithm(&self, can_gc: CanGc) {
        // Reset the flag that signals whether loadeddata was ever fired for
        // this invokation of the load algorithm.
        self.fired_loadeddata_event.set(false);

        // Step 1-2.
        self.generation_id.set(self.generation_id.get() + 1);

        // Steps 3-4.
        while !self.in_flight_play_promises_queue.borrow().is_empty() {
            self.fulfill_in_flight_play_promises(|| ());
        }

        let global = self.owner_global();
        let task_manager = global.task_manager();
        let task_source = task_manager.media_element_task_source();

        // Step 5.
        let network_state = self.network_state.get();
        if network_state == NetworkState::Loading || network_state == NetworkState::Idle {
            task_source.queue_simple_event(self.upcast(), atom!("abort"));
        }

        // Step 6.
        if network_state != NetworkState::Empty {
            // Step 6.1.
            task_source.queue_simple_event(self.upcast(), atom!("emptied"));

            // Step 6.2.
            if let Some(ref mut current_fetch_context) = *self.current_fetch_context.borrow_mut() {
                current_fetch_context.cancel(CancelReason::Error);
            }

            // Step 6.3.
            // FIXME(nox): Detach MediaSource media provider object.

            // Step 6.4.
            self.AudioTracks().clear();
            self.VideoTracks().clear();

            // Step 6.5.
            if self.ready_state.get() != ReadyState::HaveNothing {
                self.change_ready_state(ReadyState::HaveNothing);
            }

            // Step 6.6.
            if !self.Paused() {
                // Step 6.6.1.
                self.paused.set(true);

                // Step 6.6.2.
                self.take_pending_play_promises(Err(Error::Abort));
                self.fulfill_in_flight_play_promises(|| ());
            }

            // Step 6.7.
            if !self.seeking.get() {
                self.seeking.set(false);
            }

            // Step 6.8.
            let queue_timeupdate_event = self.playback_position.get() != 0.;
            self.playback_position.set(0.);
            if queue_timeupdate_event {
                task_source.queue_simple_event(self.upcast(), atom!("timeupdate"));
            }

            // Step 6.9.
            // FIXME(nox): Set timeline offset to NaN.

            // Step 6.10.
            self.duration.set(f64::NAN);
        }

        // Step 7.
        self.playbackRate.set(self.defaultPlaybackRate.get());

        // Step 8.
        self.error.set(None);
        self.autoplaying.set(true);

        // Step 9.
        self.invoke_resource_selection_algorithm(can_gc);

        // Step 10.
        // FIXME(nox): Stop playback of any previously running media resource.
    }

    /// Appends a promise to the list of pending play promises.
    fn push_pending_play_promise(&self, promise: &Rc<Promise>) {
        self.pending_play_promises
            .borrow_mut()
            .push(promise.clone());
    }

    /// Takes the pending play promises.
    ///
    /// The result with which these promises will be fulfilled is passed here
    /// and this method returns nothing because we actually just move the
    /// current list of pending play promises to the
    /// `in_flight_play_promises_queue` field.
    ///
    /// Each call to this method must be followed by a call to
    /// `fulfill_in_flight_play_promises`, to actually fulfill the promises
    /// which were taken and moved to the in-flight queue.
    fn take_pending_play_promises(&self, result: ErrorResult) {
        let pending_play_promises = std::mem::take(&mut *self.pending_play_promises.borrow_mut());
        self.in_flight_play_promises_queue
            .borrow_mut()
            .push_back((pending_play_promises.into(), result));
    }

    /// Fulfills the next in-flight play promises queue after running a closure.
    ///
    /// See the comment on `take_pending_play_promises` for why this method
    /// does not take a list of promises to fulfill. Callers cannot just pop
    /// the front list off of `in_flight_play_promises_queue` and later fulfill
    /// the promises because that would mean putting
    /// `#[cfg_attr(crown, allow(crown::unrooted_must_root))]` on even more functions, potentially
    /// hiding actual safety bugs.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn fulfill_in_flight_play_promises<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        let (promises, result) = self
            .in_flight_play_promises_queue
            .borrow_mut()
            .pop_front()
            .expect("there should be at least one list of in flight play promises");
        f();
        for promise in &*promises {
            match result {
                Ok(ref value) => promise.resolve_native(value, CanGc::note()),
                Err(ref error) => promise.reject_error(error.clone(), CanGc::note()),
            }
        }
    }

    /// Handles insertion of `source` children.
    ///
    /// <https://html.spec.whatwg.org/multipage/#the-source-element:nodes-are-inserted>
    pub(crate) fn handle_source_child_insertion(&self, can_gc: CanGc) {
        if self.upcast::<Element>().has_attribute(&local_name!("src")) {
            return;
        }
        if self.network_state.get() != NetworkState::Empty {
            return;
        }
        self.media_element_load_algorithm(can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-seek
    fn seek(&self, time: f64, _approximate_for_speed: bool) {
        // Step 1.
        self.set_show_poster(false);

        // Step 2.
        if self.ready_state.get() == ReadyState::HaveNothing {
            return;
        }

        // Step 3.
        // The fetch request associated with this seek already takes
        // care of cancelling any previous requests.

        // Step 4.
        // The flag will be cleared when the media engine tells us the seek was done.
        self.seeking.set(true);

        // Step 5.
        // XXX(ferjm) The rest of the steps should be run in parallel, so seeking cancelation
        //            can be done properly. No other browser does it yet anyway.

        // Step 6.
        let time = f64::min(time, self.Duration());

        // Step 7.
        let time = f64::max(time, 0.);

        // Step 8.
        // XXX(ferjm) seekable attribute: we need to get the information about
        //            what's been decoded and buffered so far from servo-media
        //            and add the seekable attribute as a TimeRange.
        if let Some(ref current_fetch_context) = *self.current_fetch_context.borrow() {
            if !current_fetch_context.is_seekable() {
                self.seeking.set(false);
                return;
            }
        }

        // Step 9.
        // servo-media with gstreamer does not support inaccurate seeking for now.

        // Step 10.
        self.owner_global()
            .task_manager()
            .media_element_task_source()
            .queue_simple_event(self.upcast(), atom!("seeking"));

        // Step 11.
        if let Some(ref player) = *self.player.borrow() {
            if let Err(e) = player.lock().unwrap().seek(time) {
                eprintln!("Seek error {:?}", e);
            }
        }

        // The rest of the steps are handled when the media engine signals a
        // ready state change or otherwise satisfies seek completion and signals
        // a position change.
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-seek
    fn seek_end(&self) {
        // Step 14.
        self.seeking.set(false);

        // Step 15.
        self.time_marches_on();

        // Step 16.
        let global = self.owner_global();
        let task_manager = global.task_manager();
        let task_source = task_manager.media_element_task_source();
        task_source.queue_simple_event(self.upcast(), atom!("timeupdate"));

        // Step 17.
        task_source.queue_simple_event(self.upcast(), atom!("seeked"));
    }

    /// <https://html.spec.whatwg.org/multipage/#poster-frame>
    pub(crate) fn process_poster_image_loaded(&self, image: Arc<Image>) {
        if !self.show_poster.get() {
            return;
        }

        // Step 6.
        self.handle_resize(Some(image.width), Some(image.height));
        self.video_renderer
            .lock()
            .unwrap()
            .render_poster_frame(image);
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);

        if pref!(media_testing_enabled) {
            self.owner_global()
                .task_manager()
                .media_element_task_source()
                .queue_simple_event(self.upcast(), atom!("postershown"));
        }
    }

    fn setup_media_player(&self, resource: &Resource) -> Result<(), ()> {
        let stream_type = match *resource {
            Resource::Object => {
                if let Some(ref src_object) = *self.src_object.borrow() {
                    match src_object {
                        SrcObject::MediaStream(_) => StreamType::Stream,
                        _ => StreamType::Seekable,
                    }
                } else {
                    return Err(());
                }
            },
            _ => StreamType::Seekable,
        };

        let window = self.owner_window();
        let (action_sender, action_receiver) = ipc::channel::<PlayerEvent>().unwrap();
        let video_renderer: Option<Arc<Mutex<dyn VideoFrameRenderer>>> = match self.media_type_id()
        {
            HTMLMediaElementTypeId::HTMLAudioElement => None,
            HTMLMediaElementTypeId::HTMLVideoElement => Some(self.video_renderer.clone()),
        };

        let audio_renderer = self.audio_renderer.borrow().as_ref().cloned();

        let pipeline_id = window.pipeline_id();
        let client_context_id =
            ClientContextId::build(pipeline_id.namespace_id.0, pipeline_id.index.0.get());
        let player = ServoMedia::get().create_player(
            &client_context_id,
            stream_type,
            action_sender,
            video_renderer,
            audio_renderer,
            Box::new(window.get_player_context()),
        );

        *self.player.borrow_mut() = Some(player);

        let trusted_node = Trusted::new(self);
        let task_source = self
            .owner_global()
            .task_manager()
            .media_element_task_source()
            .to_sendable();
        ROUTER.add_typed_route(
            action_receiver,
            Box::new(move |message| {
                let event = message.unwrap();
                trace!("Player event {:?}", event);
                let this = trusted_node.clone();
                task_source.queue(task!(handle_player_event: move || {
                    this.root().handle_player_event(&event, CanGc::note());
                }));
            }),
        );

        // GLPlayer thread setup
        let (player_id, image_receiver) = window
            .get_player_context()
            .glplayer_thread_sender
            .map(|pipeline| {
                let (image_sender, image_receiver) = channel().unwrap();
                pipeline
                    .send(GLPlayerMsg::RegisterPlayer(image_sender))
                    .unwrap();
                match image_receiver.recv().unwrap() {
                    GLPlayerMsgForward::PlayerId(id) => (id, Some(image_receiver)),
                    _ => unreachable!(),
                }
            })
            .unwrap_or((0, None));

        self.id.set(player_id);
        self.video_renderer.lock().unwrap().player_id = Some(player_id);

        if let Some(image_receiver) = image_receiver {
            let trusted_node = Trusted::new(self);
            let task_source = self
                .owner_global()
                .task_manager()
                .media_element_task_source()
                .to_sendable();
            ROUTER.add_typed_route(
                image_receiver,
                Box::new(move |message| {
                    let msg = message.unwrap();
                    let this = trusted_node.clone();
                    task_source.queue(task!(handle_glplayer_message: move || {
                        trace!("GLPlayer message {:?}", msg);
                        let video_renderer = this.root().video_renderer.clone();

                        match msg {
                            GLPlayerMsgForward::Lock(sender) => {
                                if let Some(holder) = video_renderer
                                    .lock()
                                    .unwrap()
                                    .current_frame_holder
                                    .as_mut() {
                                        holder.lock();
                                        sender.send(holder.get()).unwrap();
                                    };
                            },
                            GLPlayerMsgForward::Unlock() => {
                                if let Some(holder) = video_renderer
                                    .lock()
                                    .unwrap()
                                    .current_frame_holder
                                    .as_mut() { holder.unlock() }
                            },
                            _ => (),
                        }
                    }));
                }),
            );
        }

        Ok(())
    }

    pub(crate) fn set_audio_track(&self, idx: usize, enabled: bool) {
        if let Some(ref player) = *self.player.borrow() {
            if let Err(err) = player.lock().unwrap().set_audio_track(idx as i32, enabled) {
                warn!("Could not set audio track {:#?}", err);
            }
        }
    }

    pub(crate) fn set_video_track(&self, idx: usize, enabled: bool) {
        if let Some(ref player) = *self.player.borrow() {
            if let Err(err) = player.lock().unwrap().set_video_track(idx as i32, enabled) {
                warn!("Could not set video track {:#?}", err);
            }
        }
    }

    fn handle_player_event(&self, event: &PlayerEvent, can_gc: CanGc) {
        match *event {
            PlayerEvent::EndOfStream => {
                // https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
                // => "If the media data can be fetched but is found by inspection to be in
                //    an unsupported format, or can otherwise not be rendered at all"
                if self.ready_state.get() < ReadyState::HaveMetadata {
                    self.queue_dedicated_media_source_failure_steps();
                } else {
                    // https://html.spec.whatwg.org/multipage/#reaches-the-end
                    match self.direction_of_playback() {
                        PlaybackDirection::Forwards => {
                            // Step 1.
                            if self.Loop() {
                                self.seek(
                                    self.earliest_possible_position(),
                                    /* approximate_for_speed*/ false,
                                );
                            } else {
                                // Step 2.
                                // The **ended playback** condition is implemented inside of
                                // the HTMLMediaElementMethods::Ended method

                                // Step 3.
                                let this = Trusted::new(self);

                                self.owner_global().task_manager().media_element_task_source().queue(
                                    task!(reaches_the_end_steps: move || {
                                        let this = this.root();
                                        // Step 3.1.
                                        this.upcast::<EventTarget>().fire_event(atom!("timeupdate"), CanGc::note());

                                        // Step 3.2.
                                        if this.Ended() && !this.Paused() {
                                            // Step 3.2.1.
                                            this.paused.set(true);

                                            // Step 3.2.2.
                                            this.upcast::<EventTarget>().fire_event(atom!("pause"), CanGc::note());

                                            // Step 3.2.3.
                                            this.take_pending_play_promises(Err(Error::Abort));
                                            this.fulfill_in_flight_play_promises(|| ());
                                        }

                                        // Step 3.3.
                                        this.upcast::<EventTarget>().fire_event(atom!("ended"), CanGc::note());
                                    })
                                );

                                // https://html.spec.whatwg.org/multipage/#dom-media-have_current_data
                                self.change_ready_state(ReadyState::HaveCurrentData);
                            }
                        },

                        PlaybackDirection::Backwards => {
                            if self.playback_position.get() <= self.earliest_possible_position() {
                                self.owner_global()
                                    .task_manager()
                                    .media_element_task_source()
                                    .queue_simple_event(self.upcast(), atom!("ended"));
                            }
                        },
                    }
                }
            },
            PlayerEvent::Error(ref error) => {
                error!("Player error: {:?}", error);

                // If we have already flagged an error condition while processing
                // the network response, we should silently skip any observable
                // errors originating while decoding the erroneous response.
                if self.in_error_state() {
                    return;
                }

                // https://html.spec.whatwg.org/multipage/#loading-the-media-resource:media-data-13
                // 1. The user agent should cancel the fetching process.
                if let Some(ref mut current_fetch_context) =
                    *self.current_fetch_context.borrow_mut()
                {
                    current_fetch_context.cancel(CancelReason::Error);
                }
                // 2. Set the error attribute to the result of creating a MediaError with MEDIA_ERR_DECODE.
                self.error.set(Some(&*MediaError::new(
                    &self.owner_window(),
                    MEDIA_ERR_DECODE,
                    can_gc,
                )));

                // 3. Set the element's networkState attribute to the NETWORK_IDLE value.
                self.network_state.set(NetworkState::Idle);

                // 4. Set the element's delaying-the-load-event flag to false. This stops delaying the load event.
                self.delay_load_event(false, can_gc);

                // 5. Fire an event named error at the media element.
                self.upcast::<EventTarget>()
                    .fire_event(atom!("error"), can_gc);

                // TODO: 6. Abort the overall resource selection algorithm.
            },
            PlayerEvent::VideoFrameUpdated => {
                self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                // Check if the frame was resized
                if let Some(frame) = self.video_renderer.lock().unwrap().current_frame {
                    self.handle_resize(Some(frame.width as u32), Some(frame.height as u32));
                }
            },
            PlayerEvent::MetadataUpdated(ref metadata) => {
                // https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
                // => If the media resource is found to have an audio track
                if !metadata.audio_tracks.is_empty() {
                    for (i, _track) in metadata.audio_tracks.iter().enumerate() {
                        // Step 1.
                        let kind = match i {
                            0 => DOMString::from("main"),
                            _ => DOMString::new(),
                        };
                        let window = self.owner_window();
                        let audio_track = AudioTrack::new(
                            &window,
                            DOMString::new(),
                            kind,
                            DOMString::new(),
                            DOMString::new(),
                            Some(&*self.AudioTracks()),
                            can_gc,
                        );

                        // Steps 2. & 3.
                        self.AudioTracks().add(&audio_track);

                        // Step 4
                        if let Some(servo_url) = self.resource_url.borrow().as_ref() {
                            let fragment = MediaFragmentParser::from(servo_url);
                            if let Some(id) = fragment.id() {
                                if audio_track.id() == DOMString::from(id) {
                                    self.AudioTracks()
                                        .set_enabled(self.AudioTracks().len() - 1, true);
                                }
                            }

                            if fragment.tracks().contains(&audio_track.kind().into()) {
                                self.AudioTracks()
                                    .set_enabled(self.AudioTracks().len() - 1, true);
                            }
                        }

                        // Step 5. & 6,
                        if self.AudioTracks().enabled_index().is_none() {
                            self.AudioTracks()
                                .set_enabled(self.AudioTracks().len() - 1, true);
                        }

                        // Steps 7.
                        let event = TrackEvent::new(
                            self.global().as_window(),
                            atom!("addtrack"),
                            false,
                            false,
                            &Some(VideoTrackOrAudioTrackOrTextTrack::AudioTrack(audio_track)),
                            can_gc,
                        );

                        event
                            .upcast::<Event>()
                            .fire(self.upcast::<EventTarget>(), can_gc);
                    }
                }

                // => If the media resource is found to have a video track
                if !metadata.video_tracks.is_empty() {
                    for (i, _track) in metadata.video_tracks.iter().enumerate() {
                        // Step 1.
                        let kind = match i {
                            0 => DOMString::from("main"),
                            _ => DOMString::new(),
                        };
                        let window = self.owner_window();
                        let video_track = VideoTrack::new(
                            &window,
                            DOMString::new(),
                            kind,
                            DOMString::new(),
                            DOMString::new(),
                            Some(&*self.VideoTracks()),
                            can_gc,
                        );

                        // Steps 2. & 3.
                        self.VideoTracks().add(&video_track);

                        // Step 4.
                        if let Some(track) = self.VideoTracks().item(0) {
                            if let Some(servo_url) = self.resource_url.borrow().as_ref() {
                                let fragment = MediaFragmentParser::from(servo_url);
                                if let Some(id) = fragment.id() {
                                    if track.id() == DOMString::from(id) {
                                        self.VideoTracks().set_selected(0, true);
                                    }
                                } else if fragment.tracks().contains(&track.kind().into()) {
                                    self.VideoTracks().set_selected(0, true);
                                }
                            }
                        }

                        // Step 5. & 6.
                        if self.VideoTracks().selected_index().is_none() {
                            self.VideoTracks()
                                .set_selected(self.VideoTracks().len() - 1, true);
                        }

                        // Steps 7.
                        let event = TrackEvent::new(
                            self.global().as_window(),
                            atom!("addtrack"),
                            false,
                            false,
                            &Some(VideoTrackOrAudioTrackOrTextTrack::VideoTrack(video_track)),
                            can_gc,
                        );

                        event
                            .upcast::<Event>()
                            .fire(self.upcast::<EventTarget>(), can_gc);
                    }
                }

                // => "Once enough of the media data has been fetched to determine the duration..."
                // Step 1.
                // servo-media owns the media timeline.

                // Step 2.
                // XXX(ferjm) Update the timeline offset.

                // Step 3.
                self.playback_position.set(0.);

                // Step 4.
                let previous_duration = self.duration.get();
                if let Some(duration) = metadata.duration {
                    self.duration.set(duration.as_secs() as f64);
                } else {
                    self.duration.set(f64::INFINITY);
                }
                if previous_duration != self.duration.get() {
                    self.owner_global()
                        .task_manager()
                        .media_element_task_source()
                        .queue_simple_event(self.upcast(), atom!("durationchange"));
                }

                // Step 5.
                self.handle_resize(Some(metadata.width), Some(metadata.height));

                // Step 6.
                self.change_ready_state(ReadyState::HaveMetadata);

                // Step 7.
                let mut jumped = false;

                // Step 8.
                if self.default_playback_start_position.get() > 0. {
                    self.seek(
                        self.default_playback_start_position.get(),
                        /* approximate_for_speed*/ false,
                    );
                    jumped = true;
                }

                // Step 9.
                self.default_playback_start_position.set(0.);

                // Steps 10 and 11.
                if let Some(servo_url) = self.resource_url.borrow().as_ref() {
                    let fragment = MediaFragmentParser::from(servo_url);
                    if let Some(start) = fragment.start() {
                        if start > 0. && start < self.duration.get() {
                            self.playback_position.set(start);
                            if !jumped {
                                self.seek(self.playback_position.get(), false)
                            }
                        }
                    }
                }

                // Step 12 & 13 are already handled by the earlier media track processing.

                // We wait until we have metadata to render the controls, so we render them
                // with the appropriate size.
                if self.Controls() {
                    self.render_controls(can_gc);
                }

                let global = self.global();
                let window = global.as_window();

                // Update the media session metadata title with the obtained metadata.
                window.Navigator().MediaSession().update_title(
                    metadata
                        .title
                        .clone()
                        .unwrap_or(window.get_url().into_string()),
                );
            },
            PlayerEvent::NeedData => {
                // The player needs more data.
                // If we already have a valid fetch request, we do nothing.
                // Otherwise, if we have no request and the previous request was
                // cancelled because we got an EnoughData event, we restart
                // fetching where we left.
                if let Some(ref current_fetch_context) = *self.current_fetch_context.borrow() {
                    match current_fetch_context.cancel_reason() {
                        Some(reason) if *reason == CancelReason::Backoff => {
                            // XXX(ferjm) Ideally we should just create a fetch request from
                            // where we left. But keeping track of the exact next byte that the
                            // media backend expects is not the easiest task, so I'm simply
                            // seeking to the current playback position for now which will create
                            // a new fetch request for the last rendered frame.
                            self.seek(self.playback_position.get(), false)
                        },
                        _ => (),
                    }
                }
            },
            PlayerEvent::EnoughData => {
                self.change_ready_state(ReadyState::HaveEnoughData);

                // The player has enough data and it is asking us to stop pushing
                // bytes, so we cancel the ongoing fetch request iff we are able
                // to restart it from where we left. Otherwise, we continue the
                // current fetch request, assuming that some frames will be dropped.
                if let Some(ref mut current_fetch_context) =
                    *self.current_fetch_context.borrow_mut()
                {
                    if current_fetch_context.is_seekable() {
                        current_fetch_context.cancel(CancelReason::Backoff);
                    }
                }
            },
            PlayerEvent::PositionChanged(position) => {
                let position = position as f64;
                let _ = self
                    .played
                    .borrow_mut()
                    .add(self.playback_position.get(), position);
                self.playback_position.set(position);
                self.time_marches_on();
                let media_position_state =
                    MediaPositionState::new(self.duration.get(), self.playbackRate.get(), position);
                debug!(
                    "Sending media session event set position state {:?}",
                    media_position_state
                );
                self.send_media_session_event(MediaSessionEvent::SetPositionState(
                    media_position_state,
                ));
            },
            PlayerEvent::SeekData(p, ref seek_lock) => {
                self.fetch_request(Some(p), Some(seek_lock.clone()));
            },
            PlayerEvent::SeekDone(_) => {
                // Continuation of
                // https://html.spec.whatwg.org/multipage/#dom-media-seek

                // Step 13.
                let task = MediaElementMicrotask::Seeked {
                    elem: DomRoot::from_ref(self),
                    generation_id: self.generation_id.get(),
                };
                ScriptThread::await_stable_state(Microtask::MediaElement(task));
            },
            PlayerEvent::StateChanged(ref state) => {
                let mut media_session_playback_state = MediaSessionPlaybackState::None_;
                match *state {
                    PlaybackState::Paused => {
                        media_session_playback_state = MediaSessionPlaybackState::Paused;
                        if self.ready_state.get() == ReadyState::HaveMetadata {
                            self.change_ready_state(ReadyState::HaveEnoughData);
                        }
                    },
                    PlaybackState::Playing => {
                        media_session_playback_state = MediaSessionPlaybackState::Playing;
                    },
                    PlaybackState::Buffering => {
                        // Do not send the media session playback state change event
                        // in this case as a None_ state is expected to clean up the
                        // session.
                        return;
                    },
                    _ => {},
                };
                debug!(
                    "Sending media session event playback state changed to {:?}",
                    media_session_playback_state
                );
                self.send_media_session_event(MediaSessionEvent::PlaybackStateChange(
                    media_session_playback_state,
                ));
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#earliest-possible-position
    fn earliest_possible_position(&self) -> f64 {
        self.played
            .borrow()
            .start(0)
            .unwrap_or_else(|_| self.playback_position.get())
    }

    fn render_controls(&self, can_gc: CanGc) {
        let element = self.htmlelement.upcast::<Element>();
        if self.ready_state.get() < ReadyState::HaveMetadata || element.is_shadow_host() {
            // Bail out if we have no metadata yet or
            // if we are already showing the controls.
            return;
        }
        let shadow_root = element
            .attach_shadow(
                IsUserAgentWidget::Yes,
                ShadowRootMode::Closed,
                false,
                false,
                false,
                SlotAssignmentMode::Manual,
                can_gc,
            )
            .unwrap();
        let document = self.owner_document();
        let script = HTMLScriptElement::new(
            local_name!("script"),
            None,
            &document,
            None,
            ElementCreator::ScriptCreated,
            can_gc,
        );
        let mut media_controls_script = resources::read_string(EmbedderResource::MediaControlsJS);
        // This is our hacky way to temporarily workaround the lack of a privileged
        // JS context.
        // The media controls UI accesses the document.servoGetMediaControls(id) API
        // to get an instance to the media controls ShadowRoot.
        // `id` needs to match the internally generated UUID assigned to a media element.
        let id = document.register_media_controls(&shadow_root);
        let media_controls_script = media_controls_script.as_mut_str().replace("@@@id@@@", &id);
        *self.media_controls_id.borrow_mut() = Some(id);
        script
            .upcast::<Node>()
            .SetTextContent(Some(DOMString::from(media_controls_script)), can_gc);
        if let Err(e) = shadow_root
            .upcast::<Node>()
            .AppendChild(script.upcast::<Node>(), can_gc)
        {
            warn!("Could not render media controls {:?}", e);
            return;
        }

        let media_controls_style = resources::read_string(EmbedderResource::MediaControlsCSS);
        let style = HTMLStyleElement::new(
            local_name!("script"),
            None,
            &document,
            None,
            ElementCreator::ScriptCreated,
            can_gc,
        );
        style
            .upcast::<Node>()
            .SetTextContent(Some(DOMString::from(media_controls_style)), can_gc);

        if let Err(e) = shadow_root
            .upcast::<Node>()
            .AppendChild(style.upcast::<Node>(), can_gc)
        {
            warn!("Could not render media controls {:?}", e);
        }

        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    fn remove_controls(&self, can_gc: CanGc) {
        if let Some(id) = self.media_controls_id.borrow_mut().take() {
            self.owner_document().unregister_media_controls(&id, can_gc);
        }
    }

    pub(crate) fn get_current_frame(&self) -> Option<VideoFrame> {
        self.video_renderer
            .lock()
            .unwrap()
            .current_frame_holder
            .as_ref()
            .map(|holder| holder.get_frame())
    }

    pub(crate) fn get_current_frame_data(&self) -> Option<MediaFrame> {
        self.video_renderer.lock().unwrap().current_frame
    }

    pub(crate) fn clear_current_frame_data(&self) {
        self.handle_resize(None, None);
        self.video_renderer.lock().unwrap().current_frame = None;
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    fn handle_resize(&self, width: Option<u32>, height: Option<u32>) {
        if let Some(video_elem) = self.downcast::<HTMLVideoElement>() {
            video_elem.resize(width, height);
        }
    }

    /// By default the audio is rendered through the audio sink automatically
    /// selected by the servo-media Player instance. However, in some cases, like
    /// the WebAudio MediaElementAudioSourceNode, we need to set a custom audio
    /// renderer.
    pub(crate) fn set_audio_renderer(
        &self,
        audio_renderer: Arc<Mutex<dyn AudioRenderer>>,
        can_gc: CanGc,
    ) {
        *self.audio_renderer.borrow_mut() = Some(audio_renderer);
        if let Some(ref player) = *self.player.borrow() {
            if let Err(e) = player.lock().unwrap().stop() {
                eprintln!("Could not stop player {:?}", e);
            }
            self.media_element_load_algorithm(can_gc);
        }
    }

    fn send_media_session_event(&self, event: MediaSessionEvent) {
        let global = self.global();
        let media_session = global.as_window().Navigator().MediaSession();

        media_session.register_media_instance(self);

        media_session.send_event(event);
    }

    pub(crate) fn set_duration(&self, duration: f64) {
        self.duration.set(duration);
    }

    /// Sets a new value for the show_poster propperty. Updates video_rederer
    /// with the new value.
    pub(crate) fn set_show_poster(&self, show_poster: bool) {
        self.show_poster.set(show_poster);
        self.video_renderer.lock().unwrap().show_poster = show_poster;
    }

    pub(crate) fn reset(&self) {
        if let Some(ref player) = *self.player.borrow() {
            if let Err(e) = player.lock().unwrap().stop() {
                eprintln!("Could not stop player {:?}", e);
            }
        }
    }
}

// XXX Placeholder for [https://github.com/servo/servo/issues/22293]
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
enum PlaybackDirection {
    Forwards,
    #[allow(dead_code)]
    Backwards,
}

// XXX Placeholder implementations for:
//
// - https://github.com/servo/servo/issues/22293
impl HTMLMediaElement {
    // https://github.com/servo/servo/issues/22293
    fn direction_of_playback(&self) -> PlaybackDirection {
        PlaybackDirection::Forwards
    }
}

impl Drop for HTMLMediaElement {
    fn drop(&mut self) {
        self.player_context
            .send(GLPlayerMsg::UnregisterPlayer(self.id.get()));
    }
}

impl HTMLMediaElementMethods<crate::DomTypeHolder> for HTMLMediaElement {
    // https://html.spec.whatwg.org/multipage/#dom-media-networkstate
    fn NetworkState(&self) -> u16 {
        self.network_state.get() as u16
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-readystate
    fn ReadyState(&self) -> u16 {
        self.ready_state.get() as u16
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-autoplay
    make_bool_getter!(Autoplay, "autoplay");
    // https://html.spec.whatwg.org/multipage/#dom-media-autoplay
    make_bool_setter!(SetAutoplay, "autoplay");

    // https://html.spec.whatwg.org/multipage/#attr-media-loop
    make_bool_getter!(Loop, "loop");
    // https://html.spec.whatwg.org/multipage/#attr-media-loop
    make_bool_setter!(SetLoop, "loop");

    // https://html.spec.whatwg.org/multipage/#dom-media-defaultmuted
    make_bool_getter!(DefaultMuted, "muted");
    // https://html.spec.whatwg.org/multipage/#dom-media-defaultmuted
    make_bool_setter!(SetDefaultMuted, "muted");

    // https://html.spec.whatwg.org/multipage/#dom-media-controls
    make_bool_getter!(Controls, "controls");
    // https://html.spec.whatwg.org/multipage/#dom-media-controls
    make_bool_setter!(SetControls, "controls");

    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_url_getter!(Src, "src");

    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_url_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-media-crossOrigin
    fn GetCrossOrigin(&self) -> Option<DOMString> {
        reflect_cross_origin_attribute(self.upcast::<Element>())
    }
    // https://html.spec.whatwg.org/multipage/#dom-media-crossOrigin
    fn SetCrossOrigin(&self, value: Option<DOMString>, can_gc: CanGc) {
        set_cross_origin_attribute(self.upcast::<Element>(), value, can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-muted
    fn Muted(&self) -> bool {
        self.muted.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-muted
    fn SetMuted(&self, value: bool) {
        if self.muted.get() == value {
            return;
        }

        if let Some(ref player) = *self.player.borrow() {
            let _ = player.lock().unwrap().set_mute(value);
        }

        self.muted.set(value);
        self.owner_global()
            .task_manager()
            .media_element_task_source()
            .queue_simple_event(self.upcast(), atom!("volumechange"));
        if !self.is_allowed_to_play() {
            self.internal_pause_steps();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-srcobject
    fn GetSrcObject(&self) -> Option<MediaStreamOrBlob> {
        (*self.src_object.borrow())
            .as_ref()
            .map(|src_object| match src_object {
                SrcObject::Blob(blob) => MediaStreamOrBlob::Blob(DomRoot::from_ref(blob)),
                SrcObject::MediaStream(stream) => {
                    MediaStreamOrBlob::MediaStream(DomRoot::from_ref(stream))
                },
            })
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-srcobject
    fn SetSrcObject(&self, value: Option<MediaStreamOrBlob>, can_gc: CanGc) {
        *self.src_object.borrow_mut() = value.map(|value| value.into());
        self.media_element_load_algorithm(can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#attr-media-preload
    // Missing/Invalid values are user-agent defined.
    make_enumerated_getter!(
        Preload,
        "preload",
        "none" | "metadata" | "auto",
        missing => "auto",
        invalid => "auto"
    );

    // https://html.spec.whatwg.org/multipage/#attr-media-preload
    make_setter!(SetPreload, "preload");

    // https://html.spec.whatwg.org/multipage/#dom-media-currentsrc
    fn CurrentSrc(&self) -> USVString {
        USVString(self.current_src.borrow().clone())
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-load
    fn Load(&self, can_gc: CanGc) {
        self.media_element_load_algorithm(can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-canplaytype
    fn CanPlayType(&self, type_: DOMString) -> CanPlayTypeResult {
        match ServoMedia::get().can_play_type(&type_) {
            SupportsMediaType::No => CanPlayTypeResult::_empty,
            SupportsMediaType::Maybe => CanPlayTypeResult::Maybe,
            SupportsMediaType::Probably => CanPlayTypeResult::Probably,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-error
    fn GetError(&self) -> Option<DomRoot<MediaError>> {
        self.error.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-play
    fn Play(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp, can_gc);
        // Step 1.
        // FIXME(nox): Reject promise if not allowed to play.

        // Step 2.
        if self
            .error
            .get()
            .is_some_and(|e| e.Code() == MEDIA_ERR_SRC_NOT_SUPPORTED)
        {
            promise.reject_error(Error::NotSupported, can_gc);
            return promise;
        }

        // Step 3.
        self.push_pending_play_promise(&promise);

        // Step 4.
        if self.network_state.get() == NetworkState::Empty {
            self.invoke_resource_selection_algorithm(can_gc);
        }

        // Step 5.
        if self.Ended() && self.direction_of_playback() == PlaybackDirection::Forwards {
            self.seek(
                self.earliest_possible_position(),
                /* approximate_for_speed */ false,
            );
        }

        let state = self.ready_state.get();

        let global = self.owner_global();
        let task_manager = global.task_manager();
        let task_source = task_manager.media_element_task_source();
        if self.Paused() {
            // Step 6.1.
            self.paused.set(false);

            // Step 6.2.
            if self.show_poster.get() {
                self.set_show_poster(false);
                self.time_marches_on();
            }

            // Step 6.3.
            task_source.queue_simple_event(self.upcast(), atom!("play"));

            // Step 6.4.
            match state {
                ReadyState::HaveNothing |
                ReadyState::HaveMetadata |
                ReadyState::HaveCurrentData => {
                    task_source.queue_simple_event(self.upcast(), atom!("waiting"));
                },
                ReadyState::HaveFutureData | ReadyState::HaveEnoughData => {
                    self.notify_about_playing();
                },
            }
        } else if state == ReadyState::HaveFutureData || state == ReadyState::HaveEnoughData {
            // Step 7.
            self.take_pending_play_promises(Ok(()));
            let this = Trusted::new(self);
            let generation_id = self.generation_id.get();
            task_source.queue(task!(resolve_pending_play_promises: move || {
                let this = this.root();
                if generation_id != this.generation_id.get() {
                    return;
                }

                this.fulfill_in_flight_play_promises(|| {
                    this.play_media();
                });
            }));
        }

        // Step 8.
        self.autoplaying.set(false);

        // Step 9.
        promise
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-pause
    fn Pause(&self, can_gc: CanGc) {
        // Step 1
        if self.network_state.get() == NetworkState::Empty {
            self.invoke_resource_selection_algorithm(can_gc);
        }

        // Step 2
        self.internal_pause_steps();
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-paused
    fn Paused(&self) -> bool {
        self.paused.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-defaultplaybackrate>
    fn GetDefaultPlaybackRate(&self) -> Fallible<Finite<f64>> {
        Ok(Finite::wrap(self.defaultPlaybackRate.get()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-defaultplaybackrate>
    fn SetDefaultPlaybackRate(&self, value: Finite<f64>) -> ErrorResult {
        let min_allowed = -64.0;
        let max_allowed = 64.0;
        if *value < min_allowed || *value > max_allowed {
            return Err(Error::NotSupported);
        }

        if *value != self.defaultPlaybackRate.get() {
            self.defaultPlaybackRate.set(*value);
            self.queue_ratechange_event();
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-playbackrate>
    fn GetPlaybackRate(&self) -> Fallible<Finite<f64>> {
        Ok(Finite::wrap(self.playbackRate.get()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-playbackrate>
    fn SetPlaybackRate(&self, value: Finite<f64>) -> ErrorResult {
        let min_allowed = -64.0;
        let max_allowed = 64.0;
        if *value < min_allowed || *value > max_allowed {
            return Err(Error::NotSupported);
        }

        if *value != self.playbackRate.get() {
            self.playbackRate.set(*value);
            self.queue_ratechange_event();
            if self.is_potentially_playing() {
                if let Some(ref player) = *self.player.borrow() {
                    if let Err(e) = player.lock().unwrap().set_rate(*value) {
                        warn!("Could not set the playback rate {:?}", e);
                    }
                }
            }
        }

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-duration
    fn Duration(&self) -> f64 {
        self.duration.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-currenttime
    fn CurrentTime(&self) -> Finite<f64> {
        Finite::wrap(if self.default_playback_start_position.get() != 0. {
            self.default_playback_start_position.get()
        } else {
            self.playback_position.get()
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-currenttime
    fn SetCurrentTime(&self, time: Finite<f64>) {
        if self.ready_state.get() == ReadyState::HaveNothing {
            self.default_playback_start_position.set(*time);
        } else {
            self.playback_position.set(*time);
            self.seek(*time, /* approximate_for_speed */ false);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-seeking
    fn Seeking(&self) -> bool {
        self.seeking.get()
    }

    // https://html.spec.whatwg.org/multipage/#ended-playback
    fn Ended(&self) -> bool {
        if self.ready_state.get() < ReadyState::HaveMetadata {
            return false;
        }

        let playback_pos = self.playback_position.get();

        match self.direction_of_playback() {
            PlaybackDirection::Forwards => playback_pos >= self.Duration() && !self.Loop(),
            PlaybackDirection::Backwards => playback_pos <= self.earliest_possible_position(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-fastseek
    fn FastSeek(&self, time: Finite<f64>) {
        self.seek(*time, /* approximate_for_speed */ true);
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-played
    fn Played(&self) -> DomRoot<TimeRanges> {
        TimeRanges::new(
            self.global().as_window(),
            self.played.borrow().clone(),
            CanGc::note(),
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-buffered
    fn Buffered(&self) -> DomRoot<TimeRanges> {
        let mut buffered = TimeRangesContainer::default();
        if let Some(ref player) = *self.player.borrow() {
            if let Ok(ranges) = player.lock().unwrap().buffered() {
                for range in ranges {
                    let _ = buffered.add(range.start, range.end);
                }
            }
        }
        TimeRanges::new(self.global().as_window(), buffered, CanGc::note())
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-audiotracks
    fn AudioTracks(&self) -> DomRoot<AudioTrackList> {
        let window = self.owner_window();
        self.audio_tracks_list
            .or_init(|| AudioTrackList::new(&window, &[], Some(self), CanGc::note()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-videotracks
    fn VideoTracks(&self) -> DomRoot<VideoTrackList> {
        let window = self.owner_window();
        self.video_tracks_list
            .or_init(|| VideoTrackList::new(&window, &[], Some(self), CanGc::note()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-texttracks
    fn TextTracks(&self) -> DomRoot<TextTrackList> {
        let window = self.owner_window();
        self.text_tracks_list
            .or_init(|| TextTrackList::new(&window, &[], CanGc::note()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-addtexttrack
    fn AddTextTrack(
        &self,
        kind: TextTrackKind,
        label: DOMString,
        language: DOMString,
    ) -> DomRoot<TextTrack> {
        let window = self.owner_window();
        // Step 1 & 2
        // FIXME(#22314, dlrobertson) set the ready state to Loaded
        let track = TextTrack::new(
            &window,
            "".into(),
            kind,
            label,
            language,
            TextTrackMode::Hidden,
            None,
            CanGc::note(),
        );
        // Step 3 & 4
        self.TextTracks().add(&track);
        // Step 5
        DomRoot::from_ref(&track)
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-volume
    fn GetVolume(&self) -> Fallible<Finite<f64>> {
        Ok(Finite::wrap(self.volume.get()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-volume
    fn SetVolume(&self, value: Finite<f64>) -> ErrorResult {
        let minimum_volume = 0.0;
        let maximum_volume = 1.0;
        if *value < minimum_volume || *value > maximum_volume {
            return Err(Error::IndexSize);
        }

        if *value != self.volume.get() {
            self.volume.set(*value);

            self.owner_global()
                .task_manager()
                .media_element_task_source()
                .queue_simple_event(self.upcast(), atom!("volumechange"));
            if !self.is_allowed_to_play() {
                self.internal_pause_steps();
            }
        }

        Ok(())
    }
}

impl VirtualMethods for HTMLMediaElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);

        match *attr.local_name() {
            local_name!("muted") => {
                self.SetMuted(mutation.new_value(attr).is_some());
            },
            local_name!("src") => {
                if mutation.new_value(attr).is_none() {
                    self.clear_current_frame_data();
                    return;
                }
                self.media_element_load_algorithm(CanGc::note());
            },
            local_name!("controls") => {
                if mutation.new_value(attr).is_some() {
                    self.render_controls(can_gc);
                } else {
                    self.remove_controls(can_gc);
                }
            },
            _ => (),
        };
    }

    // https://html.spec.whatwg.org/multipage/#playing-the-media-resource:remove-an-element-from-a-document
    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        self.remove_controls(can_gc);

        if context.tree_connected {
            let task = MediaElementMicrotask::PauseIfNotInDocument {
                elem: DomRoot::from_ref(self),
            };
            ScriptThread::await_stable_state(Microtask::MediaElement(task));
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum MediaElementMicrotask {
    ResourceSelection {
        elem: DomRoot<HTMLMediaElement>,
        generation_id: u32,
        #[no_trace]
        base_url: ServoUrl,
    },
    PauseIfNotInDocument {
        elem: DomRoot<HTMLMediaElement>,
    },
    Seeked {
        elem: DomRoot<HTMLMediaElement>,
        generation_id: u32,
    },
}

impl MicrotaskRunnable for MediaElementMicrotask {
    fn handler(&self, can_gc: CanGc) {
        match self {
            &MediaElementMicrotask::ResourceSelection {
                ref elem,
                generation_id,
                ref base_url,
            } => {
                if generation_id == elem.generation_id.get() {
                    elem.resource_selection_algorithm_sync(base_url.clone(), can_gc);
                }
            },
            MediaElementMicrotask::PauseIfNotInDocument { elem } => {
                if !elem.upcast::<Node>().is_connected() {
                    elem.internal_pause_steps();
                }
            },
            &MediaElementMicrotask::Seeked {
                ref elem,
                generation_id,
            } => {
                if generation_id == elem.generation_id.get() {
                    elem.seek_end();
                }
            },
        }
    }

    fn enter_realm(&self) -> JSAutoRealm {
        match self {
            &MediaElementMicrotask::ResourceSelection { ref elem, .. } |
            &MediaElementMicrotask::PauseIfNotInDocument { ref elem } |
            &MediaElementMicrotask::Seeked { ref elem, .. } => enter_realm(&**elem),
        }
    }
}

enum Resource {
    Object,
    Url(ServoUrl),
}

/// Indicates the reason why a fetch request was cancelled.
#[derive(Debug, MallocSizeOf, PartialEq)]
enum CancelReason {
    /// We were asked to stop pushing data to the player.
    Backoff,
    /// An error ocurred while fetching the media data.
    Error,
    /// A new request overrode this one.
    Overridden,
}

#[derive(MallocSizeOf)]
pub(crate) struct HTMLMediaElementFetchContext {
    /// Some if the request has been cancelled.
    cancel_reason: Option<CancelReason>,
    /// Indicates whether the fetched stream is seekable.
    is_seekable: bool,
    /// Fetch canceller. Allows cancelling the current fetch request by
    /// manually calling its .cancel() method or automatically on Drop.
    fetch_canceller: FetchCanceller,
}

impl HTMLMediaElementFetchContext {
    fn new(request_id: RequestId) -> HTMLMediaElementFetchContext {
        HTMLMediaElementFetchContext {
            cancel_reason: None,
            is_seekable: false,
            fetch_canceller: FetchCanceller::new(request_id),
        }
    }

    fn is_seekable(&self) -> bool {
        self.is_seekable
    }

    fn set_seekable(&mut self, seekable: bool) {
        self.is_seekable = seekable;
    }

    fn cancel(&mut self, reason: CancelReason) {
        if self.cancel_reason.is_some() {
            return;
        }
        self.cancel_reason = Some(reason);
        self.fetch_canceller.cancel();
    }

    fn cancel_reason(&self) -> &Option<CancelReason> {
        &self.cancel_reason
    }
}

struct HTMLMediaElementFetchListener {
    /// The element that initiated the request.
    elem: Trusted<HTMLMediaElement>,
    /// The response metadata received to date.
    metadata: Option<Metadata>,
    /// The generation of the media element when this fetch started.
    generation_id: u32,
    /// Time of last progress notification.
    next_progress_event: Instant,
    /// Timing data for this resource.
    resource_timing: ResourceFetchTiming,
    /// Url for the resource.
    url: ServoUrl,
    /// Expected content length of the media asset being fetched or played.
    expected_content_length: Option<u64>,
    /// Number of the last byte fetched from the network for the ongoing
    /// request. It is only reset to 0 if we reach EOS. Seek requests
    /// set it to the requested position. Requests triggered after an
    /// EnoughData event uses this value to restart the download from
    /// the last fetched position.
    latest_fetched_content: u64,
    /// The media player discards all data pushes until the seek block
    /// is released right before pushing the data from the offset requested
    /// by a seek request.
    seek_lock: Option<SeekLock>,
}

// https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
impl FetchResponseListener for HTMLMediaElementFetchListener {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(&mut self, _: RequestId, metadata: Result<FetchMetadata, NetworkError>) {
        let elem = self.elem.root();

        if elem.generation_id.get() != self.generation_id || elem.player.borrow().is_none() {
            // A new fetch request was triggered, so we ignore this response.
            return;
        }

        self.metadata = metadata.ok().map(|m| match m {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });

        if let Some(metadata) = self.metadata.as_ref() {
            if let Some(headers) = metadata.headers.as_ref() {
                // For range requests we get the size of the media asset from the Content-Range
                // header. Otherwise, we get it from the Content-Length header.
                let content_length =
                    if let Some(content_range) = headers.typed_get::<ContentRange>() {
                        content_range.bytes_len()
                    } else {
                        headers
                            .typed_get::<ContentLength>()
                            .map(|content_length| content_length.0)
                    };

                // We only set the expected input size if it changes.
                if content_length != self.expected_content_length {
                    if let Some(content_length) = content_length {
                        if let Err(e) = elem
                            .player
                            .borrow()
                            .as_ref()
                            .unwrap()
                            .lock()
                            .unwrap()
                            .set_input_size(content_length)
                        {
                            warn!("Could not set player input size {:?}", e);
                        } else {
                            self.expected_content_length = Some(content_length);
                        }
                    }
                }
            }
        }

        let (status_is_ok, is_seekable) = self.metadata.as_ref().map_or((true, false), |s| {
            let status = &s.status;
            (
                status.is_success(),
                *status == StatusCode::PARTIAL_CONTENT ||
                    *status == StatusCode::RANGE_NOT_SATISFIABLE,
            )
        });

        if is_seekable {
            // The server supports range requests,
            if let Some(ref mut current_fetch_context) = *elem.current_fetch_context.borrow_mut() {
                current_fetch_context.set_seekable(true);
            }
        }

        // => "If the media data cannot be fetched at all..."
        if !status_is_ok {
            // Ensure that the element doesn't receive any further notifications
            // of the aborted fetch.
            if let Some(ref mut current_fetch_context) = *elem.current_fetch_context.borrow_mut() {
                current_fetch_context.cancel(CancelReason::Error);
            }
            elem.queue_dedicated_media_source_failure_steps();
        }
    }

    fn process_response_chunk(&mut self, _: RequestId, payload: Vec<u8>) {
        let elem = self.elem.root();
        // If an error was received previously or if we triggered a new fetch request,
        // we skip processing the payload.
        if elem.generation_id.get() != self.generation_id || elem.player.borrow().is_none() {
            return;
        }
        if let Some(ref current_fetch_context) = *elem.current_fetch_context.borrow() {
            if current_fetch_context.cancel_reason().is_some() {
                return;
            }
        }

        let payload_len = payload.len() as u64;

        if let Some(seek_lock) = self.seek_lock.take() {
            seek_lock.unlock(/* successful seek */ true);
        }

        // Push input data into the player.
        if let Err(e) = elem
            .player
            .borrow()
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .push_data(payload)
        {
            // If we are pushing too much data and we know that we can
            // restart the download later from where we left, we cancel
            // the current request. Otherwise, we continue the request
            // assuming that we may drop some frames.
            if e == PlayerError::EnoughData {
                if let Some(ref mut current_fetch_context) =
                    *elem.current_fetch_context.borrow_mut()
                {
                    current_fetch_context.cancel(CancelReason::Backoff);
                }
            }
            warn!("Could not push input data to player {:?}", e);
            return;
        }

        self.latest_fetched_content += payload_len;

        // https://html.spec.whatwg.org/multipage/#concept-media-load-resource step 4,
        // => "If mode is remote" step 2
        if Instant::now() > self.next_progress_event {
            elem.owner_global()
                .task_manager()
                .media_element_task_source()
                .queue_simple_event(elem.upcast(), atom!("progress"));
            self.next_progress_event = Instant::now() + Duration::from_millis(350);
        }
    }

    // https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
    fn process_response_eof(
        &mut self,
        _: RequestId,
        status: Result<ResourceFetchTiming, NetworkError>,
    ) {
        trace!("process response eof");
        if let Some(seek_lock) = self.seek_lock.take() {
            seek_lock.unlock(/* successful seek */ false);
        }

        let elem = self.elem.root();

        if elem.generation_id.get() != self.generation_id || elem.player.borrow().is_none() {
            return;
        }

        // There are no more chunks of the response body forthcoming, so we can
        // go ahead and notify the media backend not to expect any further data.
        if let Err(e) = elem
            .player
            .borrow()
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .end_of_stream()
        {
            warn!("Could not signal EOS to player {:?}", e);
        }

        // If an error was previously received we skip processing the payload.
        if let Some(ref current_fetch_context) = *elem.current_fetch_context.borrow() {
            if let Some(CancelReason::Error) = current_fetch_context.cancel_reason() {
                return;
            }
        }

        if status.is_ok() && self.latest_fetched_content != 0 {
            elem.upcast::<EventTarget>()
                .fire_event(atom!("progress"), CanGc::note());

            elem.network_state.set(NetworkState::Idle);

            elem.upcast::<EventTarget>()
                .fire_event(atom!("suspend"), CanGc::note());
        }
        // => "If the connection is interrupted after some media data has been received..."
        else if elem.ready_state.get() != ReadyState::HaveNothing {
            // If the media backend has already flagged an error, skip any observable
            // network-related errors.
            if elem.in_error_state() {
                return;
            }

            // Step 1
            if let Some(ref mut current_fetch_context) = *elem.current_fetch_context.borrow_mut() {
                current_fetch_context.cancel(CancelReason::Error);
            }

            // Step 2
            elem.error.set(Some(&*MediaError::new(
                &elem.owner_window(),
                MEDIA_ERR_NETWORK,
                CanGc::note(),
            )));

            // Step 3
            elem.network_state.set(NetworkState::Idle);

            // Step 4.
            elem.delay_load_event(false, CanGc::note());

            // Step 5
            elem.upcast::<EventTarget>()
                .fire_event(atom!("error"), CanGc::note());
        } else {
            // => "If the media data cannot be fetched at all..."
            elem.queue_dedicated_media_source_failure_steps();
        }
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self, CanGc::note())
    }
}

impl ResourceTimingListener for HTMLMediaElementFetchListener {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        let initiator_type = InitiatorType::LocalName(
            self.elem
                .root()
                .upcast::<Element>()
                .local_name()
                .to_string(),
        );
        (initiator_type, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.elem.root().owner_document().global()
    }
}

impl PreInvoke for HTMLMediaElementFetchListener {
    fn should_invoke(&self) -> bool {
        //TODO: finish_load needs to run at some point if the generation changes.
        self.elem.root().generation_id.get() == self.generation_id
    }
}

impl HTMLMediaElementFetchListener {
    fn new(
        elem: &HTMLMediaElement,
        url: ServoUrl,
        offset: u64,
        seek_lock: Option<SeekLock>,
    ) -> Self {
        Self {
            elem: Trusted::new(elem),
            metadata: None,
            generation_id: elem.generation_id.get(),
            next_progress_event: Instant::now() + Duration::from_millis(350),
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
            url,
            expected_content_length: None,
            latest_fetched_content: offset,
            seek_lock,
        }
    }
}
