/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use audio_video_metadata;
use document_loader::LoadType;
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::CanPlayTypeResult;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementConstants;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementMethods;
use dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorConstants::*;
use dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{Element, AttributeMutation};
use dom::eventtarget::EventTarget;
use dom::htmlaudioelement::HTMLAudioElement;
use dom::htmlelement::HTMLElement;
use dom::htmlsourceelement::HTMLSourceElement;
use dom::htmlvideoelement::HTMLVideoElement;
use dom::mediaerror::MediaError;
use dom::node::{window_from_node, document_from_node, Node, UnbindContext};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use microtask::{Microtask, MicrotaskRunnable};
use net_traits::{FetchResponseListener, FetchMetadata, Metadata, NetworkError};
use net_traits::request::{CredentialsMode, Destination, RequestInit, Type as RequestType};
use network_listener::{NetworkListener, PreInvoke};
use script_thread::{Runnable, ScriptThread};
use servo_url::ServoUrl;
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use task_source::TaskSource;
use time::{self, Timespec, Duration};

#[dom_struct]
pub struct HTMLMediaElement {
    htmlelement: HTMLElement,
    /// https://html.spec.whatwg.org/multipage/#dom-media-networkstate
    // FIXME(nox): Use an enum.
    network_state: Cell<NetworkState>,
    /// https://html.spec.whatwg.org/multipage/#dom-media-readystate
    // FIXME(nox): Use an enum.
    ready_state: Cell<ReadyState>,
    /// https://html.spec.whatwg.org/multipage/#dom-media-currentsrc
    current_src: DOMRefCell<String>,
    // FIXME(nox): Document this one, I have no idea what it is used for.
    generation_id: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#fire-loadeddata
    ///
    /// Reset to false every time the load algorithm is invoked.
    fired_loadeddata_event: Cell<bool>,
    /// https://html.spec.whatwg.org/multipage/#dom-media-error
    error: MutNullableJS<MediaError>,
    /// https://html.spec.whatwg.org/multipage/#dom-media-paused
    paused: Cell<bool>,
    /// https://html.spec.whatwg.org/multipage/#attr-media-autoplay
    autoplaying: Cell<bool>,
    /// The details of the video currently related to this media element.
    // FIXME(nox): Why isn't this in HTMLVideoElement?
    video: DOMRefCell<Option<VideoMedia>>,
}

/// https://html.spec.whatwg.org/multipage/#dom-media-networkstate
#[derive(Clone, Copy, HeapSizeOf, JSTraceable, PartialEq)]
#[repr(u8)]
enum NetworkState {
    Empty = HTMLMediaElementConstants::NETWORK_EMPTY as u8,
    Idle = HTMLMediaElementConstants::NETWORK_IDLE as u8,
    Loading = HTMLMediaElementConstants::NETWORK_LOADING as u8,
    NoSource = HTMLMediaElementConstants::NETWORK_NO_SOURCE as u8,
}

/// https://html.spec.whatwg.org/multipage/#dom-media-readystate
#[derive(Clone, Copy, HeapSizeOf, JSTraceable, PartialEq, PartialOrd)]
#[repr(u8)]
enum ReadyState {
    HaveNothing = HTMLMediaElementConstants::HAVE_NOTHING as u8,
    HaveMetadata = HTMLMediaElementConstants::HAVE_METADATA as u8,
    HaveCurrentData = HTMLMediaElementConstants::HAVE_CURRENT_DATA as u8,
    HaveFutureData = HTMLMediaElementConstants::HAVE_FUTURE_DATA as u8,
    HaveEnoughData = HTMLMediaElementConstants::HAVE_ENOUGH_DATA as u8,
}

#[derive(HeapSizeOf, JSTraceable)]
pub struct VideoMedia {
    format: String,
    #[ignore_heap_size_of = "defined in time"]
    duration: Duration,
    width: u32,
    height: u32,
    video: String,
    audio: Option<String>,
}

