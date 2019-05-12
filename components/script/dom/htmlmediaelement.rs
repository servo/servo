/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::{AlreadyInCompartment, InCompartment};
use crate::document_loader::{LoadBlocker, LoadType};
use crate::dom::attr::Attr;
use crate::dom::audiotrack::AudioTrack;
use crate::dom::audiotracklist::AudioTrackList;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::HTMLMediaElementBinding::CanPlayTypeResult;
use crate::dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementConstants;
use crate::dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLSourceElementBinding::HTMLSourceElementMethods;
use crate::dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorConstants::*;
use crate::dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorMethods;
use crate::dom::bindings::codegen::Bindings::TextTrackBinding::{TextTrackKind, TextTrackMode};
use crate::dom::bindings::codegen::InheritTypes::{ElementTypeId, HTMLElementTypeId};
use crate::dom::bindings::codegen::InheritTypes::{HTMLMediaElementTypeId, NodeTypeId};
use crate::dom::bindings::codegen::UnionTypes::{
    MediaStreamOrBlob, VideoTrackOrAudioTrackOrTextTrack,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::blob::Blob;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlsourceelement::HTMLSourceElement;
use crate::dom::htmlvideoelement::HTMLVideoElement;
use crate::dom::mediaerror::MediaError;
use crate::dom::mediastream::MediaStream;
use crate::dom::node::{document_from_node, window_from_node, Node, NodeDamage, UnbindContext};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::texttrack::TextTrack;
use crate::dom::texttracklist::TextTrackList;
use crate::dom::timeranges::{TimeRanges, TimeRangesContainer};
use crate::dom::trackevent::TrackEvent;
use crate::dom::url::URL;
use crate::dom::videotrack::VideoTrack;
use crate::dom::videotracklist::VideoTrackList;
use crate::dom::virtualmethods::VirtualMethods;
use crate::fetch::FetchCanceller;
use crate::microtask::{Microtask, MicrotaskRunnable};
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use crate::script_thread::ScriptThread;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;
use headers_core::HeaderMapExt;
use headers_ext::{ContentLength, ContentRange};
use html5ever::{LocalName, Prefix};
use http::header::{self, HeaderMap, HeaderValue};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::image::base::Image;
use net_traits::image_cache::ImageResponse;
use net_traits::request::{CredentialsMode, Destination, Referrer, RequestBuilder};
use net_traits::{CoreResourceMsg, FetchChannels, FetchMetadata, FetchResponseListener, Metadata};
use net_traits::{NetworkError, ResourceFetchTiming, ResourceTimingType};
use script_layout_interface::HTMLMediaData;
use servo_config::pref;
use servo_media::player::context::{GlContext, NativeDisplay, PlayerGLContext};
use servo_media::player::frame::{Frame, FrameRenderer};
use servo_media::player::{PlaybackState, Player, PlayerError, PlayerEvent, StreamType};
use servo_media::{ServoMedia, SupportsMediaType};
use servo_url::ServoUrl;
use std::cell::Cell;
use std::collections::VecDeque;
use std::f64;
use std::mem;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use time::{self, Duration, Timespec};
use webrender_api::{ImageData, ImageDescriptor, ImageFormat, ImageKey, RenderApi};
use webrender_api::{RenderApiSender, Transaction};

pub struct MediaFrameRenderer {
    api: RenderApi,
    current_frame: Option<(ImageKey, i32, i32)>,
    old_frame: Option<ImageKey>,
    very_old_frame: Option<ImageKey>,
}

impl MediaFrameRenderer {
    fn new(render_api_sender: RenderApiSender) -> Self {
        Self {
            api: render_api_sender.create_api(),
            current_frame: None,
            old_frame: None,
            very_old_frame: None,
        }
    }

    fn render_poster_frame(&mut self, image: Arc<Image>) {
        if let Some(image_id) = image.id {
            self.current_frame = Some((image_id, image.width as i32, image.height as i32));
        }
    }
}

impl FrameRenderer for MediaFrameRenderer {
    fn render(&mut self, frame: Frame) {
        let descriptor = ImageDescriptor::new(
            frame.get_width(),
            frame.get_height(),
            ImageFormat::BGRA8,
            false,
            false,
        );

        let mut txn = Transaction::new();

        let image_data = ImageData::Raw(frame.get_data());

        if let Some(old_image_key) = mem::replace(&mut self.very_old_frame, self.old_frame.take()) {
            txn.delete_image(old_image_key);
        }

        match self.current_frame {
            Some((ref image_key, ref mut width, ref mut height))
                if *width == frame.get_width() && *height == frame.get_height() =>
            {
                txn.update_image(
                    *image_key,
                    descriptor,
                    image_data,
                    &webrender_api::DirtyRect::All,
                );

                if let Some(old_image_key) = self.old_frame.take() {
                    txn.delete_image(old_image_key);
                }
            }
            Some((ref mut image_key, ref mut width, ref mut height)) => {
                self.old_frame = Some(*image_key);

                let new_image_key = self.api.generate_image_key();
                txn.add_image(new_image_key, descriptor, image_data, None);
                *image_key = new_image_key;
                *width = frame.get_width();
                *height = frame.get_height();
            },
            None => {
                let image_key = self.api.generate_image_key();
                txn.add_image(image_key, descriptor, image_data, None);
                self.current_frame = Some((image_key, frame.get_width(), frame.get_height()));
            },
        }
        self.api.update_resources(txn.resource_updates);
    }
}

struct PlayerContextDummy();
impl PlayerGLContext for PlayerContextDummy {
    fn get_gl_context(&self) -> GlContext {
        return GlContext::Unknown;
    }
    fn get_native_display(&self) -> NativeDisplay {
        return NativeDisplay::Unknown;
    }
}

#[must_root]
#[derive(JSTraceable, MallocSizeOf)]
enum SrcObject {
    MediaStream(Dom<MediaStream>),
    Blob(Dom<Blob>),
}

impl From<MediaStreamOrBlob> for SrcObject {
    #[allow(unrooted_must_root)]
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
pub struct HTMLMediaElement {
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
    #[ignore_malloc_size_of = "promises are hard"]
    in_flight_play_promises_queue: DomRefCell<VecDeque<(Box<[Rc<Promise>]>, ErrorResult)>>,
    #[ignore_malloc_size_of = "servo_media"]
    player: DomRefCell<Option<Box<Player>>>,
    #[ignore_malloc_size_of = "Arc"]
    frame_renderer: Arc<Mutex<MediaFrameRenderer>>,
    /// https://html.spec.whatwg.org/multipage/#show-poster-flag
    show_poster: Cell<bool>,
    /// https://html.spec.whatwg.org/multipage/#dom-media-duration
    duration: Cell<f64>,
    /// https://html.spec.whatwg.org/multipage/#official-playback-position
    playback_position: Cell<f64>,
    /// https://html.spec.whatwg.org/multipage/#default-playback-start-position
    default_playback_start_position: Cell<f64>,
    /// https://html.spec.whatwg.org/multipage/#dom-media-volume
    volume: Cell<f64>,
    /// https://html.spec.whatwg.org/multipage/#dom-media-seeking
    seeking: Cell<bool>,
    /// https://html.spec.whatwg.org/multipage/#dom-media-muted
    muted: Cell<bool>,
    /// URL of the media resource, if any.
    resource_url: DomRefCell<Option<ServoUrl>>,
    /// URL of the media resource, if the resource is set through the src_object attribute and it
    /// is a blob.
    blob_url: DomRefCell<Option<ServoUrl>>,
    /// https://html.spec.whatwg.org/multipage/#dom-media-played
    #[ignore_malloc_size_of = "Rc"]
    played: DomRefCell<TimeRangesContainer>,
    // https://html.spec.whatwg.org/multipage/#dom-media-audiotracks
    audio_tracks_list: MutNullableDom<AudioTrackList>,
    // https://html.spec.whatwg.org/multipage/#dom-media-videotracks
    video_tracks_list: MutNullableDom<VideoTrackList>,
    /// https://html.spec.whatwg.org/multipage/#dom-media-texttracks
    text_tracks_list: MutNullableDom<TextTrackList>,
    /// Time of last timeupdate notification.
    #[ignore_malloc_size_of = "Defined in time"]
    next_timeupdate_event: Cell<Timespec>,
    /// Latest fetch request context.
    current_fetch_context: DomRefCell<Option<HTMLMediaElementFetchContext>>,
}

/// <https://html.spec.whatwg.org/multipage/#dom-media-networkstate>
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
#[repr(u8)]
pub enum NetworkState {
    Empty = HTMLMediaElementConstants::NETWORK_EMPTY as u8,
    Idle = HTMLMediaElementConstants::NETWORK_IDLE as u8,
    Loading = HTMLMediaElementConstants::NETWORK_LOADING as u8,
    NoSource = HTMLMediaElementConstants::NETWORK_NO_SOURCE as u8,
}

/// <https://html.spec.whatwg.org/multipage/#dom-media-readystate>
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ReadyState {
    HaveNothing = HTMLMediaElementConstants::HAVE_NOTHING as u8,
    HaveMetadata = HTMLMediaElementConstants::HAVE_METADATA as u8,
    HaveCurrentData = HTMLMediaElementConstants::HAVE_CURRENT_DATA as u8,
    HaveFutureData = HTMLMediaElementConstants::HAVE_FUTURE_DATA as u8,
    HaveEnoughData = HTMLMediaElementConstants::HAVE_ENOUGH_DATA as u8,
}

impl HTMLMediaElement {
    pub fn new_inherited(tag_name: LocalName, prefix: Option<Prefix>, document: &Document) -> Self {
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
            frame_renderer: Arc::new(Mutex::new(MediaFrameRenderer::new(
                document.window().get_webrender_api_sender(),
            ))),
            show_poster: Cell::new(true),
            duration: Cell::new(f64::NAN),
            playback_position: Cell::new(0.),
            default_playback_start_position: Cell::new(0.),
            volume: Cell::new(1.0),
            seeking: Cell::new(false),
            resource_url: DomRefCell::new(None),
            blob_url: DomRefCell::new(None),
            played: DomRefCell::new(TimeRangesContainer::new()),
            audio_tracks_list: Default::default(),
            video_tracks_list: Default::default(),
            text_tracks_list: Default::default(),
            next_timeupdate_event: Cell::new(time::get_time() + Duration::milliseconds(250)),
            current_fetch_context: DomRefCell::new(None),
        }
    }

    pub fn get_ready_state(&self) -> ReadyState {
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
            if let Err(e) = player.set_rate(self.playbackRate.get()) {
                warn!("Could not set the playback rate {:?}", e);
            }
            if let Err(e) = player.play() {
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
    pub fn delay_load_event(&self, delay: bool) {
        let mut blocker = self.delaying_the_load_event_flag.borrow_mut();
        if delay && blocker.is_none() {
            *blocker = Some(LoadBlocker::new(&document_from_node(self), LoadType::Media));
        } else if !delay && blocker.is_some() {
            LoadBlocker::terminate(&mut *blocker);
        }
    }

    /// https://html.spec.whatwg.org/multipage/#time-marches-on
    fn time_marches_on(&self) {
        // Step 6.
        if time::get_time() > self.next_timeupdate_event.get() {
            let window = window_from_node(self);
            window
                .task_manager()
                .media_element_task_source()
                .queue_simple_event(self.upcast(), atom!("timeupdate"), &window);
            self.next_timeupdate_event
                .set(time::get_time() + Duration::milliseconds(350));
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
            let window = window_from_node(self);
            let this = Trusted::new(self);
            let generation_id = self.generation_id.get();
            let _ = window.task_manager().media_element_task_source().queue(
                task!(internal_pause_steps: move || {
                    let this = this.root();
                    if generation_id != this.generation_id.get() {
                        return;
                    }

                    this.fulfill_in_flight_play_promises(|| {
                        // Step 2.3.1.
                        this.upcast::<EventTarget>().fire_event(atom!("timeupdate"));

                        // Step 2.3.2.
                        this.upcast::<EventTarget>().fire_event(atom!("pause"));

                        if let Some(ref player) = *this.player.borrow() {
                            if let Err(e) = player.pause() {
                                eprintln!("Could not pause player {:?}", e);
                            }
                        }

                        // Step 2.3.3.
                        // Done after running this closure in
                        // `fulfill_in_flight_play_promises`.
                    });
                }),
                window.upcast(),
            );

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
        let window = window_from_node(self);
        let this = Trusted::new(self);
        let generation_id = self.generation_id.get();
        // FIXME(nox): Why are errors silenced here?
        let _ = window.task_manager().media_element_task_source().queue(
            task!(notify_about_playing: move || {
                let this = this.root();
                if generation_id != this.generation_id.get() {
                    return;
                }

                this.fulfill_in_flight_play_promises(|| {
                    // Step 2.1.
                    this.upcast::<EventTarget>().fire_event(atom!("playing"));
                    this.play_media();

                    // Step 2.2.
                    // Done after running this closure in
                    // `fulfill_in_flight_play_promises`.
                });

            }),
            window.upcast(),
        );
    }

    // https://html.spec.whatwg.org/multipage/#ready-states
    fn change_ready_state(&self, ready_state: ReadyState) {
        let old_ready_state = self.ready_state.get();
        self.ready_state.set(ready_state);

        if self.network_state.get() == NetworkState::Empty {
            return;
        }

        let window = window_from_node(self);
        let task_source = window.task_manager().media_element_task_source();

        // Step 1.
        match (old_ready_state, ready_state) {
            (ReadyState::HaveNothing, ReadyState::HaveMetadata) => {
                task_source.queue_simple_event(self.upcast(), atom!("loadedmetadata"), &window);

                // No other steps are applicable in this case.
                return;
            },
            (ReadyState::HaveMetadata, new) if new >= ReadyState::HaveCurrentData => {
                if !self.fired_loadeddata_event.get() {
                    self.fired_loadeddata_event.set(true);
                    let this = Trusted::new(self);
                    // FIXME(nox): Why are errors silenced here?
                    let _ = task_source.queue(
                        task!(media_reached_current_data: move || {
                            let this = this.root();
                            this.upcast::<EventTarget>().fire_event(atom!("loadeddata"));
                            this.delay_load_event(false);
                        }),
                        window.upcast(),
                    );
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
            task_source.queue_simple_event(self.upcast(), atom!("canplay"), &window);

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
                    self.show_poster.set(false);
                    self.time_marches_on();
                }
                // Step 3
                task_source.queue_simple_event(self.upcast(), atom!("play"), &window);
                // Step 4
                self.notify_about_playing();
                // Step 5
                self.autoplaying.set(false);
            }

            // FIXME(nox): According to the spec, this should come *before* the
            // "play" event.
            task_source.queue_simple_event(self.upcast(), atom!("canplaythrough"), &window);
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn invoke_resource_selection_algorithm(&self) {
        // Step 1.
        self.network_state.set(NetworkState::NoSource);

        // Step 2.
        self.show_poster.set(true);

        // Step 3.
        self.delay_load_event(true);

        // Step 4.
        // If the resource selection mode in the synchronous section is
        // "attribute", the URL of the resource to fetch is relative to the
        // media element's node document when the src attribute was last
        // changed, which is why we need to pass the base URL in the task
        // right here.
        let doc = document_from_node(self);
        let task = MediaElementMicrotask::ResourceSelectionTask {
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
    fn resource_selection_algorithm_sync(&self, base_url: ServoUrl) {
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
            self.delay_load_event(false);
            return;
        };

        // Step 7.
        self.network_state.set(NetworkState::Loading);

        // Step 8.
        let window = window_from_node(self);
        window
            .task_manager()
            .media_element_task_source()
            .queue_simple_event(self.upcast(), atom!("loadstart"), &window);

        // Step 9.
        match mode {
            // Step 9.obj.
            Mode::Object => {
                // Step 9.obj.1.
                *self.current_src.borrow_mut() = "".to_owned();

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
                    source.upcast::<EventTarget>().fire_event(atom!("error"));
                    self.queue_dedicated_media_source_failure_steps();
                    return;
                }
                // Step 9.attr.3.
                let url_record = match base_url.join(&src) {
                    Ok(url) => url,
                    Err(_) => {
                        source.upcast::<EventTarget>().fire_event(atom!("error"));
                        self.queue_dedicated_media_source_failure_steps();
                        return;
                    },
                };
                // Step 9.attr.8.
                self.resource_fetch_algorithm(Resource::Url(url_record));
            },
        }
    }

    fn fetch_request(&self, offset: Option<u64>) {
        if self.resource_url.borrow().is_none() && self.blob_url.borrow().is_none() {
            eprintln!("Missing request url");
            self.queue_dedicated_media_source_failure_steps();
            return;
        }

        // FIXME(nox): Handle CORS setting from crossorigin attribute.
        let document = document_from_node(self);
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

        let request = RequestBuilder::new(url.clone())
            .headers(headers)
            .destination(destination)
            .credentials_mode(CredentialsMode::Include)
            .use_url_credentials(true)
            .origin(document.origin().immutable().clone())
            .pipeline_id(Some(self.global().pipeline_id()))
            .referrer(Some(Referrer::ReferrerUrl(document.url())))
            .referrer_policy(document.get_referrer_policy());

        let mut current_fetch_context = self.current_fetch_context.borrow_mut();
        if let Some(ref mut current_fetch_context) = *current_fetch_context {
            current_fetch_context.cancel(CancelReason::Overridden);
        }
        let (fetch_context, cancel_receiver) = HTMLMediaElementFetchContext::new();
        *current_fetch_context = Some(fetch_context);
        let fetch_listener = Arc::new(Mutex::new(HTMLMediaElementFetchListener::new(
            self,
            url.clone(),
            offset.unwrap_or(0),
        )));
        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let window = window_from_node(self);
        let (task_source, canceller) = window
            .task_manager()
            .networking_task_source_with_canceller();
        let network_listener = NetworkListener {
            context: fetch_listener,
            task_source,
            canceller: Some(canceller),
        };
        ROUTER.add_route(
            action_receiver.to_opaque(),
            Box::new(move |message| {
                network_listener.notify_fetch(message.to().unwrap());
            }),
        );
        let global = self.global();
        global
            .core_resource_thread()
            .send(CoreResourceMsg::Fetch(
                request,
                FetchChannels::ResponseMsg(action_sender, Some(cancel_receiver)),
            ))
            .unwrap();
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
                    let window = window_from_node(self);
                    window
                        .task_manager()
                        .media_element_task_source()
                        .queue_simple_event(self.upcast(), atom!("suspend"), &window);

                    // Step 4.remote.1.3.
                    let this = Trusted::new(self);
                    window
                        .task_manager()
                        .media_element_task_source()
                        .queue(
                            task!(set_media_delay_load_event_flag_to_false: move || {
                                this.root().delay_load_event(false);
                            }),
                            window.upcast(),
                        )
                        .unwrap();

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
                self.fetch_request(None);
            },
            Resource::Object => {
                if let Some(ref src_object) = *self.src_object.borrow() {
                    match src_object {
                        SrcObject::Blob(blob) => {
                            let blob_url = URL::CreateObjectURL(&self.global(), &*blob);
                            *self.blob_url.borrow_mut() =
                                Some(ServoUrl::parse(&blob_url).expect("infallible"));
                            self.fetch_request(None);
                        },
                        SrcObject::MediaStream(ref stream) => {
                            for stream in &*stream.get_tracks() {
                                if let Err(_) = self
                                    .player
                                    .borrow()
                                    .as_ref()
                                    .unwrap()
                                    .set_stream(&stream.id())
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
        let window = window_from_node(self);
        let this = Trusted::new(self);
        let generation_id = self.generation_id.get();
        self.take_pending_play_promises(Err(Error::NotSupported));
        // FIXME(nox): Why are errors silenced here?
        let _ = window.task_manager().media_element_task_source().queue(
            task!(dedicated_media_source_failure_steps: move || {
                let this = this.root();
                if generation_id != this.generation_id.get() {
                    return;
                }

                this.fulfill_in_flight_play_promises(|| {
                    // Step 1.
                    this.error.set(Some(&*MediaError::new(
                        &window_from_node(&*this),
                        MEDIA_ERR_SRC_NOT_SUPPORTED,
                    )));

                    // Step 2.
                    this.AudioTracks().clear();
                    this.VideoTracks().clear();

                    // Step 3.
                    this.network_state.set(NetworkState::NoSource);

                    // Step 4.
                    this.show_poster.set(true);

                    // Step 5.
                    this.upcast::<EventTarget>().fire_event(atom!("error"));

                    if let Some(ref player) = *this.player.borrow() {
                        if let Err(e) = player.stop() {
                            eprintln!("Could not stop player {:?}", e);
                        }
                    }

                    // Step 6.
                    // Done after running this closure in
                    // `fulfill_in_flight_play_promises`.
                });

                // Step 7.
                this.delay_load_event(false);
            }),
            window.upcast(),
        );
    }

    fn queue_ratechange_event(&self) {
        let window = window_from_node(self);
        let task_source = window.task_manager().media_element_task_source();
        task_source.queue_simple_event(self.upcast(), atom!("ratechange"), &window);
    }

    // https://html.spec.whatwg.org/multipage/#potentially-playing
    fn is_potentially_playing(&self) -> bool {
        !self.paused.get() &&
        // FIXME: We need https://github.com/servo/servo/pull/22348
        //              to know whether playback has ended or not
        // !self.Ended() &&
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
    fn media_element_load_algorithm(&self) {
        // Reset the flag that signals whether loadeddata was ever fired for
        // this invokation of the load algorithm.
        self.fired_loadeddata_event.set(false);

        // Step 1-2.
        self.generation_id.set(self.generation_id.get() + 1);

        // Steps 3-4.
        while !self.in_flight_play_promises_queue.borrow().is_empty() {
            self.fulfill_in_flight_play_promises(|| ());
        }

        let window = window_from_node(self);
        let task_source = window.task_manager().media_element_task_source();

        // Step 5.
        let network_state = self.network_state.get();
        if network_state == NetworkState::Loading || network_state == NetworkState::Idle {
            task_source.queue_simple_event(self.upcast(), atom!("abort"), &window);
        }

        // Step 6.
        if network_state != NetworkState::Empty {
            // Step 6.1.
            task_source.queue_simple_event(self.upcast(), atom!("emptied"), &window);

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
                task_source.queue_simple_event(self.upcast(), atom!("timeupdate"), &window);
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
        self.invoke_resource_selection_algorithm();

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
        let pending_play_promises =
            mem::replace(&mut *self.pending_play_promises.borrow_mut(), vec![]);
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
    /// `#[allow(unrooted_must_root)]` on even more functions, potentially
    /// hiding actual safety bugs.
    #[allow(unrooted_must_root)]
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
                Ok(ref value) => promise.resolve_native(value),
                Err(ref error) => promise.reject_error(error.clone()),
            }
        }
    }

    /// Handles insertion of `source` children.
    ///
    /// <https://html.spec.whatwg.org/multipage/#the-source-element:nodes-are-inserted>
    pub fn handle_source_child_insertion(&self) {
        if self.upcast::<Element>().has_attribute(&local_name!("src")) {
            return;
        }
        if self.network_state.get() != NetworkState::Empty {
            return;
        }
        self.media_element_load_algorithm();
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-seek
    fn seek(&self, time: f64, _approximate_for_speed: bool) {
        // Step 1.
        self.show_poster.set(false);

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
        let window = window_from_node(self);
        let task_source = window.task_manager().media_element_task_source();
        task_source.queue_simple_event(self.upcast(), atom!("seeking"), &window);

        // Step 11.
        if let Some(ref player) = *self.player.borrow() {
            if let Err(e) = player.seek(time) {
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
        let window = window_from_node(self);
        let task_source = window.task_manager().media_element_task_source();
        task_source.queue_simple_event(self.upcast(), atom!("timeupdate"), &window);

        // Step 17.
        task_source.queue_simple_event(self.upcast(), atom!("seeked"), &window);
    }

    /// https://html.spec.whatwg.org/multipage/#poster-frame
    pub fn process_poster_response(&self, image: ImageResponse) {
        if !self.show_poster.get() {
            return;
        }

        // Step 6.
        if let ImageResponse::Loaded(image, _) = image {
            self.frame_renderer
                .lock()
                .unwrap()
                .render_poster_frame(image);
            self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
            if pref!(media.testing.enabled) {
                let window = window_from_node(self);
                let task_source = window.task_manager().media_element_task_source();
                task_source.queue_simple_event(self.upcast(), atom!("postershown"), &window);
            } else {
                return;
            }
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

        let player = ServoMedia::get()
            .unwrap()
            .create_player(stream_type, Box::new(PlayerContextDummy()));

        let (action_sender, action_receiver) = ipc::channel().unwrap();
        player.register_event_handler(action_sender);
        player.register_frame_renderer(self.frame_renderer.clone());

        *self.player.borrow_mut() = Some(player);

        let trusted_node = Trusted::new(self);
        let window = window_from_node(self);
        let (task_source, canceller) = window
            .task_manager()
            .media_element_task_source_with_canceller();
        ROUTER.add_route(
            action_receiver.to_opaque(),
            Box::new(move |message| {
                let event: PlayerEvent = message.to().unwrap();
                trace!("Player event {:?}", event);
                let this = trusted_node.clone();
                if let Err(err) = task_source.queue_with_canceller(
                    task!(handle_player_event: move || {
                        this.root().handle_player_event(&event);
                    }),
                    &canceller,
                ) {
                    warn!("Could not queue player event handler task {:?}", err);
                }
            }),
        );

        Ok(())
    }

    fn handle_player_event(&self, event: &PlayerEvent) {
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
                                let window = window_from_node(self);
                                let this = Trusted::new(self);

                                let _ = window.task_manager().media_element_task_source().queue(
                                    task!(reaches_the_end_steps: move || {
                                        let this = this.root();
                                        // Step 3.1.
                                        this.upcast::<EventTarget>().fire_event(atom!("timeupdate"));

                                        // Step 3.2.
                                        if this.Ended() && !this.Paused() {
                                            // Step 3.2.1.
                                            this.paused.set(true);

                                            // Step 3.2.2.
                                            this.upcast::<EventTarget>().fire_event(atom!("pause"));

                                            // Step 3.2.3.
                                            this.take_pending_play_promises(Err(Error::Abort));
                                            this.fulfill_in_flight_play_promises(|| ());
                                        }

                                        // Step 3.3.
                                        this.upcast::<EventTarget>().fire_event(atom!("ended"));
                                    }),
                                    window.upcast(),
                                );
                            }
                        },

                        PlaybackDirection::Backwards => {
                            if self.playback_position.get() <= self.earliest_possible_position() {
                                let window = window_from_node(self);

                                window
                                    .task_manager()
                                    .media_element_task_source()
                                    .queue_simple_event(self.upcast(), atom!("ended"), &window);
                            }
                        },
                    }
                }
            },
            PlayerEvent::Error(ref error) => {
                error!("Player error: {:?}", error);
                self.error.set(Some(&*MediaError::new(
                    &*window_from_node(self),
                    MEDIA_ERR_DECODE,
                )));
                self.upcast::<EventTarget>().fire_event(atom!("error"));
            },
            PlayerEvent::FrameUpdated => {
                self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
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
                        let window = window_from_node(self);
                        let audio_track = AudioTrack::new(
                            &window,
                            DOMString::new(),
                            kind,
                            DOMString::new(),
                            DOMString::new(),
                        );

                        // Steps 2. & 3.
                        self.AudioTracks().add(&audio_track);

                        // Step 4
                        // https://www.w3.org/TR/media-frags/#media-fragment-syntax
                        // https://github.com/servo/servo/issues/22366

                        // Step 5. & 6,
                        if self.AudioTracks().enabled_index().is_none() {
                            self.AudioTracks()
                                .set_enabled(self.AudioTracks().len() - 1, true);
                        }

                        // Steps 7.
                        let event = TrackEvent::new(
                            &self.global(),
                            atom!("addtrack"),
                            false,
                            false,
                            &Some(VideoTrackOrAudioTrackOrTextTrack::AudioTrack(audio_track)),
                        );

                        event.upcast::<Event>().fire(self.upcast::<EventTarget>());
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
                        let window = window_from_node(self);
                        let video_track = VideoTrack::new(
                            &window,
                            DOMString::new(),
                            kind,
                            DOMString::new(),
                            DOMString::new(),
                        );

                        // Steps 2. & 3.
                        self.VideoTracks().add(&video_track);

                        // Step 4.
                        // https://www.w3.org/TR/media-frags/#media-fragment-syntax
                        // https://github.com/servo/servo/issues/22366

                        // Step 5. & 6.
                        if self.VideoTracks().selected_index().is_none() {
                            self.VideoTracks()
                                .set_selected(self.VideoTracks().len() - 1, true);
                        }

                        // Steps 7.
                        let event = TrackEvent::new(
                            &self.global(),
                            atom!("addtrack"),
                            false,
                            false,
                            &Some(VideoTrackOrAudioTrackOrTextTrack::VideoTrack(video_track)),
                        );

                        event.upcast::<Event>().fire(self.upcast::<EventTarget>());
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
                    let window = window_from_node(self);
                    let task_source = window.task_manager().media_element_task_source();
                    task_source.queue_simple_event(self.upcast(), atom!("durationchange"), &window);
                }

                // Step 5.
                if self.is::<HTMLVideoElement>() {
                    let video_elem = self.downcast::<HTMLVideoElement>().unwrap();
                    if video_elem.get_video_width() != metadata.width ||
                        video_elem.get_video_height() != metadata.height
                    {
                        video_elem.set_video_width(metadata.width);
                        video_elem.set_video_height(metadata.height);
                        let window = window_from_node(self);
                        let task_source = window.task_manager().media_element_task_source();
                        task_source.queue_simple_event(self.upcast(), atom!("resize"), &window);
                    }
                }

                // Step 6.
                self.change_ready_state(ReadyState::HaveMetadata);

                // Step 7.
                let mut _jumped = false;

                // Step 8.
                if self.default_playback_start_position.get() > 0. {
                    self.seek(
                        self.default_playback_start_position.get(),
                        /* approximate_for_speed*/ false,
                    );
                    _jumped = true;
                }

                // Step 9.
                self.default_playback_start_position.set(0.);

                // Steps 10 and 11.
                // XXX(ferjm) Implement parser for
                //            https://www.w3.org/TR/media-frags/#media-fragment-syntax
                //            https://github.com/servo/media/issues/156

                // Step 12 & 13 are already handled by the earlier media track processing.
            },
            PlayerEvent::NeedData => {
                // The player needs more data.
                // If we already have a valid fetch request, we do nothing.
                // Otherwise, if we have no request and the previous request was
                // cancelled because we got an EnoughData event, we restart
                // fetching where we left.
                if let Some(ref current_fetch_context) = *self.current_fetch_context.borrow() {
                    match current_fetch_context.cancel_reason() {
                        Some(ref reason) if *reason == CancelReason::Backoff => {
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
            },
            PlayerEvent::SeekData(p) => {
                self.fetch_request(Some(p));
            },
            PlayerEvent::SeekDone(_) => {
                // Continuation of
                // https://html.spec.whatwg.org/multipage/#dom-media-seek

                // Step 13.
                let task = MediaElementMicrotask::SeekedTask {
                    elem: DomRoot::from_ref(self),
                    generation_id: self.generation_id.get(),
                };
                ScriptThread::await_stable_state(Microtask::MediaElement(task));
            },
            PlayerEvent::StateChanged(ref state) => match *state {
                PlaybackState::Paused => {
                    if self.ready_state.get() == ReadyState::HaveMetadata {
                        self.change_ready_state(ReadyState::HaveEnoughData);
                    }
                },
                _ => {},
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
        if let Some(ref player) = *self.player.borrow() {
            if let Err(err) = player.shutdown() {
                warn!("Error shutting down player {:?}", err);
            }
        }
    }
}

impl HTMLMediaElementMethods for HTMLMediaElement {
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

    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_url_getter!(Src, "src");

    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_url_setter!(SetSrc, "src");

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
            let _ = player.set_mute(value);
        }

        self.muted.set(value);
        let window = window_from_node(self);
        window
            .task_manager()
            .media_element_task_source()
            .queue_simple_event(self.upcast(), atom!("volumechange"), &window);
        if !self.is_allowed_to_play() {
            self.internal_pause_steps();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-srcobject
    fn GetSrcObject(&self) -> Option<MediaStreamOrBlob> {
        match *self.src_object.borrow() {
            Some(ref src_object) => Some(match src_object {
                SrcObject::Blob(blob) => MediaStreamOrBlob::Blob(DomRoot::from_ref(&*blob)),
                SrcObject::MediaStream(stream) => {
                    MediaStreamOrBlob::MediaStream(DomRoot::from_ref(&*stream))
                },
            }),
            None => None,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-srcobject
    fn SetSrcObject(&self, value: Option<MediaStreamOrBlob>) {
        *self.src_object.borrow_mut() = value.map(|value| value.into());
        self.media_element_load_algorithm();
    }

    // https://html.spec.whatwg.org/multipage/#attr-media-preload
    // Missing value default is user-agent defined.
    make_enumerated_getter!(Preload, "preload", "", "none" | "metadata" | "auto");
    // https://html.spec.whatwg.org/multipage/#attr-media-preload
    make_setter!(SetPreload, "preload");

    // https://html.spec.whatwg.org/multipage/#dom-media-currentsrc
    fn CurrentSrc(&self) -> USVString {
        USVString(self.current_src.borrow().clone())
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-load
    fn Load(&self) {
        self.media_element_load_algorithm();
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-canplaytype
    fn CanPlayType(&self, type_: DOMString) -> CanPlayTypeResult {
        match ServoMedia::get().unwrap().can_play_type(&type_) {
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
    fn Play(&self) -> Rc<Promise> {
        let in_compartment_proof = AlreadyInCompartment::assert(&self.global());
        let promise = Promise::new_in_current_compartment(
            &self.global(),
            InCompartment::Already(&in_compartment_proof),
        );
        // Step 1.
        // FIXME(nox): Reject promise if not allowed to play.

        // Step 2.
        if self
            .error
            .get()
            .map_or(false, |e| e.Code() == MEDIA_ERR_SRC_NOT_SUPPORTED)
        {
            promise.reject_error(Error::NotSupported);
            return promise;
        }

        // Step 3.
        self.push_pending_play_promise(&promise);

        // Step 4.
        if self.network_state.get() == NetworkState::Empty {
            self.invoke_resource_selection_algorithm();
        }

        // Step 5.
        if self.Ended() && self.direction_of_playback() == PlaybackDirection::Forwards {
            self.seek(
                self.earliest_possible_position(),
                /* approximate_for_speed */ false,
            );
        }

        let state = self.ready_state.get();

        let window = window_from_node(self);
        // FIXME(nox): Why are errors silenced here?
        let task_source = window.task_manager().media_element_task_source();
        if self.Paused() {
            // Step 6.1.
            self.paused.set(false);

            // Step 6.2.
            if self.show_poster.get() {
                self.show_poster.set(false);
                self.time_marches_on();
            }

            // Step 6.3.
            task_source.queue_simple_event(self.upcast(), atom!("play"), &window);

            // Step 6.4.
            match state {
                ReadyState::HaveNothing |
                ReadyState::HaveMetadata |
                ReadyState::HaveCurrentData => {
                    task_source.queue_simple_event(self.upcast(), atom!("waiting"), &window);
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
            task_source
                .queue(
                    task!(resolve_pending_play_promises: move || {
                        let this = this.root();
                        if generation_id != this.generation_id.get() {
                            return;
                        }

                        this.fulfill_in_flight_play_promises(|| {
                            this.play_media();
                        });
                    }),
                    window.upcast(),
                )
                .unwrap();
        }

        // Step 8.
        self.autoplaying.set(false);

        // Step 9.
        promise
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-pause
    fn Pause(&self) {
        // Step 1
        if self.network_state.get() == NetworkState::Empty {
            self.invoke_resource_selection_algorithm();
        }

        // Step 2
        self.internal_pause_steps();
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-paused
    fn Paused(&self) -> bool {
        self.paused.get()
    }

    /// https://html.spec.whatwg.org/multipage/#dom-media-defaultplaybackrate
    fn GetDefaultPlaybackRate(&self) -> Fallible<Finite<f64>> {
        Ok(Finite::wrap(self.defaultPlaybackRate.get()))
    }

    /// https://html.spec.whatwg.org/multipage/#dom-media-defaultplaybackrate
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

    /// https://html.spec.whatwg.org/multipage/#dom-media-playbackrate
    fn GetPlaybackRate(&self) -> Fallible<Finite<f64>> {
        Ok(Finite::wrap(self.playbackRate.get()))
    }

    /// https://html.spec.whatwg.org/multipage/#dom-media-playbackrate
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
                    if let Err(e) = player.set_rate(*value) {
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
        TimeRanges::new(self.global().as_window(), self.played.borrow().clone())
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-buffered
    fn Buffered(&self) -> DomRoot<TimeRanges> {
        let mut buffered = TimeRangesContainer::new();
        if let Some(ref player) = *self.player.borrow() {
            if let Ok(ranges) = player.buffered() {
                for range in ranges {
                    let _ = buffered.add(range.start as f64, range.end as f64);
                }
            }
        }
        TimeRanges::new(self.global().as_window(), buffered)
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-audiotracks
    fn AudioTracks(&self) -> DomRoot<AudioTrackList> {
        let window = window_from_node(self);
        self.audio_tracks_list
            .or_init(|| AudioTrackList::new(&window, &[]))
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-videotracks
    fn VideoTracks(&self) -> DomRoot<VideoTrackList> {
        let window = window_from_node(self);
        self.video_tracks_list
            .or_init(|| VideoTrackList::new(&window, &[]))
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-texttracks
    fn TextTracks(&self) -> DomRoot<TextTrackList> {
        let window = window_from_node(self);
        self.text_tracks_list
            .or_init(|| TextTrackList::new(&window, &[]))
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-addtexttrack
    fn AddTextTrack(
        &self,
        kind: TextTrackKind,
        label: DOMString,
        language: DOMString,
    ) -> DomRoot<TextTrack> {
        let window = window_from_node(self);
        // Step 1 & 2
        // FIXME(#22314, dlrobertson) set the ready state to Loaded
        let track = TextTrack::new(
            &window,
            "".into(),
            kind,
            label,
            language,
            TextTrackMode::Hidden,
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

            let window = window_from_node(self);
            window
                .task_manager()
                .media_element_task_source()
                .queue_simple_event(self.upcast(), atom!("volumechange"), &window);
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

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        if &local_name!("muted") == attr.local_name() {
            self.SetMuted(mutation.new_value(attr).is_some());
            return;
        }

        if mutation.new_value(attr).is_none() {
            return;
        }

        match attr.local_name() {
            &local_name!("src") => {
                self.media_element_load_algorithm();
            },
            _ => (),
        };
    }

    // https://html.spec.whatwg.org/multipage/#playing-the-media-resource:remove-an-element-from-a-document
    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        if context.tree_connected {
            let task = MediaElementMicrotask::PauseIfNotInDocumentTask {
                elem: DomRoot::from_ref(self),
            };
            ScriptThread::await_stable_state(Microtask::MediaElement(task));
        }
    }
}

pub trait LayoutHTMLMediaElementHelpers {
    fn data(&self) -> HTMLMediaData;
}

impl LayoutHTMLMediaElementHelpers for LayoutDom<HTMLMediaElement> {
    #[allow(unsafe_code)]
    fn data(&self) -> HTMLMediaData {
        let media = unsafe { &*self.unsafe_get() };
        HTMLMediaData {
            current_frame: media.frame_renderer.lock().unwrap().current_frame.clone(),
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub enum MediaElementMicrotask {
    ResourceSelectionTask {
        elem: DomRoot<HTMLMediaElement>,
        generation_id: u32,
        base_url: ServoUrl,
    },
    PauseIfNotInDocumentTask {
        elem: DomRoot<HTMLMediaElement>,
    },
    SeekedTask {
        elem: DomRoot<HTMLMediaElement>,
        generation_id: u32,
    },
}

impl MicrotaskRunnable for MediaElementMicrotask {
    fn handler(&self) {
        match self {
            &MediaElementMicrotask::ResourceSelectionTask {
                ref elem,
                generation_id,
                ref base_url,
            } => {
                if generation_id == elem.generation_id.get() {
                    elem.resource_selection_algorithm_sync(base_url.clone());
                }
            },
            &MediaElementMicrotask::PauseIfNotInDocumentTask { ref elem } => {
                if !elem.upcast::<Node>().is_connected() {
                    elem.internal_pause_steps();
                }
            },
            &MediaElementMicrotask::SeekedTask {
                ref elem,
                generation_id,
            } => {
                if generation_id == elem.generation_id.get() {
                    elem.seek_end();
                }
            },
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
pub struct HTMLMediaElementFetchContext {
    /// Some if the request has been cancelled.
    cancel_reason: Option<CancelReason>,
    /// Indicates whether the fetched stream is seekable.
    is_seekable: bool,
    /// Fetch canceller. Allows cancelling the current fetch request by
    /// manually calling its .cancel() method or automatically on Drop.
    fetch_canceller: FetchCanceller,
}

impl HTMLMediaElementFetchContext {
    fn new() -> (HTMLMediaElementFetchContext, ipc::IpcReceiver<()>) {
        let mut fetch_canceller = FetchCanceller::new();
        let cancel_receiver = fetch_canceller.initialize();
        (
            HTMLMediaElementFetchContext {
                cancel_reason: None,
                is_seekable: false,
                fetch_canceller,
            },
            cancel_receiver,
        )
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
    next_progress_event: Timespec,
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
}

// https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
impl FetchResponseListener for HTMLMediaElementFetchListener {
    fn process_request_body(&mut self) {}

    fn process_request_eof(&mut self) {}

    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
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
                    } else if let Some(content_length) = headers.typed_get::<ContentLength>() {
                        Some(content_length.0)
                    } else {
                        None
                    };

                // We only set the expected input size if it changes.
                if content_length != self.expected_content_length {
                    if let Some(content_length) = content_length {
                        if let Err(e) = elem
                            .player
                            .borrow()
                            .as_ref()
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

        let (status_is_ok, is_seekable) = self
            .metadata
            .as_ref()
            .and_then(|m| m.status.as_ref())
            .map_or((true, false), |s| {
                (s.0 >= 200 && s.0 < 300, s.0 == 206 || s.0 == 416)
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

    fn process_response_chunk(&mut self, payload: Vec<u8>) {
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

        // Push input data into the player.
        if let Err(e) = elem.player.borrow().as_ref().unwrap().push_data(payload) {
            // If we are pushing too much data and we know that we can
            // restart the download later from where we left, we cancel
            // the current request. Otherwise, we continue the request
            // assuming that we may drop some frames.
            match e {
                PlayerError::EnoughData => {
                    if let Some(ref mut current_fetch_context) =
                        *elem.current_fetch_context.borrow_mut()
                    {
                        current_fetch_context.cancel(CancelReason::Backoff);
                    }
                },
                _ => (),
            }
            warn!("Could not push input data to player {:?}", e);
            return;
        }

        self.latest_fetched_content += payload_len;

        // https://html.spec.whatwg.org/multipage/#concept-media-load-resource step 4,
        // => "If mode is remote" step 2
        if time::get_time() > self.next_progress_event {
            let window = window_from_node(&*elem);
            window
                .task_manager()
                .media_element_task_source()
                .queue_simple_event(elem.upcast(), atom!("progress"), &window);
            self.next_progress_event = time::get_time() + Duration::milliseconds(350);
        }
    }

    // https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
    fn process_response_eof(&mut self, status: Result<ResourceFetchTiming, NetworkError>) {
        let elem = self.elem.root();

        if elem.player.borrow().is_none() {
            return;
        }

        // If an error was previously received and no new fetch request was triggered,
        // we skip processing the payload and notify the media backend that we are done
        // pushing data.
        if elem.generation_id.get() == self.generation_id {
            if let Some(ref current_fetch_context) = *elem.current_fetch_context.borrow() {
                if let Some(CancelReason::Error) = current_fetch_context.cancel_reason() {
                    if let Err(e) = elem.player.borrow().as_ref().unwrap().end_of_stream() {
                        warn!("Could not signal EOS to player {:?}", e);
                    }
                    return;
                }
            }
        }

        if status.is_ok() && self.latest_fetched_content != 0 {
            if elem.ready_state.get() == ReadyState::HaveNothing {
                // Make sure that we don't skip the HaveMetadata and HaveCurrentData
                // states for short streams.
                elem.change_ready_state(ReadyState::HaveMetadata);
            }
            elem.change_ready_state(ReadyState::HaveEnoughData);

            elem.upcast::<EventTarget>().fire_event(atom!("progress"));

            elem.network_state.set(NetworkState::Idle);

            elem.upcast::<EventTarget>().fire_event(atom!("suspend"));

            elem.delay_load_event(false);
        }
        // => "If the connection is interrupted after some media data has been received..."
        else if elem.ready_state.get() != ReadyState::HaveNothing {
            // Step 1
            if let Some(ref mut current_fetch_context) = *elem.current_fetch_context.borrow_mut() {
                current_fetch_context.cancel(CancelReason::Error);
            }

            // Step 2
            elem.error.set(Some(&*MediaError::new(
                &*window_from_node(&*elem),
                MEDIA_ERR_NETWORK,
            )));

            // Step 3
            elem.network_state.set(NetworkState::Idle);

            // Step 4.
            elem.delay_load_event(false);

            // Step 5
            elem.upcast::<EventTarget>().fire_event(atom!("error"));
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
        network_listener::submit_timing(self)
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
        (document_from_node(&*self.elem.root()).global())
    }
}

impl PreInvoke for HTMLMediaElementFetchListener {
    fn should_invoke(&self) -> bool {
        //TODO: finish_load needs to run at some point if the generation changes.
        self.elem.root().generation_id.get() == self.generation_id
    }
}

impl HTMLMediaElementFetchListener {
    fn new(elem: &HTMLMediaElement, url: ServoUrl, offset: u64) -> Self {
        Self {
            elem: Trusted::new(elem),
            metadata: None,
            generation_id: elem.generation_id.get(),
            next_progress_event: time::get_time() + Duration::milliseconds(350),
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
            url,
            expected_content_length: None,
            latest_fetched_content: offset,
        }
    }
}
