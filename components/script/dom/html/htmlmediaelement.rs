/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};
use std::{f64, mem};

use base::generic_channel::GenericSharedMemory;
use base::id::WebViewId;
use compositing_traits::{CrossProcessPaintApi, ImageUpdate, SerializableImageData};
use content_security_policy::sandboxing_directive::SandboxingFlagSet;
use dom_struct::dom_struct;
use embedder_traits::{MediaPositionState, MediaSessionEvent, MediaSessionPlaybackState};
use euclid::default::Size2D;
use headers::{ContentLength, ContentRange, HeaderMapExt};
use html5ever::{LocalName, Prefix, QualName, local_name, ns};
use http::StatusCode;
use http::header::{self, HeaderMap, HeaderValue};
use ipc_channel::ipc::{self};
use ipc_channel::router::ROUTER;
use js::jsapi::JSAutoRealm;
use layout_api::MediaFrame;
use media::{GLPlayerMsg, GLPlayerMsgForward, WindowGLContext};
use net_traits::request::{Destination, RequestId};
use net_traits::{
    CoreResourceThread, FetchMetadata, FilteredMetadata, NetworkError, ResourceFetchTiming,
};
use pixels::RasterImage;
use script_bindings::codegen::InheritTypes::{
    ElementTypeId, HTMLElementTypeId, HTMLMediaElementTypeId, NodeTypeId,
};
use script_bindings::root::assert_in_script;
use script_bindings::weakref::WeakRef;
use servo_config::pref;
use servo_media::player::audio::AudioRenderer;
use servo_media::player::video::{VideoFrame, VideoFrameRenderer};
use servo_media::player::{PlaybackState, Player, PlayerError, PlayerEvent, SeekLock, StreamType};
use servo_media::{ClientContextId, ServoMedia, SupportsMediaType};
use servo_url::ServoUrl;
use stylo_atoms::Atom;
use uuid::Uuid;
use webrender_api::{
    ExternalImageData, ExternalImageId, ExternalImageType, ImageBufferKind, ImageDescriptor,
    ImageDescriptorFlags, ImageFormat, ImageKey,
};

use crate::document_loader::{LoadBlocker, LoadType};
use crate::dom::attr::Attr;
use crate::dom::audio::audiotrack::AudioTrack;
use crate::dom::audio::audiotracklist::AudioTrackList;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLMediaElementBinding::{
    CanPlayTypeResult, HTMLMediaElementConstants, HTMLMediaElementMethods,
};
use crate::dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorConstants::*;
use crate::dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorMethods;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::Navigator_Binding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
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
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::document::Document;
use crate::dom::element::{
    AttributeMutation, AttributeMutationReason, CustomElementCreationMode, Element, ElementCreator,
    cors_setting_for_element, reflect_cross_origin_attribute, set_cross_origin_attribute,
};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlsourceelement::HTMLSourceElement;
use crate::dom::html::htmlvideoelement::HTMLVideoElement;
use crate::dom::mediaerror::MediaError;
use crate::dom::mediafragmentparser::MediaFragmentParser;
use crate::dom::medialist::MediaList;
use crate::dom::mediastream::MediaStream;
use crate::dom::node::{Node, NodeDamage, NodeTraits, UnbindContext};
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
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
use crate::network_listener::{self, FetchResponseListener, ResourceTimingListener};
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;
use crate::task_source::SendableTaskSource;

/// A CSS file to style the media controls.
static MEDIA_CONTROL_CSS: &str = include_str!("../../resources/media-controls.css");

/// A JS file to control the media controls.
static MEDIA_CONTROL_JS: &str = include_str!("../../resources/media-controls.js");

#[derive(MallocSizeOf, PartialEq)]
enum FrameStatus {
    Locked,
    Unlocked,
}

#[derive(MallocSizeOf)]
struct FrameHolder(
    FrameStatus,
    #[ignore_malloc_size_of = "defined in servo-media"] VideoFrame,
);

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

#[derive(MallocSizeOf)]
pub(crate) struct MediaFrameRenderer {
    webview_id: WebViewId,
    player_id: Option<usize>,
    glplayer_id: Option<u64>,
    paint_api: CrossProcessPaintApi,
    #[ignore_malloc_size_of = "Defined in other crates"]
    player_context: WindowGLContext,
    current_frame: Option<MediaFrame>,
    old_frame: Option<ImageKey>,
    very_old_frame: Option<ImageKey>,
    current_frame_holder: Option<FrameHolder>,
    /// <https://html.spec.whatwg.org/multipage/#poster-frame>
    poster_frame: Option<MediaFrame>,
}

impl MediaFrameRenderer {
    fn new(
        webview_id: WebViewId,
        paint_api: CrossProcessPaintApi,
        player_context: WindowGLContext,
    ) -> Self {
        Self {
            webview_id,
            player_id: None,
            glplayer_id: None,
            paint_api,
            player_context,
            current_frame: None,
            old_frame: None,
            very_old_frame: None,
            current_frame_holder: None,
            poster_frame: None,
        }
    }