impl HTMLMediaElement {
    pub fn new_inherited(
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> Self {
        Self {
            htmlelement: HTMLElement::new_inherited(tag_name, prefix, document),
            network_state: Cell::new(NetworkState::Empty),
            ready_state: Cell::new(ReadyState::HaveNothing),
            current_src: DOMRefCell::new("".to_owned()),
            generation_id: Cell::new(0),
            fired_loadeddata_event: Cell::new(false),
            error: Default::default(),
            paused: Cell::new(true),
            // FIXME(nox): Why is this initialised to true?
            autoplaying: Cell::new(true),
            video: DOMRefCell::new(None),
        }
    }

    /// https://html.spec.whatwg.org/multipage/#internal-pause-steps
    fn internal_pause_steps(&self) {
        // Step 1.
        self.autoplaying.set(false);

        // Step 2.
        if !self.Paused() {
            // Step 2.1.
            self.paused.set(true);

            // Step 2.2.
            // FIXME(nox): Take pending play promises and let promises be the
            // result.

            // Step 2.3.
            let window = window_from_node(self);
            // FIXME(nox): Why are errors silenced here?
            let _ = window.dom_manipulation_task_source().queue(
                box InternalPauseStepsTask(Trusted::new(self.upcast())),
                window.upcast(),
            );
            struct InternalPauseStepsTask(Trusted<EventTarget>);
            impl Runnable for InternalPauseStepsTask {
                fn handler(self: Box<Self>) {
                    let target = self.0.root();

                    // Step 2.3.1.
                    target.fire_event(atom!("timeupdate"));

                    // Step 2.3.2.
                    target.fire_event(atom!("pause"));

                    // Step 2.3.3.
                    // FIXME(nox): Reject pending play promises with promises
                    // and an "AbortError" DOMException.
                }
            }

            // Step 2.4.
            // FIXME(nox): Set the official playback position to the current
            // playback position.
        }
    }

    // https://html.spec.whatwg.org/multipage/#notify-about-playing
    fn notify_about_playing(&self) {
        // Step 1.
        // TODO(nox): Take pending play promises and let promises be the result.

        // Step 2.
        let window = window_from_node(self);
        // FIXME(nox): Why are errors silenced here?
        let _ = window.dom_manipulation_task_source().queue(
            box NotifyAboutPlayingTask(Trusted::new(self.upcast())),
            window.upcast(),
        );
        struct NotifyAboutPlayingTask(Trusted<EventTarget>);
        impl Runnable for NotifyAboutPlayingTask {
            fn handler(self: Box<Self>) {
                let target = self.0.root();

                // Step 2.1.
                target.fire_event(atom!("playing"));

                // Step 2.2.
                // FIXME(nox): Resolve pending play promises with promises.
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#ready-states
    fn change_ready_state(&self, ready_state: ReadyState) {
        let old_ready_state = self.ready_state.get();
        self.ready_state.set(ready_state);

        if self.network_state.get() == NetworkState::Empty {
            return;
        }

        let window = window_from_node(self);
        let task_source = window.dom_manipulation_task_source();

        // Step 1.
        match (old_ready_state, ready_state) {
            (ReadyState::HaveNothing, ReadyState::HaveMetadata) => {
                task_source.queue_simple_event(
                    self.upcast(),
                    atom!("loadedmetadata"),
                    &window,
                );

                // No other steps are applicable in this case.
                return;
            },
            (ReadyState::HaveMetadata, new) if new >= ReadyState::HaveCurrentData => {
                if !self.fired_loadeddata_event.get() {
                    self.fired_loadeddata_event.set(true);
                    task_source.queue_simple_event(
                        self.upcast(),
                        atom!("loadeddata"),
                        &window,
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

        if old_ready_state <= ReadyState::HaveCurrentData && ready_state >= ReadyState::HaveFutureData {
            task_source.queue_simple_event(
                self.upcast(),
                atom!("canplay"),
                &window,
            );

            if !self.Paused() {
                self.notify_about_playing();
            }
        }

        if ready_state != ReadyState::HaveEnoughData {
            // TODO: Check sandboxed automatic features browsing context flag.
            // FIXME(nox): I have no idea what this TODO is about.

            task_source.queue_simple_event(
                self.upcast(),
                atom!("canplaythrough"),
                &window,
            );

            // FIXME(nox): Review this block.
            if self.autoplaying.get() &&
                self.Paused() &&
                self.Autoplay() {
                // Step 1
                self.paused.set(false);
                // TODO step 2: show poster
                // Step 3
                task_source.queue_simple_event(
                    self.upcast(),
                    atom!("play"),
                    &window,
                );
                // Step 4
                self.notify_about_playing();
                // Step 5
                self.autoplaying.set(false);
            }
        }

        // TODO Step 2: Media controller.
        // FIXME(nox): There is no step 2 in the spec.
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn invoke_resource_selection_algorithm(&self) {
        // Step 1
        self.network_state.set(NetworkState::NoSource);

        // TODO step 2 (show poster)
        // TODO step 3 (delay load event)

        // Step 4
        let doc = document_from_node(self);
        let task = MediaElementMicrotask::ResourceSelectionTask {
            elem: Root::from_ref(self),
            base_url: doc.base_url()
        };
        ScriptThread::await_stable_state(Microtask::MediaElement(task));
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    #[allow(unreachable_code)]
    fn resource_selection_algorithm_sync(&self, base_url: ServoUrl) {
        // TODO step 5 (populate pending text tracks)

        // Step 6
        let mode = if false {
            // TODO media provider object
            ResourceSelectionMode::Object
        } else if let Some(attr) = self.upcast::<Element>().get_attribute(&ns!(), &local_name!("src")) {
            ResourceSelectionMode::Attribute(attr.Value().to_string())
        } else if false {  // TODO: when implementing this remove #[allow(unreachable_code)] above.
            // TODO <source> child
            ResourceSelectionMode::Children(panic!())
        } else {
            self.network_state.set(NetworkState::Empty);
            return;
        };

        // Step 7
        self.network_state.set(NetworkState::Loading);

        // Step 8
        let window = window_from_node(self);
        window.dom_manipulation_task_source().queue_simple_event(
            self.upcast(),
            atom!("loadstart"),
            &window,
        );

        // Step 9
        match mode {
            ResourceSelectionMode::Object => {
                // Step 1
                *self.current_src.borrow_mut() = "".to_owned();

                // Step 4
                self.resource_fetch_algorithm(Resource::Object);
            }

            ResourceSelectionMode::Attribute(src) => {
                // Step 1
                if src.is_empty() {
                    self.queue_dedicated_media_source_failure_steps();
                    return;
                }

                // Step 2
                let absolute_url = base_url.join(&src).map_err(|_| ());

                // Step 3
                if let Ok(url) = absolute_url {
                    *self.current_src.borrow_mut() = url.as_str().into();
                    // Step 4
                    self.resource_fetch_algorithm(Resource::Url(url));
                } else {
                    self.queue_dedicated_media_source_failure_steps();
                }
            }

            ResourceSelectionMode::Children(_child) => {
                // TODO
                self.queue_dedicated_media_source_failure_steps()
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-resource
    fn resource_fetch_algorithm(&self, resource: Resource) {
        // TODO step 3 (remove text tracks)

        // Step 4
        if let Resource::Url(url) = resource {
            // 4.1
            if self.Preload() == "none" && !self.autoplaying.get() {
                // 4.1.1
                self.network_state.set(NetworkState::Idle);

                // 4.1.2
                let window = window_from_node(self);
                window.dom_manipulation_task_source().queue_simple_event(
                    self.upcast(),
                    atom!("suspend"),
                    &window,
                );

                // TODO 4.1.3 (delay load flag)

                // TODO 4.1.5-7 (state for load that initiates later)
                return;
            }

            // 4.2
            let context = Arc::new(Mutex::new(HTMLMediaElementContext::new(self, url.clone())));
            let (action_sender, action_receiver) = ipc::channel().unwrap();
            let window = window_from_node(self);
            let listener = NetworkListener {
                context: context,
                task_source: window.networking_task_source(),
                wrapper: Some(window.get_runnable_wrapper())
            };

            ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
                listener.notify_fetch(message.to().unwrap());
            });

            // FIXME: we're supposed to block the load event much earlier than now
            let document = document_from_node(self);

            let ty = if self.is::<HTMLAudioElement>() {
                RequestType::Audio
            } else if self.is::<HTMLVideoElement>() {
                RequestType::Video
            } else {
                unreachable!("Unexpected HTMLMediaElement")
            };

            let request = RequestInit {
                url: url.clone(),
                type_: ty,
                destination: Destination::Media,
                credentials_mode: CredentialsMode::Include,
                use_url_credentials: true,
                origin: document.origin().immutable().clone(),
                pipeline_id: Some(self.global().pipeline_id()),
                referrer_url: Some(document.url()),
                referrer_policy: document.get_referrer_policy(),
                .. RequestInit::default()
            };

            document.fetch_async(LoadType::Media(url), request, action_sender);
        } else {
            // TODO local resource fetch
            self.queue_dedicated_media_source_failure_steps();
        }
    }

    fn queue_dedicated_media_source_failure_steps(&self) {
        let window = window_from_node(self);
        let _ = window.dom_manipulation_task_source().queue(
            box DedicatedMediaSourceFailureTask::new(self), window.upcast());
    }

    // https://html.spec.whatwg.org/multipage/#dedicated-media-source-failure-steps
    fn dedicated_media_source_failure(&self) {
        // Step 1
        self.error.set(Some(&*MediaError::new(&*window_from_node(self),
                                              MEDIA_ERR_SRC_NOT_SUPPORTED)));

        // TODO step 2 (forget resource tracks)

        // Step 3
        self.network_state.set(NetworkState::NoSource);

        // TODO step 4 (show poster)

        // Step 5
        self.upcast::<EventTarget>().fire_event(atom!("error"));

        // TODO step 6 (resolve pending play promises)
        // TODO step 7 (delay load event)
    }

    // https://html.spec.whatwg.org/multipage/#media-element-load-algorithm
    fn media_element_load_algorithm(&self) {
        // Reset the flag that signals whether loadeddata was ever fired for
        // this invokation of the load algorithm.
        self.fired_loadeddata_event.set(false);

        // TODO Step 1 (abort resource selection algorithm instances)

        // Step 2
        self.generation_id.set(self.generation_id.get() + 1);
        // TODO reject pending play promises

        let window = window_from_node(self);
        let task_source = window.dom_manipulation_task_source();

        // Step 3
        let network_state = self.network_state.get();
        if network_state == NetworkState::Loading || network_state == NetworkState::Idle {
            task_source.queue_simple_event(
                self.upcast(),
                atom!("abort"),
                &window,
            );
        }

        // Step 4
        if network_state != NetworkState::Empty {
            // 4.1
            task_source.queue_simple_event(self.upcast(), atom!("emptied"), &window);

            // TODO 4.2 (abort in-progress fetch)

            // TODO 4.3 (detach media provider object)
            // TODO 4.4 (forget resource tracks)

            // 4.5
            if self.ready_state.get() != ReadyState::HaveNothing {
                self.change_ready_state(ReadyState::HaveNothing);
            }

            // 4.6
            if !self.Paused() {
                self.paused.set(true);
            }
            // TODO 4.7 (seeking)
            // TODO 4.8 (playback position)
            // TODO 4.9 (timeline offset)
            // TODO 4.10 (duration)
        }

        // TODO step 5 (playback rate)
        // Step 6
        self.error.set(None);
        self.autoplaying.set(true);

        // Step 7
        self.invoke_resource_selection_algorithm();

        // TODO step 8 (stop previously playing resource)
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

    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_url_getter!(Src, "src");
    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#attr-media-preload
    // Missing value default is user-agent defined.
    make_enumerated_getter!(Preload, "preload", "", "none" | "metadata" | "auto");
    // https://html.spec.whatwg.org/multipage/#attr-media-preload
    make_setter!(SetPreload, "preload");

    // https://html.spec.whatwg.org/multipage/#dom-media-currentsrc
    fn CurrentSrc(&self) -> DOMString {
        DOMString::from(self.current_src.borrow().clone())
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-load
    fn Load(&self) {
        self.media_element_load_algorithm();
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-canplaytype
    fn CanPlayType(&self, _type_: DOMString) -> CanPlayTypeResult {
        // TODO: application/octet-stream
        CanPlayTypeResult::Maybe
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-error
    fn GetError(&self) -> Option<Root<MediaError>> {
        self.error.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-play
    fn Play(&self) {
        // TODO step 1

        // Step 2
        if self.error.get().map_or(false, |e| e.Code() == MEDIA_ERR_SRC_NOT_SUPPORTED) {
            // TODO return rejected promise
            return;
        }

        // TODO step 3

        // Step 4
        if self.network_state.get() == NetworkState::Empty {
            self.invoke_resource_selection_algorithm();
        }

        // TODO step 5 (seek backwards)

        // TODO step 6 (media controller)

        let state = self.ready_state.get();

        // Step 7
        if self.Paused() {
            // 7.1
            self.paused.set(false);

            // TODO 7.2 (show poster)

            let window = window_from_node(self);
            let task_source = window.dom_manipulation_task_source();

            // 7.3
            task_source.queue_simple_event(self.upcast(), atom!("play"), &window);

            // 7.4
            if state == ReadyState::HaveNothing ||
               state == ReadyState::HaveMetadata ||
               state == ReadyState::HaveCurrentData {
                task_source.queue_simple_event(
                    self.upcast(),
                    atom!("waiting"),
                    &window,
                );
            } else {
                self.notify_about_playing();
            }
        }
        // Step 8
        else if state == ReadyState::HaveFutureData || state == ReadyState::HaveEnoughData {
            // TODO resolve pending play promises
        }

        // Step 9
        self.autoplaying.set(false);

        // TODO step 10 (media controller)

        // TODO return promise
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
}

impl VirtualMethods for HTMLMediaElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        match attr.local_name() {
            &local_name!("src") => {
                if mutation.new_value(attr).is_some() {
                    self.media_element_load_algorithm();
                }
            }
            _ => (),
        };
    }

    // https://html.spec.whatwg.org/multipage/#playing-the-media-resource:remove-an-element-from-a-document
    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        if context.tree_in_doc {
            let task = MediaElementMicrotask::PauseIfNotInDocumentTask {
                elem: Root::from_ref(self)
            };
            ScriptThread::await_stable_state(Microtask::MediaElement(task));
        }
    }
}

#[derive(HeapSizeOf, JSTraceable)]
pub enum MediaElementMicrotask {
    ResourceSelectionTask {
        elem: Root<HTMLMediaElement>,
        base_url: ServoUrl
    },
    PauseIfNotInDocumentTask {
        elem: Root<HTMLMediaElement>,
    }
}

impl MicrotaskRunnable for MediaElementMicrotask {
    fn handler(&self) {
        match self {
            &MediaElementMicrotask::ResourceSelectionTask { ref elem, ref base_url } => {
                elem.resource_selection_algorithm_sync(base_url.clone());
            },
            &MediaElementMicrotask::PauseIfNotInDocumentTask { ref elem } => {
                if !elem.upcast::<Node>().is_in_doc() {
                    elem.internal_pause_steps();
                }
            },
        }
    }
}

struct DedicatedMediaSourceFailureTask {
    elem: Trusted<HTMLMediaElement>,
}

impl DedicatedMediaSourceFailureTask {
    fn new(elem: &HTMLMediaElement) -> DedicatedMediaSourceFailureTask {
        DedicatedMediaSourceFailureTask {
            elem: Trusted::new(elem),
        }
    }
}

impl Runnable for DedicatedMediaSourceFailureTask {
    fn handler(self: Box<DedicatedMediaSourceFailureTask>) {
        self.elem.root().dedicated_media_source_failure();
    }
}

enum ResourceSelectionMode {
    Object,
    Attribute(String),
    Children(Root<HTMLSourceElement>),
}

enum Resource {
    Object,
    Url(ServoUrl),
}

struct HTMLMediaElementContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLMediaElement>,
    /// The response body received to date.
    data: Vec<u8>,
    /// The response metadata received to date.
    metadata: Option<Metadata>,
    /// The generation of the media element when this fetch started.
    generation_id: u32,
    /// Time of last progress notification.
    next_progress_event: Timespec,
    /// Url of resource requested.
    url: ServoUrl,
    /// Whether the media metadata has been completely received.
    have_metadata: bool,
    /// True if this response is invalid and should be ignored.
    ignore_response: bool,
}

impl FetchResponseListener for HTMLMediaElementContext {
    fn process_request_body(&mut self) {}

    fn process_request_eof(&mut self) {}

    // https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
        self.metadata = metadata.ok().map(|m| {
            match m {
                FetchMetadata::Unfiltered(m) => m,
                FetchMetadata::Filtered { unsafe_, .. } => unsafe_
            }
        });

        // => "If the media data cannot be fetched at all..."
        let is_failure = self.metadata
                             .as_ref()
                             .and_then(|m| m.status
                                            .as_ref()
                                            .map(|&(s, _)| s < 200 || s >= 300))
                             .unwrap_or(false);
        if is_failure {
            // Ensure that the element doesn't receive any further notifications
            // of the aborted fetch. The dedicated failure steps will be executed
            // when response_complete runs.
            self.ignore_response = true;
        }
    }

    fn process_response_chunk(&mut self, mut payload: Vec<u8>) {
        if self.ignore_response {
            return;
        }

        self.data.append(&mut payload);

        let elem = self.elem.root();

        // https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
        // => "Once enough of the media data has been fetched to determine the duration..."
        if !self.have_metadata {
            self.check_metadata(&elem);
        } else {
            elem.change_ready_state(ReadyState::HaveCurrentData);
        }

        // https://html.spec.whatwg.org/multipage/#concept-media-load-resource step 4,
        // => "If mode is remote" step 2
        if time::get_time() > self.next_progress_event {
            let window = window_from_node(&*elem);
            window.dom_manipulation_task_source().queue_simple_event(
                elem.upcast(),
                atom!("progress"),
                &window,
            );
            self.next_progress_event = time::get_time() + Duration::milliseconds(350);
        }
    }

    // https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
    fn process_response_eof(&mut self, status: Result<(), NetworkError>) {
        let elem = self.elem.root();

        // => "If the media data can be fetched but is found by inspection to be in an unsupported
        //     format, or can otherwise not be rendered at all"
        if !self.have_metadata {
            elem.queue_dedicated_media_source_failure_steps();
        }
        // => "Once the entire media resource has been fetched..."
        else if status.is_ok() {
            elem.change_ready_state(ReadyState::HaveEnoughData);

            elem.upcast::<EventTarget>().fire_event(atom!("progress"));

            elem.network_state.set(NetworkState::Idle);

            elem.upcast::<EventTarget>().fire_event(atom!("suspend"));
        }
        // => "If the connection is interrupted after some media data has been received..."
        else if elem.ready_state.get() != ReadyState::HaveNothing {
            // Step 2
            elem.error.set(Some(&*MediaError::new(&*window_from_node(&*elem),
                                                  MEDIA_ERR_NETWORK)));

            // Step 3
            elem.network_state.set(NetworkState::Idle);

            // TODO: Step 4 - update delay load flag

            // Step 5
            elem.upcast::<EventTarget>().fire_event(atom!("error"));
        } else {
            // => "If the media data cannot be fetched at all..."
            elem.queue_dedicated_media_source_failure_steps();
        }

        let document = document_from_node(&*elem);
        document.finish_load(LoadType::Media(self.url.clone()));
    }
}

impl PreInvoke for HTMLMediaElementContext {
    fn should_invoke(&self) -> bool {
        //TODO: finish_load needs to run at some point if the generation changes.
        self.elem.root().generation_id.get() == self.generation_id
    }
}

impl HTMLMediaElementContext {
    fn new(elem: &HTMLMediaElement, url: ServoUrl) -> HTMLMediaElementContext {
        HTMLMediaElementContext {
            elem: Trusted::new(elem),
            data: vec![],
            metadata: None,
            generation_id: elem.generation_id.get(),
            next_progress_event: time::get_time() + Duration::milliseconds(350),
            url: url,
            have_metadata: false,
            ignore_response: false,
        }
    }

    fn check_metadata(&mut self, elem: &HTMLMediaElement) {
        match audio_video_metadata::get_format_from_slice(&self.data) {
            Ok(audio_video_metadata::Metadata::Video(meta)) => {
                let dur = meta.audio.duration.unwrap_or(::std::time::Duration::new(0, 0));
                *elem.video.borrow_mut() = Some(VideoMedia {
                    format: format!("{:?}", meta.format),
                    duration: Duration::seconds(dur.as_secs() as i64) +
                              Duration::nanoseconds(dur.subsec_nanos() as i64),
                    width: meta.dimensions.width,
                    height: meta.dimensions.height,
                    video: meta.video.unwrap_or("".to_owned()),
                    audio: meta.audio.audio,
                });
                // Step 6
                elem.change_ready_state(ReadyState::HaveMetadata);
                self.have_metadata = true;
            }
            _ => {}
        }
    }
}