    fn setup(
        &mut self,
        player_id: usize,
        task_source: SendableTaskSource,
        weak_video_renderer: Weak<Mutex<MediaFrameRenderer>>,
    ) {
        self.player_id = Some(player_id);

        let (glplayer_id, image_receiver) = self
            .player_context
            .glplayer_thread_sender
            .as_ref()
            .map(|sender| {
                let (image_sender, image_receiver) = ipc::channel::<GLPlayerMsgForward>().unwrap();
                sender
                    .send(GLPlayerMsg::RegisterPlayer(image_sender))
                    .unwrap();
                match image_receiver.recv().unwrap() {
                    GLPlayerMsgForward::PlayerId(id) => (Some(id), Some(image_receiver)),
                    _ => unreachable!(),
                }
            })
            .unwrap_or((None, None));

        self.glplayer_id = glplayer_id;

        let Some(image_receiver) = image_receiver else {
            return;
        };

        ROUTER.add_typed_route(
            image_receiver,
            Box::new(move |message| {
                let message = message.unwrap();
                let weak_video_renderer = weak_video_renderer.clone();

                task_source.queue(task!(handle_glplayer_message: move || {
                    trace!("GLPlayer message {:?}", message);

                    let Some(video_renderer) = weak_video_renderer.upgrade() else {
                        return;
                    };

                    match message {
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

    fn reset(&mut self) {
        self.player_id = None;

        if let Some(glplayer_id) = self.glplayer_id.take() {
            self.player_context
                .send(GLPlayerMsg::UnregisterPlayer(glplayer_id));
        }

        self.current_frame_holder = None;

        let mut updates = smallvec::smallvec![];

        if let Some(current_frame) = self.current_frame.take() {
            updates.push(ImageUpdate::DeleteImage(current_frame.image_key));
        }

        if let Some(old_image_key) = self.old_frame.take() {
            updates.push(ImageUpdate::DeleteImage(old_image_key));
        }

        if let Some(very_old_image_key) = self.very_old_frame.take() {
            updates.push(ImageUpdate::DeleteImage(very_old_image_key));
        }

        if !updates.is_empty() {
            self.paint_api
                .update_images(self.webview_id.into(), updates);
        }
    }

    fn set_poster_frame(&mut self, image: Option<Arc<RasterImage>>) {
        self.poster_frame = image.and_then(|image| {
            image.id.map(|image_key| MediaFrame {
                image_key,
                width: image.metadata.width as i32,
                height: image.metadata.height as i32,
            })
        });
    }
}

impl Drop for MediaFrameRenderer {
    fn drop(&mut self) {
        self.reset();
    }
}

impl VideoFrameRenderer for MediaFrameRenderer {
    fn render(&mut self, frame: VideoFrame) {
        if self.player_id.is_none() || (frame.is_gl_texture() && self.glplayer_id.is_none()) {
            return;
        }

        let mut updates = smallvec::smallvec![];

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
                        SerializableImageData::Raw(GenericSharedMemory::from_bytes(
                            &frame.get_data(),
                        )),
                        None,
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

                let Some(new_image_key) =
                    self.paint_api.generate_image_key_blocking(self.webview_id)
                else {
                    return;
                };

                /* update current_frame */
                current_frame.image_key = new_image_key;
                current_frame.width = frame.get_width();
                current_frame.height = frame.get_height();

                let image_data = if frame.is_gl_texture() && self.glplayer_id.is_some() {
                    let texture_target = if frame.is_external_oes() {
                        ImageBufferKind::TextureExternal
                    } else {
                        ImageBufferKind::Texture2D
                    };

                    SerializableImageData::External(ExternalImageData {
                        id: ExternalImageId(self.glplayer_id.unwrap()),
                        channel_index: 0,
                        image_type: ExternalImageType::TextureHandle(texture_target),
                        normalized_uvs: false,
                    })
                } else {
                    SerializableImageData::Raw(GenericSharedMemory::from_bytes(&frame.get_data()))
                };

                self.current_frame_holder
                    .get_or_insert_with(|| FrameHolder::new(frame.clone()))
                    .set(frame);

                updates.push(ImageUpdate::AddImage(new_image_key, descriptor, image_data));
            },
            None => {
                let Some(image_key) = self.paint_api.generate_image_key_blocking(self.webview_id)
                else {
                    return;
                };

                self.current_frame = Some(MediaFrame {
                    image_key,
                    width: frame.get_width(),
                    height: frame.get_height(),
                });

                let image_data = if frame.is_gl_texture() && self.glplayer_id.is_some() {
                    let texture_target = if frame.is_external_oes() {
                        ImageBufferKind::TextureExternal
                    } else {
                        ImageBufferKind::Texture2D
                    };

                    SerializableImageData::External(ExternalImageData {
                        id: ExternalImageId(self.glplayer_id.unwrap()),
                        channel_index: 0,
                        image_type: ExternalImageType::TextureHandle(texture_target),
                        normalized_uvs: false,
                    })
                } else {
                    SerializableImageData::Raw(GenericSharedMemory::from_bytes(&frame.get_data()))
                };

                self.current_frame_holder = Some(FrameHolder::new(frame));

                updates.push(ImageUpdate::AddImage(image_key, descriptor, image_data));
            },
        }
        self.paint_api
            .update_images(self.webview_id.into(), updates);
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

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
enum LoadState {
    NotLoaded,
    LoadingFromSrcObject,
    LoadingFromSrcAttribute,
    LoadingFromSourceChild,
    WaitingForSource,
}

/// <https://html.spec.whatwg.org/multipage/#loading-the-media-resource:media-element-29>
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
struct SourceChildrenPointer {
    source_before_pointer: Dom<HTMLSourceElement>,
    inclusive: bool,
}

impl SourceChildrenPointer {
    fn new(source_before_pointer: DomRoot<HTMLSourceElement>, inclusive: bool) -> Self {
        Self {
            source_before_pointer: source_before_pointer.as_traced(),
            inclusive,
        }
    }
}

/// Generally the presence of the loop attribute should be considered to mean playback has not
/// "ended", as "ended" and "looping" are mutually exclusive.
/// <https://html.spec.whatwg.org/multipage/#ended-playback>
#[derive(Clone, Copy, Debug, PartialEq)]
enum LoopCondition {
    Included,
    Ignored,
}

#[dom_struct]
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
    default_playback_rate: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-playbackrate>
    playback_rate: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#attr-media-autoplay>
    autoplaying: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#delaying-the-load-event-flag>
    delaying_the_load_event_flag: DomRefCell<Option<LoadBlocker>>,
    /// <https://html.spec.whatwg.org/multipage/#list-of-pending-play-promises>
    #[conditional_malloc_size_of]
    pending_play_promises: DomRefCell<Vec<Rc<Promise>>>,
    /// Play promises which are soon to be fulfilled by a queued task.
    #[allow(clippy::type_complexity)]
    #[conditional_malloc_size_of]
    in_flight_play_promises_queue: DomRefCell<VecDeque<(Box<[Rc<Promise>]>, ErrorResult)>>,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    player: DomRefCell<Option<Arc<Mutex<dyn Player>>>>,
    #[conditional_malloc_size_of]
    #[no_trace]
    video_renderer: Arc<Mutex<MediaFrameRenderer>>,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    audio_renderer: DomRefCell<Option<Arc<Mutex<dyn AudioRenderer>>>>,
    #[conditional_malloc_size_of]
    #[no_trace]
    event_handler: RefCell<Option<Arc<Mutex<HTMLMediaElementEventHandler>>>>,
    /// <https://html.spec.whatwg.org/multipage/#show-poster-flag>
    show_poster: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-duration>
    duration: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#current-playback-position>
    current_playback_position: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#official-playback-position>
    official_playback_position: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#default-playback-start-position>
    default_playback_start_position: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-volume>
    volume: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-seeking>
    seeking: Cell<bool>,
    /// The latest seek position (in seconds) is used to distinguish whether the seek request was
    /// initiated by a script or by the user agent itself, rather than by the media engine and to
    /// abort other running instance of the `seek` algorithm.
    current_seek_position: Cell<f64>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-muted>
    muted: Cell<bool>,
    /// Loading state from source, if any.
    load_state: Cell<LoadState>,
    source_children_pointer: DomRefCell<Option<SourceChildrenPointer>>,
    current_source_child: MutNullableDom<HTMLSourceElement>,
    /// URL of the media resource, if any.
    #[no_trace]
    resource_url: DomRefCell<Option<ServoUrl>>,
    /// URL of the media resource, if the resource is set through the src_object attribute and it
    /// is a blob.
    #[no_trace]
    blob_url: DomRefCell<Option<ServoUrl>>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-played>
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
    current_fetch_context: RefCell<Option<HTMLMediaElementFetchContext>>,
    /// Media controls id.
    /// In order to workaround the lack of privileged JS context, we secure the
    /// the access to the "privileged" document.servoGetMediaControls(id) API by
    /// keeping a whitelist of media controls identifiers.
    media_controls_id: DomRefCell<Option<String>>,
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
#[expect(clippy::enum_variant_names)] // Clippy warning silenced here because these names are from the specification.
pub(crate) enum ReadyState {
    HaveNothing = HTMLMediaElementConstants::HAVE_NOTHING as u8,
    HaveMetadata = HTMLMediaElementConstants::HAVE_METADATA as u8,
    HaveCurrentData = HTMLMediaElementConstants::HAVE_CURRENT_DATA as u8,
    HaveFutureData = HTMLMediaElementConstants::HAVE_FUTURE_DATA as u8,
    HaveEnoughData = HTMLMediaElementConstants::HAVE_ENOUGH_DATA as u8,
}

/// <https://html.spec.whatwg.org/multipage/#direction-of-playback>
#[derive(Clone, Copy, PartialEq)]
enum PlaybackDirection {
    Forwards,
    Backwards,
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
            default_playback_rate: Cell::new(1.0),
            playback_rate: Cell::new(1.0),
            muted: Cell::new(false),
            load_state: Cell::new(LoadState::NotLoaded),
            source_children_pointer: DomRefCell::new(None),
            current_source_child: Default::default(),
            // FIXME(nox): Why is this initialised to true?
            autoplaying: Cell::new(true),
            delaying_the_load_event_flag: Default::default(),
            pending_play_promises: Default::default(),
            in_flight_play_promises_queue: Default::default(),
            player: Default::default(),
            video_renderer: Arc::new(Mutex::new(MediaFrameRenderer::new(
                document.webview_id(),
                document.window().paint_api().clone(),
                document.window().get_player_context(),
            ))),
            audio_renderer: Default::default(),
            event_handler: Default::default(),
            show_poster: Cell::new(true),
            duration: Cell::new(f64::NAN),
            current_playback_position: Cell::new(0.),
            official_playback_position: Cell::new(0.),
            default_playback_start_position: Cell::new(0.),
            volume: Cell::new(1.0),
            seeking: Cell::new(false),
            current_seek_position: Cell::new(f64::NAN),
            resource_url: DomRefCell::new(None),
            blob_url: DomRefCell::new(None),
            played: DomRefCell::new(TimeRangesContainer::default()),
            audio_tracks_list: Default::default(),
            video_tracks_list: Default::default(),
            text_tracks_list: Default::default(),
            next_timeupdate_event: Cell::new(Instant::now() + Duration::from_millis(250)),
            current_fetch_context: RefCell::new(None),
            media_controls_id: DomRefCell::new(None),
        }
    }

    pub(crate) fn network_state(&self) -> NetworkState {
        self.network_state.get()
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

    fn update_media_state(&self) {
        let is_playing = self
            .player
            .borrow()
            .as_ref()
            .is_some_and(|player| !player.lock().unwrap().paused());

        if self.is_potentially_playing() && !is_playing {
            if let Some(ref player) = *self.player.borrow() {
                let player = player.lock().unwrap();

                if let Err(error) = player.set_playback_rate(self.playback_rate.get()) {
                    warn!("Could not set the playback rate: {error:?}");
                }
                if let Err(error) = player.set_volume(self.volume.get()) {
                    warn!("Could not set the volume: {error:?}");
                }
                if let Err(error) = player.play() {
                    error!("Could not play media: {error:?}");
                }
            }
        } else if is_playing {
            if let Some(ref player) = *self.player.borrow() {
                if let Err(error) = player.lock().unwrap().pause() {
                    error!("Could not pause player: {error:?}");
                }
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
        // Step 6. If the time was reached through the usual monotonic increase of the current
        // playback position during normal playback, and if the user agent has not fired a
        // timeupdate event at the element in the past 15 to 250ms and is not still running event
        // handlers for such an event, then the user agent must queue a media element task given the
        // media element to fire an event named timeupdate at the element.
        if Instant::now() > self.next_timeupdate_event.get() {
            self.queue_media_element_task_to_fire_event(atom!("timeupdate"));
            self.next_timeupdate_event
                .set(Instant::now() + Duration::from_millis(250));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#internal-play-steps>
    fn internal_play_steps(&self, can_gc: CanGc) {
        // Step 1. If the media element's networkState attribute has the value NETWORK_EMPTY, invoke
        // the media element's resource selection algorithm.
        if self.network_state.get() == NetworkState::Empty {
            self.invoke_resource_selection_algorithm(can_gc);
        }

        // Step 2. If the playback has ended and the direction of playback is forwards, seek to the
        // earliest possible position of the media resource.
        // Generally "ended" and "looping" are exclusive. Here, the loop attribute is ignored to
        // seek back to start in case loop was set after playback ended.
        // <https://github.com/whatwg/html/issues/4487>
        if self.ended_playback(LoopCondition::Ignored) &&
            self.direction_of_playback() == PlaybackDirection::Forwards
        {
            self.seek(
                self.earliest_possible_position(),
                /* approximate_for_speed */ false,
            );
        }

        let state = self.ready_state.get();

        // Step 3. If the media element's paused attribute is true, then:
        if self.Paused() {
            // Step 3.1. Change the value of paused to false.
            self.paused.set(false);

            // Step 3.2. If the show poster flag is true, set the element's show poster flag to
            // false and run the time marches on steps.
            if self.show_poster.get() {
                self.show_poster.set(false);
                self.time_marches_on();
            }

            // Step 3.3. Queue a media element task given the media element to fire an event named
            // play at the element.
            self.queue_media_element_task_to_fire_event(atom!("play"));

            // Step 3.4. If the media element's readyState attribute has the value HAVE_NOTHING,
            // HAVE_METADATA, or HAVE_CURRENT_DATA, queue a media element task given the media
            // element to fire an event named waiting at the element. Otherwise, the media element's
            // readyState attribute has the value HAVE_FUTURE_DATA or HAVE_ENOUGH_DATA: notify about
            // playing for the element.
            match state {
                ReadyState::HaveNothing |
                ReadyState::HaveMetadata |
                ReadyState::HaveCurrentData => {
                    self.queue_media_element_task_to_fire_event(atom!("waiting"));
                },
                ReadyState::HaveFutureData | ReadyState::HaveEnoughData => {
                    self.notify_about_playing();
                },
            }
        }
        // Step 4. Otherwise, if the media element's readyState attribute has the value
        // HAVE_FUTURE_DATA or HAVE_ENOUGH_DATA, take pending play promises and queue a media
        // element task given the media element to resolve pending play promises with the
        // result.
        else if state == ReadyState::HaveFutureData || state == ReadyState::HaveEnoughData {
            self.take_pending_play_promises(Ok(()));

            let this = Trusted::new(self);
            let generation_id = self.generation_id.get();

            self.owner_global()
                .task_manager()
                .media_element_task_source()
                .queue(task!(resolve_pending_play_promises: move || {
                    let this = this.root();
                    if generation_id != this.generation_id.get() {
                        return;
                    }

                    this.fulfill_in_flight_play_promises(|| {});
                }));
        }

        // Step 5. Set the media element's can autoplay flag to false.
        self.autoplaying.set(false);

        self.update_media_state();
    }

    /// <https://html.spec.whatwg.org/multipage/#internal-pause-steps>
    fn internal_pause_steps(&self) {
        // Step 1. Set the media element's can autoplay flag to false.
        self.autoplaying.set(false);

        // Step 2. If the media element's paused attribute is false, run the following steps:
        if !self.Paused() {
            // Step 2.1. Change the value of paused to true.
            self.paused.set(true);

            // Step 2.2. Take pending play promises and let promises be the result.
            self.take_pending_play_promises(Err(Error::Abort(None)));

            // Step 2.3. Queue a media element task given the media element and the following steps:
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
                        // Step 2.3.1. Fire an event named timeupdate at the element.
                        this.upcast::<EventTarget>().fire_event(atom!("timeupdate"), CanGc::note());

                        // Step 2.3.2. Fire an event named pause at the element.
                        this.upcast::<EventTarget>().fire_event(atom!("pause"), CanGc::note());

                        // Step 2.3.3. Reject pending play promises with promises and an
                        // "AbortError" DOMException.
                        // Done after running this closure in `fulfill_in_flight_play_promises`.
                    });
                }));

            // Step 2.4. Set the official playback position to the current playback position.
            self.official_playback_position
                .set(self.current_playback_position.get());
        }

        self.update_media_state();
    }

    /// <https://html.spec.whatwg.org/multipage/#allowed-to-play>
    fn is_allowed_to_play(&self) -> bool {
        true
    }

    /// <https://html.spec.whatwg.org/multipage/#notify-about-playing>
    fn notify_about_playing(&self) {
        // Step 1. Take pending play promises and let promises be the result.
        self.take_pending_play_promises(Ok(()));

        // Step 2. Queue a media element task given the element and the following steps:
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
                    // Step 2.1. Fire an event named playing at the element.
                    this.upcast::<EventTarget>().fire_event(atom!("playing"), CanGc::note());

                    // Step 2.2. Resolve pending play promises with promises.
                    // Done after running this closure in `fulfill_in_flight_play_promises`.
                });
            }));
    }

    /// <https://html.spec.whatwg.org/multipage/#ready-states>
    fn change_ready_state(&self, ready_state: ReadyState) {
        let old_ready_state = self.ready_state.get();
        self.ready_state.set(ready_state);

        if self.network_state.get() == NetworkState::Empty {
            return;
        }

        if old_ready_state == ready_state {
            return;
        }

        // Step 1. Apply the first applicable set of substeps from the following list:
        match (old_ready_state, ready_state) {
            // => "If the previous ready state was HAVE_NOTHING, and the new ready state is
            // HAVE_METADATA"
            (ReadyState::HaveNothing, ReadyState::HaveMetadata) => {
                // Queue a media element task given the media element to fire an event named
                // loadedmetadata at the element.
                self.queue_media_element_task_to_fire_event(atom!("loadedmetadata"));
                // No other steps are applicable in this case.
                return;
            },
            // => "If the previous ready state was HAVE_METADATA and the new ready state is
            // HAVE_CURRENT_DATA or greater"
            (ReadyState::HaveMetadata, new) if new >= ReadyState::HaveCurrentData => {
                // If this is the first time this occurs for this media element since the load()
                // algorithm was last invoked, the user agent must queue a media element task given
                // the media element to fire an event named loadeddata at the element.
                if !self.fired_loadeddata_event.get() {
                    self.fired_loadeddata_event.set(true);

                    let this = Trusted::new(self);
                    let generation_id = self.generation_id.get();

                    self.owner_global()
                        .task_manager()
                        .media_element_task_source()
                        .queue(task!(media_reached_current_data: move || {
                            let this = this.root();
                            if generation_id != this.generation_id.get() {
                                return;
                            }

                            this.upcast::<EventTarget>().fire_event(atom!("loadeddata"), CanGc::note());
                            // Once the readyState attribute reaches HAVE_CURRENT_DATA, after the
                            // loadeddata event has been fired, set the element's
                            // delaying-the-load-event flag to false.
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

        // => "If the previous ready state was HAVE_CURRENT_DATA or less, and the new ready state is
        // HAVE_FUTURE_DATA or more"
        if old_ready_state <= ReadyState::HaveCurrentData &&
            ready_state >= ReadyState::HaveFutureData
        {
            // The user agent must queue a media element task given the media element to fire an
            // event named canplay at the element.
            self.queue_media_element_task_to_fire_event(atom!("canplay"));

            // If the element's paused attribute is false, the user agent must notify about playing
            // for the element.
            if !self.Paused() {
                self.notify_about_playing();
            }
        }

        // => "If the new ready state is HAVE_ENOUGH_DATA"
        if ready_state == ReadyState::HaveEnoughData {
            // The user agent must queue a media element task given the media element to fire an
            // event named canplaythrough at the element.
            self.queue_media_element_task_to_fire_event(atom!("canplaythrough"));

            // If the element is eligible for autoplay, then the user agent may run the following
            // substeps:
            if self.eligible_for_autoplay() {
                // Step 1. Set the paused attribute to false.
                self.paused.set(false);

                // Step 2. If the element's show poster flag is true, set it to false and run the
                // time marches on steps.
                if self.show_poster.get() {
                    self.show_poster.set(false);
                    self.time_marches_on();
                }

                // Step 3. Queue a media element task given the element to fire an event named play
                // at the element.
                self.queue_media_element_task_to_fire_event(atom!("play"));

                // Step 4. Notify about playing for the element.
                self.notify_about_playing();
            }
        }

        self.update_media_state();
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm>
    fn invoke_resource_selection_algorithm(&self, can_gc: CanGc) {
        // Step 1. Set the element's networkState attribute to the NETWORK_NO_SOURCE value.
        self.network_state.set(NetworkState::NoSource);

        // Step 2. Set the element's show poster flag to true.
        self.show_poster.set(true);

        // Step 3. Set the media element's delaying-the-load-event flag to true (this delays the
        // load event).
        self.delay_load_event(true, can_gc);

        // Step 4. Await a stable state, allowing the task that invoked this algorithm to continue.
        // If the resource selection mode in the synchronous section is
        // "attribute", the URL of the resource to fetch is relative to the
        // media element's node document when the src attribute was last
        // changed, which is why we need to pass the base URL in the task
        // right here.
        let task = MediaElementMicrotask::ResourceSelection {
            elem: DomRoot::from_ref(self),
            generation_id: self.generation_id.get(),
            base_url: self.owner_document().base_url(),
        };

        // FIXME(nox): This will later call the resource_selection_algorithm_sync
        // method from below, if microtasks were trait objects, we would be able
        // to put the code directly in this method, without the boilerplate
        // indirections.
        ScriptThread::await_stable_state(Microtask::MediaElement(task));
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm>
    fn resource_selection_algorithm_sync(&self, base_url: ServoUrl, can_gc: CanGc) {
        // TODO Step 5. If the media element's blocked-on-parser flag is false, then populate the
        // list of pending text tracks.
        // FIXME(ferjm): Implement blocked_on_parser logic
        // https://html.spec.whatwg.org/multipage/#blocked-on-parser
        // FIXME(nox): Maybe populate the list of pending text tracks.

        enum Mode {
            Object,
            Attribute(String),
            Children(DomRoot<HTMLSourceElement>),
        }

        // Step 6.
        let mode = if self.src_object.borrow().is_some() {
            // If the media element has an assigned media provider object, then let mode be object.
            Mode::Object
        } else if let Some(attribute) = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("src"))
        {
            // Otherwise, if the media element has no assigned media provider object but has a src
            // attribute, then let mode be attribute.
            Mode::Attribute((**attribute.value()).to_owned())
        } else if let Some(source) = self
            .upcast::<Node>()
            .children()
            .find_map(DomRoot::downcast::<HTMLSourceElement>)
        {
            // Otherwise, if the media element does not have an assigned media provider object and
            // does not have a src attribute, but does have a source element child, then let mode be
            // children and let candidate be the first such source element child in tree order.
            Mode::Children(source)
        } else {
            // Otherwise, the media element has no assigned media provider object and has neither a
            // src attribute nor a source element child:
            self.load_state.set(LoadState::NotLoaded);

            // Step 6.none.1. Set the networkState to NETWORK_EMPTY.
            self.network_state.set(NetworkState::Empty);

            // Step 6.none.2. Set the element's delaying-the-load-event flag to false. This stops
            // delaying the load event.
            self.delay_load_event(false, can_gc);

            // Step 6.none.3. End the synchronous section and return.
            return;
        };

        // Step 7. Set the media element's networkState to NETWORK_LOADING.
        self.network_state.set(NetworkState::Loading);

        // Step 8. Queue a media element task given the media element to fire an event named
        // loadstart at the media element.
        self.queue_media_element_task_to_fire_event(atom!("loadstart"));

        // Step 9. Run the appropriate steps from the following list:
        match mode {
            Mode::Object => {
                // => "If mode is object"
                self.load_from_src_object();
            },
            Mode::Attribute(src) => {
                // => "If mode is attribute"
                self.load_from_src_attribute(base_url, &src);
            },
            Mode::Children(source) => {
                // => "Otherwise (mode is children)""
                self.load_from_source_child(&source);
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm>
    fn load_from_src_object(&self) {
        self.load_state.set(LoadState::LoadingFromSrcObject);

        // Step 9.object.1. Set the currentSrc attribute to the empty string.
        "".clone_into(&mut self.current_src.borrow_mut());

        // Step 9.object.3. Run the resource fetch algorithm with the assigned media
        // provider object. If that algorithm returns without aborting this one, then the
        // load failed.
        // Note that the resource fetch algorithm itself takes care of the cleanup in case
        // of failure itself.
        self.resource_fetch_algorithm(Resource::Object);
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm>
    fn load_from_src_attribute(&self, base_url: ServoUrl, src: &str) {
        self.load_state.set(LoadState::LoadingFromSrcAttribute);

        // Step 9.attribute.1. If the src attribute's value is the empty string, then end
        // the synchronous section, and jump down to the failed with attribute step below.
        if src.is_empty() {
            self.queue_dedicated_media_source_failure_steps();
            return;
        }

        // Step 9.attribute.2. Let urlRecord be the result of encoding-parsing a URL given
        // the src attribute's value, relative to the media element's node document when the
        // src attribute was last changed.
        let Ok(url_record) = base_url.join(src) else {
            self.queue_dedicated_media_source_failure_steps();
            return;
        };

        // Step 9.attribute.3. If urlRecord is not failure, then set the currentSrc
        // attribute to the result of applying the URL serializer to urlRecord.
        *self.current_src.borrow_mut() = url_record.as_str().into();

        // Step 9.attribute.5. If urlRecord is not failure, then run the resource fetch
        // algorithm with urlRecord. If that algorithm returns without aborting this one,
        // then the load failed.
        // Note that the resource fetch algorithm itself takes care
        // of the cleanup in case of failure itself.
        self.resource_fetch_algorithm(Resource::Url(url_record));
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm>
    fn load_from_source_child(&self, source: &HTMLSourceElement) {
        self.load_state.set(LoadState::LoadingFromSourceChild);

        // Step 9.children.1. Let pointer be a position defined by two adjacent nodes in the media
        // element's child list, treating the start of the list (before the first child in the list,
        // if any) and end of the list (after the last child in the list, if any) as nodes in their
        // own right. One node is the node before pointer, and the other node is the node after
        // pointer. Initially, let pointer be the position between the candidate node and the next
        // node, if there are any, or the end of the list, if it is the last node.
        *self.source_children_pointer.borrow_mut() =
            Some(SourceChildrenPointer::new(DomRoot::from_ref(source), false));

        let element = source.upcast::<Element>();

        // Step 9.children.2. Process candidate: If candidate does not have a src attribute, or if
        // its src attribute's value is the empty string, then end the synchronous section, and jump
        // down to the failed with elements step below.
        let Some(src) = element
            .get_attribute(&ns!(), &local_name!("src"))
            .filter(|attribute| !attribute.value().is_empty())
        else {
            self.load_from_source_child_failure_steps(source);
            return;
        };

        // Step 9.children.3. If candidate has a media attribute whose value does not match the
        // environment, then end the synchronous section, and jump down to the failed with elements
        // step below.
        if let Some(media) = element.get_attribute(&ns!(), &local_name!("media")) {
            if !MediaList::matches_environment(&element.owner_document(), &media.value()) {
                self.load_from_source_child_failure_steps(source);
                return;
            }
        }

        // Step 9.children.4. Let urlRecord be the result of encoding-parsing a URL given
        // candidate's src attribute's value, relative to candidate's node document when the src
        // attribute was last changed.
        let Ok(url_record) = source.owner_document().base_url().join(&src.value()) else {
            // Step 9.children.5. If urlRecord is failure, then end the synchronous section,
            // and jump down to the failed with elements step below.
            self.load_from_source_child_failure_steps(source);
            return;
        };

        // Step 9.children.6. If candidate has a type attribute whose value, when parsed as a MIME
        // type (including any codecs described by the codecs parameter, for types that define that
        // parameter), represents a type that the user agent knows it cannot render, then end the
        // synchronous section, and jump down to the failed with elements step below.
        if let Some(type_) = element.get_attribute(&ns!(), &local_name!("type")) {
            if ServoMedia::get().can_play_type(&type_.value()) == SupportsMediaType::No {
                self.load_from_source_child_failure_steps(source);
                return;
            }
        }

        // Reset the media player before loading the next source child.
        self.reset_media_player();

        self.current_source_child.set(Some(source));

        // Step 9.children.7. Set the currentSrc attribute to the result of applying the URL
        // serializer to urlRecord.
        *self.current_src.borrow_mut() = url_record.as_str().into();

        // Step 9.children.9. Run the resource fetch algorithm with urlRecord. If that
        // algorithm returns without aborting this one, then the load failed.
        // Note that the resource fetch algorithm itself takes care
        // of the cleanup in case of failure itself.
        self.resource_fetch_algorithm(Resource::Url(url_record));
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm>
    fn load_from_source_child_failure_steps(&self, source: &HTMLSourceElement) {
        // Step 9.children.10. Failed with elements: Queue a media element task given the media
        // element to fire an event named error at candidate.
        let trusted_this = Trusted::new(self);
        let trusted_source = Trusted::new(source);
        let generation_id = self.generation_id.get();

        self.owner_global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(queue_error_event: move || {
                let this = trusted_this.root();
                if generation_id != this.generation_id.get() {
                    return;
                }

                let source = trusted_source.root();
                source.upcast::<EventTarget>().fire_event(atom!("error"), CanGc::note());
            }));

        // Step 9.children.11. Await a stable state.
        let task = MediaElementMicrotask::SelectNextSourceChild {
            elem: DomRoot::from_ref(self),
            generation_id: self.generation_id.get(),
        };

        ScriptThread::await_stable_state(Microtask::MediaElement(task));
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm>
    fn select_next_source_child(&self, can_gc: CanGc) {
        // Step 9.children.12. Forget the media element's media-resource-specific tracks.
        self.AudioTracks(can_gc).clear();
        self.VideoTracks(can_gc).clear();

        // Step 9.children.13. Find next candidate: Let candidate be null.
        let mut source_candidate = None;

        // Step 9.children.14. Search loop: If the node after pointer is the end of the list, then
        // jump to the waiting step below.
        // Step 9.children.15. If the node after pointer is a source element, let candidate be that
        // element.
        // Step 9.children.16. Advance pointer so that the node before pointer is now the node that
        // was after pointer, and the node after pointer is the node after the node that used to be
        // after pointer, if any.
        if let Some(ref source_children_pointer) = *self.source_children_pointer.borrow() {
            // Note that shared implementation between opaque types from
            // `inclusively_following_siblings` and `following_siblings` if not possible due to
            // precise capturing.
            if source_children_pointer.inclusive {
                for next_sibling in source_children_pointer
                    .source_before_pointer
                    .upcast::<Node>()
                    .inclusively_following_siblings()
                {
                    if let Some(next_source) = DomRoot::downcast::<HTMLSourceElement>(next_sibling)
                    {
                        source_candidate = Some(next_source);
                        break;
                    }
                }
            } else {
                for next_sibling in source_children_pointer
                    .source_before_pointer
                    .upcast::<Node>()
                    .following_siblings()
                {
                    if let Some(next_source) = DomRoot::downcast::<HTMLSourceElement>(next_sibling)
                    {
                        source_candidate = Some(next_source);
                        break;
                    }
                }
            };
        }

        // Step 9.children.17. If candidate is null, jump back to the search loop step. Otherwise,
        // jump back to the process candidate step.
        if let Some(source_candidate) = source_candidate {
            self.load_from_source_child(&source_candidate);
            return;
        }

        self.load_state.set(LoadState::WaitingForSource);

        *self.source_children_pointer.borrow_mut() = None;

        // Step 9.children.18. Waiting: Set the element's networkState attribute to the
        // NETWORK_NO_SOURCE value.
        self.network_state.set(NetworkState::NoSource);

        // Step 9.children.19. Set the element's show poster flag to true.
        self.show_poster.set(true);

        // Step 9.children.20. Queue a media element task given the media element to set the
        // element's delaying-the-load-event flag to false. This stops delaying the load event.
        let this = Trusted::new(self);
        let generation_id = self.generation_id.get();

        self.owner_global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(queue_delay_load_event: move || {
                let this = this.root();
                if generation_id != this.generation_id.get() {
                    return;
                }

                this.delay_load_event(false, CanGc::note());
            }));

        // Step 9.children.22. Wait until the node after pointer is a node other than the end of the
        // list. (This step might wait forever.)
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm>
    fn resource_selection_algorithm_failure_steps(&self) {
        match self.load_state.get() {
            LoadState::LoadingFromSrcObject => {
                // Step 9.object.4. Failed with media provider: Reaching this step indicates that
                // the media resource failed to load. Take pending play promises and queue a media
                // element task given the media element to run the dedicated media source failure
                // steps with the result.
                self.queue_dedicated_media_source_failure_steps();
            },
            LoadState::LoadingFromSrcAttribute => {
                // Step 9.attribute.6. Failed with attribute: Reaching this step indicates that the
                // media resource failed to load or that urlRecord is failure. Take pending play
                // promises and queue a media element task given the media element to run the
                // dedicated media source failure steps with the result.
                self.queue_dedicated_media_source_failure_steps();
            },
            LoadState::LoadingFromSourceChild => {
                // Step 9.children.10. Failed with elements: Queue a media element task given the
                // media element to fire an event named error at candidate.
                if let Some(source) = self.current_source_child.take() {
                    self.load_from_source_child_failure_steps(&source);
                }
            },
            _ => {},
        }
    }

    fn fetch_request(&self, offset: Option<u64>, seek_lock: Option<SeekLock>) {
        if self.resource_url.borrow().is_none() && self.blob_url.borrow().is_none() {
            error!("Missing request url");
            if let Some(seek_lock) = seek_lock {
                seek_lock.unlock(/* successful seek */ false);
            }
            self.resource_selection_algorithm_failure_steps();
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
        let global = self.global();
        let request = create_a_potential_cors_request(
            Some(document.webview_id()),
            url.clone(),
            destination,
            cors_setting,
            None,
            global.get_referrer(),
            document.insecure_requests_policy(),
            document.has_trustworthy_ancestor_or_current_origin(),
            global.policy_container(),
        )
        .headers(headers)
        .origin(document.origin().immutable().clone())
        .pipeline_id(Some(self.global().pipeline_id()))
        .referrer_policy(document.get_referrer_policy());

        let mut current_fetch_context = self.current_fetch_context.borrow_mut();
        if let Some(ref mut current_fetch_context) = *current_fetch_context {
            current_fetch_context.cancel(CancelReason::Abort);
        }

        *current_fetch_context = Some(HTMLMediaElementFetchContext::new(
            request.id,
            global.core_resource_thread(),
        ));
        let listener =
            HTMLMediaElementFetchListener::new(self, request.id, url.clone(), offset.unwrap_or(0));

        self.owner_document().fetch_background(request, listener);

        // Since we cancelled the previous fetch, from now on the media element
        // will only receive response data from the new fetch that's been
        // initiated. This means the player can resume operation, since all subsequent data
        // pushes will originate from the new seek offset.
        if let Some(seek_lock) = seek_lock {
            seek_lock.unlock(/* successful seek */ true);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#eligible-for-autoplay>
    fn eligible_for_autoplay(&self) -> bool {
        // its can autoplay flag is true;
        self.autoplaying.get() &&

        // its paused attribute is true;
        self.Paused() &&

        // it has an autoplay attribute specified;
        self.Autoplay() &&

        // its node document's active sandboxing flag set does not have the sandboxed automatic
        // features browsing context flag set; and
        {
            let document = self.owner_document();

            !document.has_active_sandboxing_flag(
                SandboxingFlagSet::SANDBOXED_AUTOMATIC_FEATURES_BROWSING_CONTEXT_FLAG,
            )
        }

        // its node document is allowed to use the "autoplay" feature.
        // TODO: Feature policy: https://html.spec.whatwg.org/iframe-embed-object.html#allowed-to-use
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-media-load-resource>
    fn resource_fetch_algorithm(&self, resource: Resource) {
        if let Err(e) = self.create_media_player(&resource) {
            error!("Create media player error {:?}", e);
            self.resource_selection_algorithm_failure_steps();
            return;
        }

        // Steps 1-2.
        // Unapplicable, the `resource` variable already conveys which mode
        // is in use.

        // Step 3.
        // FIXME(nox): Remove all media-resource-specific text tracks.

        // Step 5. Run the appropriate steps from the following list:
        match resource {
            Resource::Url(url) => {
                // Step 5.remote.1. Optionally, run the following substeps. This is the expected
                // behavior if the user agent intends to not attempt to fetch the resource until the
                // user requests it explicitly (e.g. as a way to implement the preload attribute's
                // none keyword).
                if self.Preload() == "none" && !self.autoplaying.get() {
                    // Step 5.remote.1.1. Set the networkState to NETWORK_IDLE.
                    self.network_state.set(NetworkState::Idle);

                    // Step 5.remote.1.2. Queue a media element task given the media element to fire
                    // an event named suspend at the element.
                    self.queue_media_element_task_to_fire_event(atom!("suspend"));

                    // Step 5.remote.1.3. Queue a media element task given the media element to set
                    // the element's delaying-the-load-event flag to false. This stops delaying the
                    // load event.
                    let this = Trusted::new(self);
                    let generation_id = self.generation_id.get();

                    self.owner_global()
                        .task_manager()
                        .media_element_task_source()
                        .queue(task!(queue_delay_load_event: move || {
                            let this = this.root();
                            if generation_id != this.generation_id.get() {
                                return;
                            }

                            this.delay_load_event(false, CanGc::note());
                        }));

                    // TODO Steps 5.remote.1.4. Wait for the task to be run.
                    // FIXME(nox): Somehow we should wait for the task from previous
                    // step to be ran before continuing.

                    // TODO Steps 5.remote.1.5-5.remote.1.7.
                    // FIXME(nox): Wait for an implementation-defined event and
                    // then continue with the normal set of steps instead of just
                    // returning.
                    return;
                }

                *self.resource_url.borrow_mut() = Some(url);

                // Steps 5.remote.2-5.remote.8
                self.fetch_request(None, None);
            },
            Resource::Object => {
                if let Some(ref src_object) = *self.src_object.borrow() {
                    match src_object {
                        SrcObject::Blob(blob) => {
                            let blob_url = URL::CreateObjectURL(&self.global(), blob);
                            *self.blob_url.borrow_mut() =
                                Some(ServoUrl::parse(&blob_url.str()).expect("infallible"));
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
                                    self.resource_selection_algorithm_failure_steps();
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
        self.take_pending_play_promises(Err(Error::NotSupported(None)));
        self.owner_global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(dedicated_media_source_failure_steps: move || {
                let this = this.root();
                if generation_id != this.generation_id.get() {
                    return;
                }

                this.fulfill_in_flight_play_promises(|| {
                    // Step 1. Set the error attribute to the result of creating a MediaError with
                    // MEDIA_ERR_SRC_NOT_SUPPORTED.
                    this.error.set(Some(&*MediaError::new(
                        &this.owner_window(),
                        MEDIA_ERR_SRC_NOT_SUPPORTED, CanGc::note())));

                    // Step 2. Forget the media element's media-resource-specific tracks.
                    this.AudioTracks(CanGc::note()).clear();
                    this.VideoTracks(CanGc::note()).clear();

                    // Step 3. Set the element's networkState attribute to the NETWORK_NO_SOURCE
                    // value.
                    this.network_state.set(NetworkState::NoSource);

                    // Step 4. Set the element's show poster flag to true.
                    this.show_poster.set(true);

                    // Step 5. Fire an event named error at the media element.
                    this.upcast::<EventTarget>().fire_event(atom!("error"), CanGc::note());

                    if let Some(ref player) = *this.player.borrow() {
                        if let Err(error) = player.lock().unwrap().stop() {
                            error!("Could not stop player: {error:?}");
                        }
                    }

                    // Step 6. Reject pending play promises with promises and a "NotSupportedError"
                    // DOMException.
                    // Done after running this closure in `fulfill_in_flight_play_promises`.
                });

                // Step 7. Set the element's delaying-the-load-event flag to false. This stops
                // delaying the load event.
                this.delay_load_event(false, CanGc::note());
            }));
    }

    fn in_error_state(&self) -> bool {
        self.error.get().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#potentially-playing>
    fn is_potentially_playing(&self) -> bool {
        !self.paused.get() &&
            !self.ended_playback(LoopCondition::Included) &&
            self.error.get().is_none() &&
            !self.is_blocked_media_element()
    }

    /// <https://html.spec.whatwg.org/multipage/#blocked-media-element>
    fn is_blocked_media_element(&self) -> bool {
        self.ready_state.get() <= ReadyState::HaveCurrentData ||
            self.is_paused_for_user_interaction() ||
            self.is_paused_for_in_band_content()
    }

    /// <https://html.spec.whatwg.org/multipage/#paused-for-user-interaction>
    fn is_paused_for_user_interaction(&self) -> bool {
        // FIXME: we will likely be able to fill this placeholder once (if) we
        //        implement the MediaSession API.
        false
    }

    /// <https://html.spec.whatwg.org/multipage/#paused-for-in-band-content>
    fn is_paused_for_in_band_content(&self) -> bool {
        // FIXME: we will likely be able to fill this placeholder once (if) we
        //        implement https://github.com/servo/servo/issues/22314
        false
    }

    /// <https://html.spec.whatwg.org/multipage/#media-element-load-algorithm>
    fn media_element_load_algorithm(&self, can_gc: CanGc) {
        // Reset the flag that signals whether loadeddata was ever fired for
        // this invokation of the load algorithm.
        self.fired_loadeddata_event.set(false);

        // TODO Step 1. Set this element's is currently stalled to false.

        // Step 2. Abort any already-running instance of the resource selection algorithm for this
        // element.
        self.generation_id.set(self.generation_id.get() + 1);

        self.load_state.set(LoadState::NotLoaded);
        *self.source_children_pointer.borrow_mut() = None;
        self.current_source_child.set(None);

        // Step 3. Let pending tasks be a list of all tasks from the media element's media element
        // event task source in one of the task queues.

        // Step 4. For each task in pending tasks that would resolve pending play promises or reject
        // pending play promises, immediately resolve or reject those promises in the order the
        // corresponding tasks were queued.
        while !self.in_flight_play_promises_queue.borrow().is_empty() {
            self.fulfill_in_flight_play_promises(|| ());
        }

        // Step 5. Remove each task in pending tasks from its task queue.
        // Note that each media element's pending event and callback is scheduled with associated
        // generation id and will be aborted eventually (from Step 2).

        let network_state = self.network_state.get();

        // Step 6. If the media element's networkState is set to NETWORK_LOADING or NETWORK_IDLE,
        // queue a media element task given the media element to fire an event named abort at the
        // media element.
        if network_state == NetworkState::Loading || network_state == NetworkState::Idle {
            self.queue_media_element_task_to_fire_event(atom!("abort"));
        }

        // Reset the media player for any previously playing media resource (see Step 11).
        self.reset_media_player();

        // Step 7. If the media element's networkState is not set to NETWORK_EMPTY, then:
        if network_state != NetworkState::Empty {
            // Step 7.1. Queue a media element task given the media element to fire an event named
            // emptied at the media element.
            self.queue_media_element_task_to_fire_event(atom!("emptied"));

            // Step 7.2. If a fetching process is in progress for the media element, the user agent
            // should stop it.
            if let Some(ref mut current_fetch_context) = *self.current_fetch_context.borrow_mut() {
                current_fetch_context.cancel(CancelReason::Abort);
            }

            // TODO Step 7.3. If the media element's assigned media provider object is a MediaSource
            // object, then detach it.

            // Step 7.4. Forget the media element's media-resource-specific tracks.
            self.AudioTracks(can_gc).clear();
            self.VideoTracks(can_gc).clear();

            // Step 7.5. If readyState is not set to HAVE_NOTHING, then set it to that state.
            if self.ready_state.get() != ReadyState::HaveNothing {
                self.change_ready_state(ReadyState::HaveNothing);
            }

            // Step 7.6. If the paused attribute is false, then:
            if !self.Paused() {
                // Step 7.6.1. Set the paused attribute to true.
                self.paused.set(true);

                // Step 7.6.2. Take pending play promises and reject pending play promises with the
                // result and an "AbortError" DOMException.
                self.take_pending_play_promises(Err(Error::Abort(None)));
                self.fulfill_in_flight_play_promises(|| ());
            }

            // Step 7.7. If seeking is true, set it to false.
            self.seeking.set(false);

            self.current_seek_position.set(f64::NAN);

            // Step 7.8. Set the current playback position to 0.
            // Set the official playback position to 0.
            // If this changed the official playback position, then queue a media element task given
            // the media element to fire an event named timeupdate at the media element.
            self.current_playback_position.set(0.);
            if self.official_playback_position.get() != 0. {
                self.queue_media_element_task_to_fire_event(atom!("timeupdate"));
            }
            self.official_playback_position.set(0.);

            // TODO Step 7.9. Set the timeline offset to Not-a-Number (NaN).

            // Step 7.10. Update the duration attribute to Not-a-Number (NaN).
            self.duration.set(f64::NAN);
        }

        // Step 8. Set the playbackRate attribute to the value of the defaultPlaybackRate attribute.
        self.playback_rate.set(self.default_playback_rate.get());

        // Step 9. Set the error attribute to null and the can autoplay flag to true.
        self.error.set(None);
        self.autoplaying.set(true);

        // Step 10. Invoke the media element's resource selection algorithm.
        self.invoke_resource_selection_algorithm(can_gc);

        // Step 11. Note: Playback of any previously playing media resource for this element stops.
    }

    /// Queue a media element task given the media element to fire an event at the media element.
    /// <https://html.spec.whatwg.org/multipage/#queue-a-media-element-task>
    fn queue_media_element_task_to_fire_event(&self, name: Atom) {
        let this = Trusted::new(self);
        let generation_id = self.generation_id.get();

        self.owner_global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(queue_event: move || {
                let this = this.root();
                if generation_id != this.generation_id.get() {
                    return;
                }

                this.upcast::<EventTarget>().fire_event(name, CanGc::note());
            }));
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

    pub(crate) fn handle_source_child_insertion(&self, source: &HTMLSourceElement, can_gc: CanGc) {
        // <https://html.spec.whatwg.org/multipage/#the-source-element:html-element-insertion-steps>
        // Step 2. If parent is a media element that has no src attribute and whose networkState has
        // the value NETWORK_EMPTY, then invoke that media element's resource selection algorithm.
        if self.upcast::<Element>().has_attribute(&local_name!("src")) {
            return;
        }

        if self.network_state.get() == NetworkState::Empty {
            self.invoke_resource_selection_algorithm(can_gc);
            return;
        }

        // <https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm>
        // Step 9.children.22. Wait until the node after pointer is a node other than the end of the
        // list. (This step might wait forever.)
        if self.load_state.get() != LoadState::WaitingForSource {
            return;
        }

        self.load_state.set(LoadState::LoadingFromSourceChild);

        *self.source_children_pointer.borrow_mut() =
            Some(SourceChildrenPointer::new(DomRoot::from_ref(source), true));

        // Step 9.children.23. Await a stable state.
        let task = MediaElementMicrotask::SelectNextSourceChildAfterWait {
            elem: DomRoot::from_ref(self),
            generation_id: self.generation_id.get(),
        };

        ScriptThread::await_stable_state(Microtask::MediaElement(task));
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm>
    fn select_next_source_child_after_wait(&self, can_gc: CanGc) {
        // Step 9.children.24. Set the element's delaying-the-load-event flag back to true (this
        // delays the load event again, in case it hasn't been fired yet).
        self.delay_load_event(true, can_gc);

        // Step 9.children.25. Set the networkState back to NETWORK_LOADING.
        self.network_state.set(NetworkState::Loading);

        // Step 9.children.26. Jump back to the find next candidate step above.
        self.select_next_source_child(can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list>
    /// => "If the media data cannot be fetched at all, due to network errors..."
    /// => "If the media data can be fetched but is found by inspection to be in an unsupported
    /// format, or can otherwise not be rendered at all"
    fn media_data_processing_failure_steps(&self) {
        // Step 1. The user agent should cancel the fetching process.
        if let Some(ref mut current_fetch_context) = *self.current_fetch_context.borrow_mut() {
            current_fetch_context.cancel(CancelReason::Error);
        }

        // Step 2. Abort this subalgorithm, returning to the resource selection algorithm.
        self.resource_selection_algorithm_failure_steps();
    }

    /// <https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list>
    /// => "If the connection is interrupted after some media data has been received..."
    /// => "If the media data is corrupted"
    fn media_data_processing_fatal_steps(&self, error: u16, can_gc: CanGc) {
        *self.source_children_pointer.borrow_mut() = None;
        self.current_source_child.set(None);

        // Step 1. The user agent should cancel the fetching process.
        if let Some(ref mut current_fetch_context) = *self.current_fetch_context.borrow_mut() {
            current_fetch_context.cancel(CancelReason::Error);
        }

        // Step 2. Set the error attribute to the result of creating a MediaError with
        // MEDIA_ERR_NETWORK/MEDIA_ERR_DECODE.
        self.error
            .set(Some(&*MediaError::new(&self.owner_window(), error, can_gc)));

        // Step 3. Set the element's networkState attribute to the NETWORK_IDLE value.
        self.network_state.set(NetworkState::Idle);

        // Step 4. Set the element's delaying-the-load-event flag to false. This stops delaying
        // the load event.
        self.delay_load_event(false, can_gc);

        // Step 5. Fire an event named error at the media element.
        self.upcast::<EventTarget>()
            .fire_event(atom!("error"), can_gc);

        // Step 6. Abort the overall resource selection algorithm.
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-seek>
    fn seek(&self, time: f64, _approximate_for_speed: bool) {
        // Step 1. Set the media element's show poster flag to false.
        self.show_poster.set(false);

        // Step 2. If the media element's readyState is HAVE_NOTHING, return.
        if self.ready_state.get() == ReadyState::HaveNothing {
            return;
        }

        // Step 3. If the element's seeking IDL attribute is true, then another instance of this
        // algorithm is already running. Abort that other instance of the algorithm without waiting
        // for the step that it is running to complete.
        self.current_seek_position.set(f64::NAN);

        // Step 4. Set the seeking IDL attribute to true.
        self.seeking.set(true);

        // Step 5. If the seek was in response to a DOM method call or setting of an IDL attribute,
        // then continue the script. The remainder of these steps must be run in parallel.

        // Step 6. If the new playback position is later than the end of the media resource, then
        // let it be the end of the media resource instead.
        let time = f64::min(time, self.Duration());

        // Step 7. If the new playback position is less than the earliest possible position, let it
        // be that position instead.
        let time = f64::max(time, self.earliest_possible_position());

        // Step 8. If the (possibly now changed) new playback position is not in one of the ranges
        // given in the seekable attribute, then let it be the position in one of the ranges given
        // in the seekable attribute that is the nearest to the new playback position. If there are
        // no ranges given in the seekable attribute, then set the seeking IDL attribute to false
        // and return.
        let seekable = self.seekable();

        if seekable.is_empty() {
            self.seeking.set(false);
            return;
        }

        let mut nearest_seekable_position = 0.0;
        let mut in_seekable_range = false;
        let mut nearest_seekable_distance = f64::MAX;
        for i in 0..seekable.len() {
            let start = seekable.start(i).unwrap().abs();
            let end = seekable.end(i).unwrap().abs();
            if time >= start && time <= end {
                nearest_seekable_position = time;
                in_seekable_range = true;
                break;
            } else if time < start {
                let distance = start - time;
                if distance < nearest_seekable_distance {
                    nearest_seekable_distance = distance;
                    nearest_seekable_position = start;
                }
            } else {
                let distance = time - end;
                if distance < nearest_seekable_distance {
                    nearest_seekable_distance = distance;
                    nearest_seekable_position = end;
                }
            }
        }
        let time = if in_seekable_range {
            time
        } else {
            nearest_seekable_position
        };

        // Step 9. If the approximate-for-speed flag is set, adjust the new playback position to a
        // value that will allow for playback to resume promptly. If new playback position before
        // this step is before current playback position, then the adjusted new playback position
        // must also be before the current playback position. Similarly, if the new playback
        // position before this step is after current playback position, then the adjusted new
        // playback position must also be after the current playback position.
        // TODO: Note that servo-media with gstreamer does not support inaccurate seeking for now.

        // Step 10. Queue a media element task given the media element to fire an event named
        // seeking at the element.
        self.queue_media_element_task_to_fire_event(atom!("seeking"));

        // Step 11. Set the current playback position to the new playback position.
        self.current_playback_position.set(time);

        if let Some(ref player) = *self.player.borrow() {
            if let Err(error) = player.lock().unwrap().seek(time) {
                error!("Could not seek player: {error:?}");
            }
        }

        self.current_seek_position.set(time);

        // Step 12. Wait until the user agent has established whether or not the media data for the
        // new playback position is available, and, if it is, until it has decoded enough data to
        // play back that position.
        // The rest of the steps are handled when the media engine signals a ready state change or
        // otherwise satisfies seek completion and signals a position change.
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-seek>
    fn seek_end(&self) {
        // Any time the user agent provides a stable state, the official playback position must be
        // set to the current playback position.
        self.official_playback_position
            .set(self.current_playback_position.get());

        // Step 14. Set the seeking IDL attribute to false.
        self.seeking.set(false);

        self.current_seek_position.set(f64::NAN);

        // Step 15. Run the time marches on steps.
        self.time_marches_on();

        // Step 16. Queue a media element task given the media element to fire an event named
        // timeupdate at the element.
        self.queue_media_element_task_to_fire_event(atom!("timeupdate"));

        // Step 17. Queue a media element task given the media element to fire an event named seeked
        // at the element.
        self.queue_media_element_task_to_fire_event(atom!("seeked"));
    }

    /// <https://html.spec.whatwg.org/multipage/#poster-frame>
    pub(crate) fn set_poster_frame(&self, image: Option<Arc<RasterImage>>) {
        if pref!(media_testing_enabled) && image.is_some() {
            self.queue_media_element_task_to_fire_event(atom!("postershown"));
        }

        self.video_renderer.lock().unwrap().set_poster_frame(image);

        self.upcast::<Node>().dirty(NodeDamage::Other);
    }

    fn player_id(&self) -> Option<usize> {
        self.player
            .borrow()
            .as_ref()
            .map(|player| player.lock().unwrap().get_id())
    }

    fn create_media_player(&self, resource: &Resource) -> Result<(), ()> {
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
        let player_id = {
            let player_guard = player.lock().unwrap();

            if let Err(error) = player_guard.set_mute(self.muted.get()) {
                warn!("Could not set mute state: {error:?}");
            }

            player_guard.get_id()
        };

        *self.player.borrow_mut() = Some(player);

        let event_handler = Arc::new(Mutex::new(HTMLMediaElementEventHandler::new(self)));
        let weak_event_handler = Arc::downgrade(&event_handler);
        *self.event_handler.borrow_mut() = Some(event_handler);

        let task_source = self
            .owner_global()
            .task_manager()
            .media_element_task_source()
            .to_sendable();
        ROUTER.add_typed_route(
            action_receiver,
            Box::new(move |message| {
                let event = message.unwrap();
                let weak_event_handler = weak_event_handler.clone();

                task_source.queue(task!(handle_player_event: move || {
                    trace!("HTMLMediaElement event: {event:?}");

                    let Some(event_handler) = weak_event_handler.upgrade() else {
                        return;
                    };

                    event_handler.lock().unwrap().handle_player_event(player_id, event, CanGc::note());
                }));
            }),
        );

        let task_source = self
            .owner_global()
            .task_manager()
            .media_element_task_source()
            .to_sendable();
        let weak_video_renderer = Arc::downgrade(&self.video_renderer);

        self.video_renderer
            .lock()
            .unwrap()
            .setup(player_id, task_source, weak_video_renderer);

        Ok(())
    }

    fn reset_media_player(&self) {
        if self.player.borrow().is_none() {
            return;
        }

        if let Some(ref player) = *self.player.borrow() {
            if let Err(error) = player.lock().unwrap().stop() {
                error!("Could not stop player: {error:?}");
            }
        }

        *self.player.borrow_mut() = None;
        self.video_renderer.lock().unwrap().reset();
        *self.event_handler.borrow_mut() = None;

        if let Some(video_element) = self.downcast::<HTMLVideoElement>() {
            video_element.set_natural_dimensions(None, None);
        }
    }

    pub(crate) fn set_audio_track(&self, idx: usize, enabled: bool) {
        if let Some(ref player) = *self.player.borrow() {
            if let Err(error) = player.lock().unwrap().set_audio_track(idx as i32, enabled) {
                warn!("Could not set audio track {error:?}");
            }
        }
    }

    pub(crate) fn set_video_track(&self, idx: usize, enabled: bool) {
        if let Some(ref player) = *self.player.borrow() {
            if let Err(error) = player.lock().unwrap().set_video_track(idx as i32, enabled) {
                warn!("Could not set video track: {error:?}");
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#direction-of-playback>
    fn direction_of_playback(&self) -> PlaybackDirection {
        // If the element's playbackRate is positive or zero, then the direction of playback is
        // forwards. Otherwise, it is backwards.
        if self.playback_rate.get() >= 0. {
            PlaybackDirection::Forwards
        } else {
            PlaybackDirection::Backwards
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#ended-playback>
    fn ended_playback(&self, loop_condition: LoopCondition) -> bool {
        // A media element is said to have ended playback when:

        // The element's readyState attribute is HAVE_METADATA or greater, and
        if self.ready_state.get() < ReadyState::HaveMetadata {
            return false;
        }

        let playback_position = self.current_playback_position.get();

        match self.direction_of_playback() {
            // Either: The current playback position is the end of the media resource, and the
            // direction of playback is forwards, and the media element does not have a loop
            // attribute specified.
            PlaybackDirection::Forwards => {
                playback_position >= self.Duration() &&
                    (loop_condition == LoopCondition::Ignored || !self.Loop())
            },
            // Or: The current playback position is the earliest possible position, and the
            // direction of playback is backwards.
            PlaybackDirection::Backwards => playback_position <= self.earliest_possible_position(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#reaches-the-end>
    fn end_of_playback_in_forwards_direction(&self) {
        // When the current playback position reaches the end of the media resource when the
        // direction of playback is forwards, then the user agent must follow these steps:

        // Step 1. If the media element has a loop attribute specified, then seek to the earliest
        // posible position of the media resource and return.
        if self.Loop() {
            self.seek(
                self.earliest_possible_position(),
                /* approximate_for_speed */ false,
            );
            return;
        }

        // Step 2. As defined above, the ended IDL attribute starts returning true once the event
        // loop returns to step 1.

        // Step 3. Queue a media element task given the media element and the following steps:
        let this = Trusted::new(self);
        let generation_id = self.generation_id.get();

        self.owner_global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(reaches_the_end_steps: move || {
                let this = this.root();
                if generation_id != this.generation_id.get() {
                    return;
                }

                // Step 3.1. Fire an event named timeupdate at the media element.
                this.upcast::<EventTarget>().fire_event(atom!("timeupdate"), CanGc::note());

                // Step 3.2. If the media element has ended playback, the direction of playback is
                // forwards, and paused is false, then:
                if this.ended_playback(LoopCondition::Included) &&
                    this.direction_of_playback() == PlaybackDirection::Forwards &&
                    !this.Paused() {
                    // Step 3.2.1. Set the paused attribute to true.
                    this.paused.set(true);

                    // Step 3.2.2. Fire an event named pause at the media element.
                    this.upcast::<EventTarget>().fire_event(atom!("pause"), CanGc::note());

                    // Step 3.2.3. Take pending play promises and reject pending play promises with
                    // the result and an "AbortError" DOMException.
                    this.take_pending_play_promises(Err(Error::Abort(None)));
                    this.fulfill_in_flight_play_promises(|| ());
                }

                // Step 3.3. Fire an event named ended at the media element.
                this.upcast::<EventTarget>().fire_event(atom!("ended"), CanGc::note());
            }));

        // <https://html.spec.whatwg.org/multipage/#dom-media-have_current_data>
        self.change_ready_state(ReadyState::HaveCurrentData);
    }

    /// <https://html.spec.whatwg.org/multipage/#reaches-the-end>
    fn end_of_playback_in_backwards_direction(&self) {
        // When the current playback position reaches the earliest possible position of the media
        // resource when the direction of playback is backwards, then the user agent must only queue
        // a media element task given the media element to fire an event named timeupdate at the
        // element.
        if self.current_playback_position.get() <= self.earliest_possible_position() {
            self.queue_media_element_task_to_fire_event(atom!("timeupdate"));
        }
    }

    fn playback_end(&self) {
        // Abort the following steps of the end of playback if seeking is in progress.
        if self.seeking.get() {
            return;
        }

        match self.direction_of_playback() {
            PlaybackDirection::Forwards => self.end_of_playback_in_forwards_direction(),
            PlaybackDirection::Backwards => self.end_of_playback_in_backwards_direction(),
        }
    }

    fn playback_error(&self, error: &str, can_gc: CanGc) {
        error!("Player error: {:?}", error);

        // If we have already flagged an error condition while processing
        // the network response, we should silently skip any observable
        // errors originating while decoding the erroneous response.
        if self.in_error_state() {
            return;
        }

        // <https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list>
        if self.ready_state.get() == ReadyState::HaveNothing {
            // => "If the media data can be fetched but is found by inspection to be in an
            // unsupported format, or can otherwise not be rendered at all"
            self.media_data_processing_failure_steps();
        } else {
            // => "If the media data is corrupted"
            self.media_data_processing_fatal_steps(MEDIA_ERR_DECODE, can_gc);
        }
    }

    fn playback_metadata_updated(
        &self,
        metadata: &servo_media::player::metadata::Metadata,
        can_gc: CanGc,
    ) {
        // The following steps should be run once on the initial `metadata` signal from the media
        // engine.
        if self.ready_state.get() != ReadyState::HaveNothing {
            return;
        }

        // https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
        // => "If the media resource is found to have an audio track"
        for (i, _track) in metadata.audio_tracks.iter().enumerate() {
            let audio_track_list = self.AudioTracks(can_gc);

            // Step 1. Create an AudioTrack object to represent the audio track.
            let kind = match i {
                0 => DOMString::from("main"),
                _ => DOMString::new(),
            };

            let audio_track = AudioTrack::new(
                self.global().as_window(),
                DOMString::new(),
                kind,
                DOMString::new(),
                DOMString::new(),
                Some(&*audio_track_list),
                can_gc,
            );

            // Steps 2. Update the media element's audioTracks attribute's AudioTrackList object
            // with the new AudioTrack object.
            audio_track_list.add(&audio_track);

            // Step 3. Let enable be unknown.
            // Step 4. If either the media resource or the URL of the current media resource
            // indicate a particular set of audio tracks to enable, or if the user agent has
            // information that would facilitate the selection of specific audio tracks to
            // improve the user's experience, then: if this audio track is one of the ones to
            // enable, then set enable to true, otherwise, set enable to false.
            if let Some(servo_url) = self.resource_url.borrow().as_ref() {
                let fragment = MediaFragmentParser::from(servo_url);
                if let Some(id) = fragment.id() {
                    if audio_track.id() == id {
                        audio_track_list.set_enabled(audio_track_list.len() - 1, true);
                    }
                }

                if fragment.tracks().contains(&audio_track.kind().into()) {
                    audio_track_list.set_enabled(audio_track_list.len() - 1, true);
                }
            }

            // Step 5. If enable is still unknown, then, if the media element does not yet have an
            // enabled audio track, then set enable to true, otherwise, set enable to false.
            // Step 6. If enable is true, then enable this audio track, otherwise, do not enable
            // this audio track.
            if audio_track_list.enabled_index().is_none() {
                audio_track_list.set_enabled(audio_track_list.len() - 1, true);
            }

            // Step 7. Fire an event named addtrack at this AudioTrackList object, using TrackEvent,
            // with the track attribute initialized to the new AudioTrack object.
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
                .fire(audio_track_list.upcast::<EventTarget>(), can_gc);
        }

        // => "If the media resource is found to have a video track"
        for (i, _track) in metadata.video_tracks.iter().enumerate() {
            let video_track_list = self.VideoTracks(can_gc);

            // Step 1. Create a VideoTrack object to represent the video track.
            let kind = match i {
                0 => DOMString::from("main"),
                _ => DOMString::new(),
            };

            let video_track = VideoTrack::new(
                self.global().as_window(),
                DOMString::new(),
                kind,
                DOMString::new(),
                DOMString::new(),
                Some(&*video_track_list),
                can_gc,
            );

            // Steps 2. Update the media element's videoTracks attribute's VideoTrackList object
            // with the new VideoTrack object.
            video_track_list.add(&video_track);

            // Step 3. Let enable be unknown.
            // Step 4. If either the media resource or the URL of the current media resource
            // indicate a particular set of video tracks to enable, or if the user agent has
            // information that would facilitate the selection of specific video tracks to
            // improve the user's experience, then: if this video track is the first such video
            // track, then set enable to true, otherwise, set enable to false.
            if let Some(track) = video_track_list.item(0) {
                if let Some(servo_url) = self.resource_url.borrow().as_ref() {
                    let fragment = MediaFragmentParser::from(servo_url);
                    if let Some(id) = fragment.id() {
                        if track.id() == id {
                            video_track_list.set_selected(0, true);
                        }
                    } else if fragment.tracks().contains(&track.kind().into()) {
                        video_track_list.set_selected(0, true);
                    }
                }
            }

            // Step 5. If enable is still unknown, then, if the media element does not yet have a
            // selected video track, then set enable to true, otherwise, set enable to false.
            // Step 6. If enable is true, then select this track and unselect any previously
            // selected video tracks, otherwise, do not select this video track. If other tracks are
            // unselected, then a change event will be fired.
            if video_track_list.selected_index().is_none() {
                video_track_list.set_selected(video_track_list.len() - 1, true);
            }

            // Step 7. Fire an event named addtrack at this VideoTrackList object, using TrackEvent,
            // with the track attribute initialized to the new VideoTrack object.
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
                .fire(video_track_list.upcast::<EventTarget>(), can_gc);
        }

        // => "Once enough of the media data has been fetched to determine the duration..."

        // TODO Step 1. Establish the media timeline for the purposes of the current playback
        // position and the earliest possible position, based on the media data.

        // TODO Step 2. Update the timeline offset to the date and time that corresponds to the zero
        // time in the media timeline established in the previous step, if any. If no explicit time
        // and date is given by the media resource, the timeline offset must be set to Not-a-Number
        // (NaN).

        // Step 3. Set the current playback position and the official playback position to the
        // earliest possible position.
        let earliest_possible_position = self.earliest_possible_position();
        self.current_playback_position
            .set(earliest_possible_position);
        self.official_playback_position
            .set(earliest_possible_position);

        // Step 4. Update the duration attribute with the time of the last frame of the resource, if
        // known, on the media timeline established above. If it is not known (e.g. a stream that is
        // in principle infinite), update the duration attribute to the value positive Infinity.
        // Note: The user agent will queue a media element task given the media element to fire an
        // event named durationchange at the element at this point.
        self.duration.set(
            metadata
                .duration
                .map_or(f64::INFINITY, |duration| duration.as_secs_f64()),
        );
        self.queue_media_element_task_to_fire_event(atom!("durationchange"));

        // Step 5. For video elements, set the videoWidth and videoHeight attributes, and queue a
        // media element task given the media element to fire an event named resize at the media
        // element.
        if let Some(video_element) = self.downcast::<HTMLVideoElement>() {
            video_element.set_natural_dimensions(Some(metadata.width), Some(metadata.height));
            self.queue_media_element_task_to_fire_event(atom!("resize"));
        }

        // Step 6. Set the readyState attribute to HAVE_METADATA.
        self.change_ready_state(ReadyState::HaveMetadata);

        // Step 7. Let jumped be false.
        let mut jumped = false;

        // Step 8. If the media element's default playback start position is greater than zero, then
        // seek to that time, and let jumped be true.
        if self.default_playback_start_position.get() > 0. {
            self.seek(
                self.default_playback_start_position.get(),
                /* approximate_for_speed */ false,
            );
            jumped = true;
        }

        // Step 9. Set the media element's default playback start position to zero.
        self.default_playback_start_position.set(0.);

        // Step 10. Let the initial playback position be 0.
        // Step 11. If either the media resource or the URL of the current media resource indicate a
        // particular start time, then set the initial playback position to that time and, if jumped
        // is still false, seek to that time.
        if let Some(servo_url) = self.resource_url.borrow().as_ref() {
            let fragment = MediaFragmentParser::from(servo_url);
            if let Some(initial_playback_position) = fragment.start() {
                if initial_playback_position > 0. &&
                    initial_playback_position < self.duration.get() &&
                    !jumped
                {
                    self.seek(
                        initial_playback_position,
                        /* approximate_for_speed */ false,
                    )
                }
            }
        }

        // Step 12. If there is no enabled audio track, then enable an audio track. This will cause
        // a change event to be fired.
        // Step 13. If there is no selected video track, then select a video track. This will cause
        // a change event to be fired.
        // Note that these steps are already handled by the earlier media track processing.

        let global = self.global();
        let window = global.as_window();

        // Update the media session metadata title with the obtained metadata.
        window.Navigator().MediaSession().update_title(
            metadata
                .title
                .clone()
                .unwrap_or(window.get_url().into_string()),
        );
    }

    fn playback_duration_changed(&self, duration: Option<Duration>) {
        let duration = duration.map_or(f64::INFINITY, |duration| duration.as_secs_f64());

        if self.duration.get() == duration {
            return;
        }

        self.duration.set(duration);

        // When the length of the media resource changes to a known value (e.g. from being unknown
        // to known, or from a previously established length to a new length), the user agent must
        // queue a media element task given the media element to fire an event named durationchange
        // at the media element.
        // <https://html.spec.whatwg.org/multipage/#offsets-into-the-media-resource:media-resource-22>
        self.queue_media_element_task_to_fire_event(atom!("durationchange"));

        // If the duration is changed such that the current playback position ends up being greater
        // than the time of the end of the media resource, then the user agent must also seek to the
        // time of the end of the media resource.
        if self.current_playback_position.get() > duration {
            self.seek(duration, /* approximate_for_speed */ false);
        }
    }

    fn playback_video_frame_updated(&self) {
        let Some(video_element) = self.downcast::<HTMLVideoElement>() else {
            return;
        };

        // Whenever the natural width or natural height of the video changes (including, for
        // example, because the selected video track was changed), if the element's readyState
        // attribute is not HAVE_NOTHING, the user agent must queue a media element task given
        // the media element to fire an event named resize at the media element.
        // <https://html.spec.whatwg.org/multipage/#concept-video-intrinsic-width>

        // The event for the prerolled frame from media engine could reached us before the media
        // element HAVE_METADATA ready state so subsequent steps will be cancelled.
        if self.ready_state.get() == ReadyState::HaveNothing {
            return;
        }

        if let Some(frame) = self.video_renderer.lock().unwrap().current_frame {
            if video_element
                .set_natural_dimensions(Some(frame.width as u32), Some(frame.height as u32))
            {
                self.queue_media_element_task_to_fire_event(atom!("resize"));
            } else {
                // If the natural dimensions have not been changed, the node should be marked as
                // damaged to force a repaint with the new frame contents.
                self.upcast::<Node>().dirty(NodeDamage::Other);
            }
        }
    }

    fn playback_need_data(&self) {
        // The media engine signals that the source needs more data. If we already have a valid
        // fetch request, we do nothing. Otherwise, if we have no request and the previous request
        // was cancelled because we got an EnoughData event, we restart fetching where we left.
        if let Some(ref current_fetch_context) = *self.current_fetch_context.borrow() {
            if let Some(reason) = current_fetch_context.cancel_reason() {
                // XXX(ferjm) Ideally we should just create a fetch request from
                // where we left. But keeping track of the exact next byte that the
                // media backend expects is not the easiest task, so I'm simply
                // seeking to the current playback position for now which will create
                // a new fetch request for the last rendered frame.
                if *reason == CancelReason::Backoff {
                    self.seek(
                        self.current_playback_position.get(),
                        /* approximate_for_speed */ false,
                    );
                }
                return;
            }
        }

        if let Some(ref mut current_fetch_context) = *self.current_fetch_context.borrow_mut() {
            if let Err(e) = {
                let mut data_source = current_fetch_context.data_source().borrow_mut();
                data_source.set_locked(false);
                data_source.process_into_player_from_queue(self.player.borrow().as_ref().unwrap())
            } {
                // If we are pushing too much data and we know that we can
                // restart the download later from where we left, we cancel
                // the current request. Otherwise, we continue the request
                // assuming that we may drop some frames.
                if e == PlayerError::EnoughData {
                    current_fetch_context.cancel(CancelReason::Backoff);
                }
            }
        }
    }

    fn playback_enough_data(&self) {
        // The media engine signals that the source has enough data and asks us to stop pushing bytes
        // to avoid excessive buffer queueing, so we cancel the ongoing fetch request if we are able
        // to restart it from where we left. Otherwise, we continue the current fetch request,
        // assuming that some frames will be dropped.
        if let Some(ref mut current_fetch_context) = *self.current_fetch_context.borrow_mut() {
            if current_fetch_context.is_seekable() {
                current_fetch_context.cancel(CancelReason::Backoff);
            }
        }
    }

    fn playback_position_changed(&self, position: f64) {
        // Abort the following steps of the current time update if seeking is in progress.
        if self.seeking.get() {
            return;
        }

        let _ = self
            .played
            .borrow_mut()
            .add(self.current_playback_position.get(), position);
        self.current_playback_position.set(position);
        self.official_playback_position.set(position);
        self.time_marches_on();

        let media_position_state =
            MediaPositionState::new(self.duration.get(), self.playback_rate.get(), position);
        debug!(
            "Sending media session event set position state {:?}",
            media_position_state
        );
        self.send_media_session_event(MediaSessionEvent::SetPositionState(media_position_state));
    }

    fn playback_seek_done(&self, position: f64) {
        // If the seek was initiated by script or by the user agent itself continue with the
        // following steps, otherwise abort.
        if !self.seeking.get() || position != self.current_seek_position.get() {
            return;
        }

        // <https://html.spec.whatwg.org/multipage/#dom-media-seek>
        // Step 13. Await a stable state.
        let task = MediaElementMicrotask::Seeked {
            elem: DomRoot::from_ref(self),
            generation_id: self.generation_id.get(),
        };

        ScriptThread::await_stable_state(Microtask::MediaElement(task));
    }

    fn playback_state_changed(&self, state: &PlaybackState) {
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
                if self.ready_state.get() == ReadyState::HaveMetadata {
                    self.change_ready_state(ReadyState::HaveEnoughData);
                }
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
    }

    fn seekable(&self) -> TimeRangesContainer {
        let mut seekable = TimeRangesContainer::default();
        if let Some(ref player) = *self.player.borrow() {
            let ranges = player.lock().unwrap().seekable();
            for range in ranges {
                let _ = seekable.add(range.start, range.end);
            }
        }
        seekable
    }

    /// <https://html.spec.whatwg.org/multipage/#earliest-possible-position>
    fn earliest_possible_position(&self) -> f64 {
        self.seekable()
            .start(0)
            .unwrap_or_else(|_| self.current_playback_position.get())
    }

    fn render_controls(&self, can_gc: CanGc) {
        if self.upcast::<Element>().is_shadow_host() {
            // Bail out if we are already showing the controls.
            return;
        }

        // FIXME(stevennovaryo): Recheck styling of media element to avoid
        //                       reparsing styles.
        let shadow_root = self
            .upcast::<Element>()
            .attach_ua_shadow_root(false, can_gc);
        let document = self.owner_document();
        let script = Element::create(
            QualName::new(None, ns!(html), local_name!("script")),
            None,
            &document,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
            can_gc,
        );
        // This is our hacky way to temporarily workaround the lack of a privileged
        // JS context.
        // The media controls UI accesses the document.servoGetMediaControls(id) API
        // to get an instance to the media controls ShadowRoot.
        // `id` needs to match the internally generated UUID assigned to a media element.
        let id = Uuid::new_v4().to_string();
        document.register_media_controls(&id, &shadow_root);
        let media_controls_script = MEDIA_CONTROL_JS.replace("@@@id@@@", &id);
        *self.media_controls_id.borrow_mut() = Some(id);
        script
            .upcast::<Node>()
            .set_text_content_for_element(Some(DOMString::from(media_controls_script)), can_gc);
        if let Err(e) = shadow_root
            .upcast::<Node>()
            .AppendChild(script.upcast::<Node>(), can_gc)
        {
            warn!("Could not render media controls {:?}", e);
            return;
        }

        let style = Element::create(
            QualName::new(None, ns!(html), local_name!("style")),
            None,
            &document,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
            can_gc,
        );

        style
            .upcast::<Node>()
            .set_text_content_for_element(Some(DOMString::from(MEDIA_CONTROL_CSS)), can_gc);

        if let Err(e) = shadow_root
            .upcast::<Node>()
            .AppendChild(style.upcast::<Node>(), can_gc)
        {
            warn!("Could not render media controls {:?}", e);
        }

        self.upcast::<Node>().dirty(NodeDamage::Other);
    }

    fn remove_controls(&self) {
        if let Some(id) = self.media_controls_id.borrow_mut().take() {
            self.owner_document().unregister_media_controls(&id);
        }
    }

    /// Gets the video frame at the current playback position.
    pub(crate) fn get_current_frame(&self) -> Option<VideoFrame> {
        self.video_renderer
            .lock()
            .unwrap()
            .current_frame_holder
            .as_ref()
            .map(|holder| holder.get_frame())
    }

    /// Gets the current frame of the video element to present, if any.
    /// <https://html.spec.whatwg.org/multipage/#the-video-element:the-video-element-7>
    pub(crate) fn get_current_frame_to_present(&self) -> Option<MediaFrame> {
        let (current_frame, poster_frame) = {
            let renderer = self.video_renderer.lock().unwrap();
            (renderer.current_frame, renderer.poster_frame)
        };

        // If the show poster flag is set (or there is no current video frame to
        // present) AND there is a poster frame, present that.
        if (self.show_poster.get() || current_frame.is_none()) && poster_frame.is_some() {
            return poster_frame;
        }

        current_frame
    }

    /// By default the audio is rendered through the audio sink automatically
    /// selected by the servo-media Player instance. However, in some cases, like
    /// the WebAudio MediaElementAudioSourceNode, we need to set a custom audio
    /// renderer.
    pub(crate) fn set_audio_renderer(
        &self,
        audio_renderer: Option<Arc<Mutex<dyn AudioRenderer>>>,
        can_gc: CanGc,
    ) {
        *self.audio_renderer.borrow_mut() = audio_renderer;

        let had_player = {
            if let Some(ref player) = *self.player.borrow() {
                if let Err(error) = player.lock().unwrap().stop() {
                    error!("Could not stop player: {error:?}");
                }
                true
            } else {
                false
            }
        };

        if had_player {
            self.media_element_load_algorithm(can_gc);
        }
    }

    fn send_media_session_event(&self, event: MediaSessionEvent) {
        let global = self.global();
        let media_session = global.as_window().Navigator().MediaSession();

        media_session.register_media_instance(self);

        media_session.send_event(event);
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-media-load-resource>
    pub(crate) fn origin_is_clean(&self) -> bool {
        // Step 5.local (media provider object).
        if self.src_object.borrow().is_some() {
            // The resource described by the current media resource, if any,
            // contains the media data. It is CORS-same-origin.
            return true;
        }

        // Step 5.remote (URL record).
        if self.resource_url.borrow().is_some() {
            // Update the media data with the contents
            // of response's unsafe response obtained in this fashion.
            // Response can be CORS-same-origin or CORS-cross-origin;
            if let Some(ref current_fetch_context) = *self.current_fetch_context.borrow() {
                return current_fetch_context.origin_is_clean();
            }
        }

        true
    }
}

impl HTMLMediaElementMethods<crate::DomTypeHolder> for HTMLMediaElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-media-networkstate>
    fn NetworkState(&self) -> u16 {
        self.network_state.get() as u16
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-readystate>
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

    /// <https://html.spec.whatwg.org/multipage/#dom-media-crossOrigin>
    fn GetCrossOrigin(&self) -> Option<DOMString> {
        reflect_cross_origin_attribute(self.upcast::<Element>())
    }
    /// <https://html.spec.whatwg.org/multipage/#dom-media-crossOrigin>
    fn SetCrossOrigin(&self, value: Option<DOMString>, can_gc: CanGc) {
        set_cross_origin_attribute(self.upcast::<Element>(), value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-muted>
    fn Muted(&self) -> bool {
        self.muted.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-muted>
    fn SetMuted(&self, value: bool) {
        if self.muted.get() == value {
            return;
        }

        self.muted.set(value);

        if let Some(ref player) = *self.player.borrow() {
            if let Err(error) = player.lock().unwrap().set_mute(value) {
                warn!("Could not set mute state: {error:?}");
            }
        }

        // The user agent must queue a media element task given the media element to fire an event
        // named volumechange at the media element.
        self.queue_media_element_task_to_fire_event(atom!("volumechange"));

        // Then, if the media element is not allowed to play, the user agent must run the internal
        // pause steps for the media element.
        if !self.is_allowed_to_play() {
            self.internal_pause_steps();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-srcobject>
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

    /// <https://html.spec.whatwg.org/multipage/#dom-media-srcobject>
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

    /// <https://html.spec.whatwg.org/multipage/#dom-media-currentsrc>
    fn CurrentSrc(&self) -> USVString {
        USVString(self.current_src.borrow().clone())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-load>
    fn Load(&self, can_gc: CanGc) {
        self.media_element_load_algorithm(can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-canplaytype>
    fn CanPlayType(&self, type_: DOMString) -> CanPlayTypeResult {
        match ServoMedia::get().can_play_type(&type_.str()) {
            SupportsMediaType::No => CanPlayTypeResult::_empty,
            SupportsMediaType::Maybe => CanPlayTypeResult::Maybe,
            SupportsMediaType::Probably => CanPlayTypeResult::Probably,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-error>
    fn GetError(&self) -> Option<DomRoot<MediaError>> {
        self.error.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-play>
    fn Play(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp, can_gc);

        // TODO Step 1. If the media element is not allowed to play, then return a promise rejected
        // with a "NotAllowedError" DOMException.

        // Step 2. If the media element's error attribute is not null and its code is
        // MEDIA_ERR_SRC_NOT_SUPPORTED, then return a promise rejected with a "NotSupportedError"
        // DOMException.
        if self
            .error
            .get()
            .is_some_and(|e| e.Code() == MEDIA_ERR_SRC_NOT_SUPPORTED)
        {
            promise.reject_error(Error::NotSupported(None), can_gc);
            return promise;
        }

        // Step 3. Let promise be a new promise and append promise to the list of pending play
        // promises.
        self.push_pending_play_promise(&promise);

        // Step 4. Run the internal play steps for the media element.
        self.internal_play_steps(can_gc);

        // Step 5. Return promise.
        promise
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-pause>
    fn Pause(&self, can_gc: CanGc) {
        // Step 1. If the media element's networkState attribute has the value NETWORK_EMPTY, invoke
        // the media element's resource selection algorithm.
        if self.network_state.get() == NetworkState::Empty {
            self.invoke_resource_selection_algorithm(can_gc);
        }

        // Step 2. Run the internal pause steps for the media element.
        self.internal_pause_steps();
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-paused>
    fn Paused(&self) -> bool {
        self.paused.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-defaultplaybackrate>
    fn GetDefaultPlaybackRate(&self) -> Fallible<Finite<f64>> {
        Ok(Finite::wrap(self.default_playback_rate.get()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-defaultplaybackrate>
    fn SetDefaultPlaybackRate(&self, value: Finite<f64>) -> ErrorResult {
        // If the given value is not supported by the user agent, then throw a "NotSupportedError"
        // DOMException.
        let min_allowed = -64.0;
        let max_allowed = 64.0;
        if *value < min_allowed || *value > max_allowed {
            return Err(Error::NotSupported(None));
        }

        if self.default_playback_rate.get() == *value {
            return Ok(());
        }

        self.default_playback_rate.set(*value);

        // The user agent must queue a media element task given the media element to fire an event
        // named ratechange at the media element.
        self.queue_media_element_task_to_fire_event(atom!("ratechange"));

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-playbackrate>
    fn GetPlaybackRate(&self) -> Fallible<Finite<f64>> {
        Ok(Finite::wrap(self.playback_rate.get()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-playbackrate>
    fn SetPlaybackRate(&self, value: Finite<f64>) -> ErrorResult {
        // The attribute is mutable: on setting, the user agent must follow these steps:

        // Step 1. If the given value is not supported by the user agent, then throw a
        // "NotSupportedError" DOMException.
        let min_allowed = -64.0;
        let max_allowed = 64.0;
        if *value < min_allowed || *value > max_allowed {
            return Err(Error::NotSupported(None));
        }

        if self.playback_rate.get() == *value {
            return Ok(());
        }

        // Step 2. Set playbackRate to the new value, and if the element is potentially playing,
        // change the playback speed.
        self.playback_rate.set(*value);

        if self.is_potentially_playing() {
            if let Some(ref player) = *self.player.borrow() {
                if let Err(error) = player.lock().unwrap().set_playback_rate(*value) {
                    warn!("Could not set the playback rate: {error:?}");
                }
            }
        }

        // The user agent must queue a media element task given the media element to fire an event
        // named ratechange at the media element.
        self.queue_media_element_task_to_fire_event(atom!("ratechange"));

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-duration>
    fn Duration(&self) -> f64 {
        self.duration.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-currenttime>
    fn CurrentTime(&self) -> Finite<f64> {
        Finite::wrap(if self.default_playback_start_position.get() != 0. {
            self.default_playback_start_position.get()
        } else if self.seeking.get() {
            // Note that the other browsers do the similar (by checking `seeking` value or clamp the
            // `official` position to the earliest possible position, the duration, and the seekable
            // ranges.
            // <https://github.com/whatwg/html/issues/11773>
            self.current_seek_position.get()
        } else {
            self.official_playback_position.get()
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-currenttime>
    fn SetCurrentTime(&self, time: Finite<f64>) {
        if self.ready_state.get() == ReadyState::HaveNothing {
            self.default_playback_start_position.set(*time);
        } else {
            self.official_playback_position.set(*time);
            self.seek(*time, /* approximate_for_speed */ false);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-seeking>
    fn Seeking(&self) -> bool {
        self.seeking.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-ended>
    fn Ended(&self) -> bool {
        self.ended_playback(LoopCondition::Included) &&
            self.direction_of_playback() == PlaybackDirection::Forwards
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-fastseek>
    fn FastSeek(&self, time: Finite<f64>) {
        self.seek(*time, /* approximate_for_speed */ true);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-played>
    fn Played(&self, can_gc: CanGc) -> DomRoot<TimeRanges> {
        TimeRanges::new(
            self.global().as_window(),
            self.played.borrow().clone(),
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-seekable>
    fn Seekable(&self, can_gc: CanGc) -> DomRoot<TimeRanges> {
        TimeRanges::new(self.global().as_window(), self.seekable(), can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-buffered>
    fn Buffered(&self, can_gc: CanGc) -> DomRoot<TimeRanges> {
        let mut buffered = TimeRangesContainer::default();
        if let Some(ref player) = *self.player.borrow() {
            let ranges = player.lock().unwrap().buffered();
            for range in ranges {
                let _ = buffered.add(range.start, range.end);
            }
        }
        TimeRanges::new(self.global().as_window(), buffered, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-audiotracks>
    fn AudioTracks(&self, can_gc: CanGc) -> DomRoot<AudioTrackList> {
        let window = self.owner_window();
        self.audio_tracks_list
            .or_init(|| AudioTrackList::new(&window, &[], Some(self), can_gc))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-videotracks>
    fn VideoTracks(&self, can_gc: CanGc) -> DomRoot<VideoTrackList> {
        let window = self.owner_window();
        self.video_tracks_list
            .or_init(|| VideoTrackList::new(&window, &[], Some(self), can_gc))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-texttracks>
    fn TextTracks(&self, can_gc: CanGc) -> DomRoot<TextTrackList> {
        let window = self.owner_window();
        self.text_tracks_list
            .or_init(|| TextTrackList::new(&window, &[], can_gc))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-addtexttrack>
    fn AddTextTrack(
        &self,
        kind: TextTrackKind,
        label: DOMString,
        language: DOMString,
        can_gc: CanGc,
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
            can_gc,
        );
        // Step 3 & 4
        self.TextTracks(can_gc).add(&track);
        // Step 5
        DomRoot::from_ref(&track)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-volume>
    fn GetVolume(&self) -> Fallible<Finite<f64>> {
        Ok(Finite::wrap(self.volume.get()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-volume>
    fn SetVolume(&self, value: Finite<f64>) -> ErrorResult {
        // If the new value is outside the range 0.0 to 1.0 inclusive, then, on setting, an
        // "IndexSizeError" DOMException must be thrown instead.
        let minimum_volume = 0.0;
        let maximum_volume = 1.0;
        if *value < minimum_volume || *value > maximum_volume {
            return Err(Error::IndexSize(None));
        }

        if self.volume.get() == *value {
            return Ok(());
        }

        self.volume.set(*value);

        if let Some(ref player) = *self.player.borrow() {
            if let Err(error) = player.lock().unwrap().set_volume(*value) {
                warn!("Could not set the volume: {error:?}");
            }
        }

        // The user agent must queue a media element task given the media element to fire an event
        // named volumechange at the media element.
        self.queue_media_element_task_to_fire_event(atom!("volumechange"));

        // Then, if the media element is not allowed to play, the user agent must run the internal
        // pause steps for the media element.
        if !self.is_allowed_to_play() {
            self.internal_pause_steps();
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
                // <https://html.spec.whatwg.org/multipage/#dom-media-muted>
                // When a media element is created, if the element has a muted content attribute
                // specified, then the muted IDL attribute should be set to true.
                if let AttributeMutation::Set(
                    _,
                    AttributeMutationReason::ByCloning | AttributeMutationReason::ByParser,
                ) = mutation
                {
                    self.SetMuted(true);
                }
            },
            local_name!("src") => {
                // <https://html.spec.whatwg.org/multipage/#location-of-the-media-resource>
                // If a src attribute of a media element is set or changed, the user agent must invoke
                // the media element's media element load algorithm (Removing the src attribute does
                // not do this, even if there are source elements present).
                if !mutation.is_removal() {
                    self.media_element_load_algorithm(can_gc);
                }
            },
            local_name!("controls") => {
                if mutation.new_value(attr).is_some() {
                    self.render_controls(can_gc);
                } else {
                    self.remove_controls();
                }
            },
            _ => (),
        };
    }

    /// <https://html.spec.whatwg.org/multipage/#playing-the-media-resource:remove-an-element-from-a-document>
    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        self.remove_controls();

        if context.tree_connected {
            let task = MediaElementMicrotask::PauseIfNotInDocument {
                elem: DomRoot::from_ref(self),
            };
            ScriptThread::await_stable_state(Microtask::MediaElement(task));
        }
    }

    fn adopting_steps(&self, old_doc: &Document, can_gc: CanGc) {
        self.super_type().unwrap().adopting_steps(old_doc, can_gc);

        // Note that media control id should be adopting between documents so "privileged"
        // document.servoGetMediaControls(id) API is keeping access to the whitelist of media
        // controls identifiers.
        if let Some(id) = &*self.media_controls_id.borrow() {
            let Some(shadow_root) = self.upcast::<Element>().shadow_root() else {
                error!("Missing media controls shadow root");
                return;
            };

            old_doc.unregister_media_controls(id);
            self.owner_document()
                .register_media_controls(id, &shadow_root);
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
    SelectNextSourceChild {
        elem: DomRoot<HTMLMediaElement>,
        generation_id: u32,
    },
    SelectNextSourceChildAfterWait {
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
            &MediaElementMicrotask::SelectNextSourceChild {
                ref elem,
                generation_id,
            } => {
                if generation_id == elem.generation_id.get() {
                    elem.select_next_source_child(can_gc);
                }
            },
            &MediaElementMicrotask::SelectNextSourceChildAfterWait {
                ref elem,
                generation_id,
            } => {
                if generation_id == elem.generation_id.get() {
                    elem.select_next_source_child_after_wait(can_gc);
                }
            },
        }
    }

    fn enter_realm(&self) -> JSAutoRealm {
        match self {
            &MediaElementMicrotask::ResourceSelection { ref elem, .. } |
            &MediaElementMicrotask::PauseIfNotInDocument { ref elem } |
            &MediaElementMicrotask::Seeked { ref elem, .. } |
            &MediaElementMicrotask::SelectNextSourceChild { ref elem, .. } |
            &MediaElementMicrotask::SelectNextSourceChildAfterWait { ref elem, .. } => {
                enter_realm(&**elem)
            },
        }
    }
}

enum Resource {
    Object,
    Url(ServoUrl),
}

#[derive(Debug, MallocSizeOf, PartialEq)]
enum DataBuffer {
    Payload(Vec<u8>),
    EndOfStream,
}

#[derive(MallocSizeOf)]
struct BufferedDataSource {
    /// During initial setup and seeking (including clearing the buffer queue
    /// and resetting the end-of-stream state), the data source should be locked and
    /// any request for processing should be ignored until the media player informs us
    /// via the NeedData event that it is ready to accept incoming data.
    locked: Cell<bool>,
    /// Temporary storage for incoming data.
    buffers: VecDeque<DataBuffer>,
}

impl BufferedDataSource {
    fn new() -> BufferedDataSource {
        BufferedDataSource {
            locked: Cell::new(true),
            buffers: VecDeque::default(),
        }
    }

    fn set_locked(&self, locked: bool) {
        self.locked.set(locked)
    }

    fn add_buffer_to_queue(&mut self, buffer: DataBuffer) {
        debug_assert_ne!(
            self.buffers.back(),
            Some(&DataBuffer::EndOfStream),
            "The media backend not expects any further data after end of stream"
        );

        self.buffers.push_back(buffer);
    }

    fn process_into_player_from_queue(
        &mut self,
        player: &Arc<Mutex<dyn Player>>,
    ) -> Result<(), PlayerError> {
        // Early out if any request for processing should be ignored.
        if self.locked.get() {
            return Ok(());
        }

        while let Some(buffer) = self.buffers.pop_front() {
            match buffer {
                DataBuffer::Payload(payload) => {
                    if let Err(error) = player.lock().unwrap().push_data(payload) {
                        warn!("Could not push input data to player: {error:?}");
                        return Err(error);
                    }
                },
                DataBuffer::EndOfStream => {
                    if let Err(error) = player.lock().unwrap().end_of_stream() {
                        warn!("Could not signal EOS to player: {error:?}");
                        return Err(error);
                    }
                },
            }
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.locked.set(true);
        self.buffers.clear();
    }
}

/// Indicates the reason why a fetch request was cancelled.
#[derive(Debug, MallocSizeOf, PartialEq)]
enum CancelReason {
    /// We were asked to stop pushing data to the player.
    Backoff,
    /// An error ocurred while fetching the media data.
    Error,
    /// The fetching process is aborted by the user.
    Abort,
}

#[derive(MallocSizeOf)]
pub(crate) struct HTMLMediaElementFetchContext {
    /// The fetch request id.
    request_id: RequestId,
    /// Some if the request has been cancelled.
    cancel_reason: Option<CancelReason>,
    /// Indicates whether the fetched stream is seekable.
    is_seekable: bool,
    /// Indicates whether the fetched stream is origin clean.
    origin_clean: bool,
    /// The buffered data source which to be processed by media backend.
    data_source: RefCell<BufferedDataSource>,
    /// Fetch canceller. Allows cancelling the current fetch request by
    /// manually calling its .cancel() method or automatically on Drop.
    fetch_canceller: FetchCanceller,
}

impl HTMLMediaElementFetchContext {
    fn new(
        request_id: RequestId,
        core_resource_thread: CoreResourceThread,
    ) -> HTMLMediaElementFetchContext {
        HTMLMediaElementFetchContext {
            request_id,
            cancel_reason: None,
            is_seekable: false,
            origin_clean: true,
            data_source: RefCell::new(BufferedDataSource::new()),
            fetch_canceller: FetchCanceller::new(request_id, core_resource_thread.clone()),
        }
    }

    fn request_id(&self) -> RequestId {
        self.request_id
    }

    fn is_seekable(&self) -> bool {
        self.is_seekable
    }

    fn set_seekable(&mut self, seekable: bool) {
        self.is_seekable = seekable;
    }

    fn origin_is_clean(&self) -> bool {
        self.origin_clean
    }

    fn set_origin_clean(&mut self, origin_clean: bool) {
        self.origin_clean = origin_clean;
    }

    fn data_source(&self) -> &RefCell<BufferedDataSource> {
        &self.data_source
    }

    fn cancel(&mut self, reason: CancelReason) {
        if self.cancel_reason.is_some() {
            return;
        }
        self.cancel_reason = Some(reason);
        self.data_source.borrow_mut().reset();
        self.fetch_canceller.cancel();
    }

    fn cancel_reason(&self) -> &Option<CancelReason> {
        &self.cancel_reason
    }
}

struct HTMLMediaElementFetchListener {
    /// The element that initiated the request.
    element: Trusted<HTMLMediaElement>,
    /// The generation of the media element when this fetch started.
    generation_id: u32,
    /// The fetch request id.
    request_id: RequestId,
    /// Time of last progress notification.
    next_progress_event: Instant,
    /// Url for the resource.
    url: ServoUrl,
    /// Expected content length of the media asset being fetched or played.
    expected_content_length: Option<u64>,
    /// Actual content length of the media asset was fetched.
    fetched_content_length: u64,
    /// Discarded content length from the network for the ongoing
    /// request if range requests are not supported. Seek requests set it
    /// to the required position (in bytes).
    content_length_to_discard: u64,
}

impl FetchResponseListener for HTMLMediaElementFetchListener {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(&mut self, _: RequestId, metadata: Result<FetchMetadata, NetworkError>) {
        let element = self.element.root();

        let (metadata, origin_clean) = match metadata {
            Ok(fetch_metadata) => match fetch_metadata {
                FetchMetadata::Unfiltered(metadata) => (Some(metadata), true),
                FetchMetadata::Filtered { filtered, unsafe_ } => (
                    Some(unsafe_),
                    matches!(
                        filtered,
                        FilteredMetadata::Basic(_) | FilteredMetadata::Cors(_)
                    ),
                ),
            },
            Err(_) => (None, true),
        };

        let (status_is_success, is_seekable) =
            metadata.as_ref().map_or((false, false), |metadata| {
                let status = &metadata.status;
                (status.is_success(), *status == StatusCode::PARTIAL_CONTENT)
            });

        // <https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list>
        if !status_is_success {
            if element.ready_state.get() == ReadyState::HaveNothing {
                // => "If the media data cannot be fetched at all, due to network errors..."
                element.media_data_processing_failure_steps();
            } else {
                // => "If the connection is interrupted after some media data has been received..."
                element.media_data_processing_fatal_steps(MEDIA_ERR_NETWORK, CanGc::note());
            }
            return;
        }

        if let Some(ref mut current_fetch_context) = *element.current_fetch_context.borrow_mut() {
            current_fetch_context.set_seekable(is_seekable);
            current_fetch_context.set_origin_clean(origin_clean);
        }

        if let Some(metadata) = metadata.as_ref() {
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
                        self.expected_content_length = Some(content_length);
                    }
                }
            }
        }

        // Explicit media player initialization with live/seekable source.
        if let Some(expected_content_length) = self.expected_content_length {
            if let Err(e) = element
                .player
                .borrow()
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .set_input_size(expected_content_length)
            {
                warn!("Could not set player input size {:?}", e);
            }
        }
    }

    fn process_response_chunk(&mut self, _: RequestId, chunk: Vec<u8>) {
        let element = self.element.root();

        self.fetched_content_length += chunk.len() as u64;

        // If an error was received previously, we skip processing the payload.
        if let Some(ref mut current_fetch_context) = *element.current_fetch_context.borrow_mut() {
            if let Some(CancelReason::Backoff) = current_fetch_context.cancel_reason() {
                return;
            }

            // Discard chunk of the response body if fetch context doesn't support range requests.
            let payload = if !current_fetch_context.is_seekable() &&
                self.content_length_to_discard != 0
            {
                if chunk.len() as u64 > self.content_length_to_discard {
                    let shrink_chunk = chunk[self.content_length_to_discard as usize..].to_vec();
                    self.content_length_to_discard = 0;
                    shrink_chunk
                } else {
                    // Completely discard this response chunk.
                    self.content_length_to_discard -= chunk.len() as u64;
                    return;
                }
            } else {
                chunk
            };

            if let Err(e) = {
                let mut data_source = current_fetch_context.data_source().borrow_mut();
                data_source.add_buffer_to_queue(DataBuffer::Payload(payload));
                data_source
                    .process_into_player_from_queue(element.player.borrow().as_ref().unwrap())
            } {
                // If we are pushing too much data and we know that we can
                // restart the download later from where we left, we cancel
                // the current request. Otherwise, we continue the request
                // assuming that we may drop some frames.
                if e == PlayerError::EnoughData {
                    current_fetch_context.cancel(CancelReason::Backoff);
                }
                return;
            }
        }

        // <https://html.spec.whatwg.org/multipage/#concept-media-load-resource>
        // While the load is not suspended (see below), every 350ms (±200ms) or for every byte
        // received, whichever is least frequent, queue a media element task given the media element
        // to fire an event named progress at the element.
        if Instant::now() > self.next_progress_event {
            element.queue_media_element_task_to_fire_event(atom!("progress"));
            self.next_progress_event = Instant::now() + Duration::from_millis(350);
        }
    }

    fn process_response_eof(self, _: RequestId, status: Result<ResourceFetchTiming, NetworkError>) {
        let element = self.element.root();

        // <https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list>
        if status.is_ok() && self.fetched_content_length != 0 {
            // => "Once the entire media resource has been fetched..."

            // There are no more chunks of the response body forthcoming, so we can
            // go ahead and notify the media backend not to expect any further data.
            if let Some(ref mut current_fetch_context) = *element.current_fetch_context.borrow_mut()
            {
                // On initial state change READY -> PAUSED the media player perform
                // seek to initial position by event with seek segment (TIME format)
                // while media stack operates in BYTES format and configuring segment
                // start and stop positions without the total size of the stream is not
                // possible. As fallback the media player perform seek with BYTES format
                // and initiate seek request via "seek-data" callback with required offset.
                if self.expected_content_length.is_none() {
                    if let Err(e) = element
                        .player
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .set_input_size(self.fetched_content_length)
                    {
                        warn!("Could not set player input size {:?}", e);
                    }
                }

                let mut data_source = current_fetch_context.data_source().borrow_mut();

                data_source.add_buffer_to_queue(DataBuffer::EndOfStream);
                let _ = data_source
                    .process_into_player_from_queue(element.player.borrow().as_ref().unwrap());
            }

            // Step 1. Fire an event named progress at the media element.
            element
                .upcast::<EventTarget>()
                .fire_event(atom!("progress"), CanGc::note());

            // Step 2. Set the networkState to NETWORK_IDLE and fire an event named suspend at the
            // media element.
            element.network_state.set(NetworkState::Idle);

            element
                .upcast::<EventTarget>()
                .fire_event(atom!("suspend"), CanGc::note());
        } else if status.is_err() && element.ready_state.get() != ReadyState::HaveNothing {
            // => "If the connection is interrupted after some media data has been received..."
            element.media_data_processing_fatal_steps(MEDIA_ERR_NETWORK, CanGc::note());
        } else {
            // => "If the media data can be fetched but is found by inspection to be in an
            // unsupported format, or can otherwise not be rendered at all"
            element.media_data_processing_failure_steps();
        }

        if let Ok(response) = status {
            network_listener::submit_timing(&self, &response, CanGc::note());
        }
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None, None);
    }

    fn should_invoke(&self) -> bool {
        let element = self.element.root();

        if element.generation_id.get() != self.generation_id || element.player.borrow().is_none() {
            return false;
        }

        let Some(ref current_fetch_context) = *element.current_fetch_context.borrow() else {
            return false;
        };

        // Whether the new fetch request was triggered.
        if current_fetch_context.request_id() != self.request_id {
            return false;
        }

        // Whether the current fetch request was cancelled due to a network or decoding error, or
        // was aborted by the user.
        if let Some(cancel_reason) = current_fetch_context.cancel_reason() {
            if matches!(*cancel_reason, CancelReason::Error | CancelReason::Abort) {
                return false;
            }
        }

        true
    }
}

impl ResourceTimingListener for HTMLMediaElementFetchListener {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        let initiator_type = InitiatorType::LocalName(
            self.element
                .root()
                .upcast::<Element>()
                .local_name()
                .to_string(),
        );
        (initiator_type, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.element.root().owner_document().global()
    }
}

impl HTMLMediaElementFetchListener {
    fn new(element: &HTMLMediaElement, request_id: RequestId, url: ServoUrl, offset: u64) -> Self {
        Self {
            element: Trusted::new(element),
            generation_id: element.generation_id.get(),
            request_id,
            next_progress_event: Instant::now() + Duration::from_millis(350),
            url,
            expected_content_length: None,
            fetched_content_length: 0,
            content_length_to_discard: offset,
        }
    }
}

/// The [`HTMLMediaElementEventHandler`] is a structure responsible for handling media events for
/// the [`HTMLMediaElement`] and exists to decouple ownership of the [`HTMLMediaElement`] from IPC
/// router callback.
#[derive(JSTraceable, MallocSizeOf)]
struct HTMLMediaElementEventHandler {
    element: WeakRef<HTMLMediaElement>,
}

#[expect(unsafe_code)]
unsafe impl Send for HTMLMediaElementEventHandler {}

impl HTMLMediaElementEventHandler {
    fn new(element: &HTMLMediaElement) -> Self {
        Self {
            element: WeakRef::new(element),
        }
    }

    fn handle_player_event(&self, player_id: usize, event: PlayerEvent, can_gc: CanGc) {
        let Some(element) = self.element.root() else {
            return;
        };

        // Abort event processing if the associated media player is outdated.
        if element.player_id().is_none_or(|id| id != player_id) {
            return;
        }

        match event {
            PlayerEvent::DurationChanged(duration) => element.playback_duration_changed(duration),
            PlayerEvent::EndOfStream => element.playback_end(),
            PlayerEvent::EnoughData => element.playback_enough_data(),
            PlayerEvent::Error(ref error) => element.playback_error(error, can_gc),
            PlayerEvent::MetadataUpdated(ref metadata) => {
                element.playback_metadata_updated(metadata, can_gc)
            },
            PlayerEvent::NeedData => element.playback_need_data(),
            PlayerEvent::PositionChanged(position) => element.playback_position_changed(position),
            PlayerEvent::SeekData(offset, seek_lock) => {
                element.fetch_request(Some(offset), Some(seek_lock))
            },
            PlayerEvent::SeekDone(position) => element.playback_seek_done(position),
            PlayerEvent::StateChanged(ref state) => element.playback_state_changed(state),
            PlayerEvent::VideoFrameUpdated => element.playback_video_frame_updated(),
        }
    }
}

impl Drop for HTMLMediaElementEventHandler {
    fn drop(&mut self) {
        // The weak reference to the media element is not thread-safe and MUST be deleted on the
        // script thread, which is guaranteed by ownership of the `event handler` in the IPC router
        // callback (queued task to the media element task source) and the media element itself.
        assert_in_script();
    }
}
