/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use audio_video_metadata;
use document_loader::LoadType;
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::CanPlayTypeResult;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementConstants::*;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementMethods;
use dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorConstants::*;
use dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, MutNullableHeap, JS};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{Element, AttributeMutation};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlaudioelement::HTMLAudioElement;
use dom::htmlelement::HTMLElement;
use dom::htmlsourceelement::HTMLSourceElement;
use dom::htmlvideoelement::HTMLVideoElement;
use dom::mediaerror::MediaError;
use dom::node::{window_from_node, document_from_node, Node, UnbindContext};
use dom::virtualmethods::VirtualMethods;
use html5ever_atoms::LocalName;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::{FetchResponseListener, FetchMetadata, Metadata, NetworkError};
use net_traits::request::{CredentialsMode, Destination, RequestInit, Type as RequestType};
use network_listener::{NetworkListener, PreInvoke};
use script_thread::{Runnable, ScriptThread};
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use task_source::TaskSource;
use time::{self, Timespec, Duration};

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
            elem.change_ready_state(HAVE_CURRENT_DATA);
        }

        // https://html.spec.whatwg.org/multipage/#concept-media-load-resource step 4,
        // => "If mode is remote" step 2
        if time::get_time() > self.next_progress_event {
            elem.queue_fire_simple_event("progress");
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
            elem.change_ready_state(HAVE_ENOUGH_DATA);

            elem.fire_simple_event("progress");

            elem.network_state.set(NETWORK_IDLE);

            elem.fire_simple_event("suspend");
        }
        // => "If the connection is interrupted after some media data has been received..."
        else if elem.ready_state.get() != HAVE_NOTHING {
            // Step 2
            elem.error.set(Some(&*MediaError::new(&*window_from_node(&*elem),
                                                  MEDIA_ERR_NETWORK)));

            // Step 3
            elem.network_state.set(NETWORK_IDLE);

            // TODO: Step 4 - update delay load flag

            // Step 5
            elem.fire_simple_event("error");
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
                elem.change_ready_state(HAVE_METADATA);
                self.have_metadata = true;
            }
            _ => {}
        }
    }
}

#[derive(JSTraceable, HeapSizeOf)]
pub struct VideoMedia {
    format: String,
    #[ignore_heap_size_of = "defined in time"]
    duration: Duration,
    width: u32,
    height: u32,
    video: String,
    audio: Option<String>,
}

#[dom_struct]
pub struct HTMLMediaElement {
    htmlelement: HTMLElement,
    network_state: Cell<u16>,
    ready_state: Cell<u16>,
    current_src: DOMRefCell<String>,
    generation_id: Cell<u32>,
    first_data_load: Cell<bool>,
    error: MutNullableHeap<JS<MediaError>>,
    paused: Cell<bool>,
    autoplaying: Cell<bool>,
    video: DOMRefCell<Option<VideoMedia>>,
}

impl HTMLMediaElement {
    pub fn new_inherited(tag_name: LocalName,
                         prefix: Option<DOMString>, document: &Document)
                         -> HTMLMediaElement {
        HTMLMediaElement {
            htmlelement:
                HTMLElement::new_inherited(tag_name, prefix, document),
            network_state: Cell::new(NETWORK_EMPTY),
            ready_state: Cell::new(HAVE_NOTHING),
            current_src: DOMRefCell::new("".to_owned()),
            generation_id: Cell::new(0),
            first_data_load: Cell::new(true),
            error: Default::default(),
            paused: Cell::new(true),
            autoplaying: Cell::new(true),
            video: DOMRefCell::new(None),
        }
    }

    #[inline]
    pub fn htmlelement(&self) -> &HTMLElement {
        &self.htmlelement
    }

    // https://html.spec.whatwg.org/multipage/#internal-pause-steps
    fn internal_pause_steps(&self) {
        // Step 1
        self.autoplaying.set(false);

        // Step 2
        if !self.Paused() {
            // 2.1
            self.paused.set(true);

            // 2.2
            self.queue_internal_pause_steps_task();

            // TODO 2.3 (official playback position)
        }

        // TODO step 3 (media controller)
    }

    // https://html.spec.whatwg.org/multipage/#notify-about-playing
    fn notify_about_playing(&self) {
        // Step 1
        self.fire_simple_event("playing");
        // TODO Step 2
    }

    fn queue_notify_about_playing(&self) {
        struct Task {
            elem: Trusted<HTMLMediaElement>,
        }

        impl Runnable for Task {
            fn handler(self: Box<Task>) {
                self.elem.root().notify_about_playing();
            }
        }

        let task = box Task {
            elem: Trusted::new(self),
        };
        let win = window_from_node(self);
        let _ = win.dom_manipulation_task_source().queue(task, win.upcast());
    }

    // https://html.spec.whatwg.org/multipage/#internal-pause-steps step 2.2
    fn queue_internal_pause_steps_task(&self) {
        struct Task {
            elem: Trusted<HTMLMediaElement>,
        }

        impl Runnable for Task {
            fn handler(self: Box<Task>) {
                let elem = self.elem.root();
                // 2.2.1
                elem.fire_simple_event("timeupdate");
                // 2.2.2
                elem.fire_simple_event("pause");
                // TODO 2.2.3
            }
        }

        let task = box Task {
            elem: Trusted::new(self),
        };
        let win = window_from_node(self);
        let _ = win.dom_manipulation_task_source().queue(task, win.upcast());
    }

    fn queue_fire_simple_event(&self, type_: &'static str) {
        let win = window_from_node(self);
        let task = box FireSimpleEventTask::new(self, type_);
        let _ = win.dom_manipulation_task_source().queue(task, win.upcast());
    }

    fn fire_simple_event(&self, type_: &str) {
        let window = window_from_node(self);
        let event = Event::new(window.upcast(),
                               Atom::from(type_),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        event.fire(self.upcast());
    }

    // https://html.spec.whatwg.org/multipage/#ready-states
    fn change_ready_state(&self, ready_state: u16) {
        let old_ready_state = self.ready_state.get();
        self.ready_state.set(ready_state);

        if self.network_state.get() == NETWORK_EMPTY {
            return;
        }

        // Step 1
        match (old_ready_state, ready_state) {
            // previous ready state was HAVE_NOTHING, and the new ready state is
            // HAVE_METADATA
            (HAVE_NOTHING, HAVE_METADATA) => {
                self.queue_fire_simple_event("loadedmetadata");
            }

            // previous ready state was HAVE_METADATA and the new ready state is
            // HAVE_CURRENT_DATA or greater
            (HAVE_METADATA, HAVE_CURRENT_DATA) |
            (HAVE_METADATA, HAVE_FUTURE_DATA) |
            (HAVE_METADATA, HAVE_ENOUGH_DATA) => {
                if self.first_data_load.get() {
                    self.first_data_load.set(false);
                    self.queue_fire_simple_event("loadeddata");
                }
            }

            // previous ready state was HAVE_FUTURE_DATA or more, and the new ready
            // state is HAVE_CURRENT_DATA or less
            (HAVE_FUTURE_DATA, HAVE_CURRENT_DATA) |
            (HAVE_ENOUGH_DATA, HAVE_CURRENT_DATA) |
            (HAVE_FUTURE_DATA, HAVE_METADATA) |
            (HAVE_ENOUGH_DATA, HAVE_METADATA) |
            (HAVE_FUTURE_DATA, HAVE_NOTHING) |
            (HAVE_ENOUGH_DATA, HAVE_NOTHING) => {
                // TODO: timeupdate event logic + waiting
            }

            _ => (),
        }

        // Step 1
        // If the new ready state is HAVE_FUTURE_DATA or HAVE_ENOUGH_DATA,
        // then the relevant steps below must then be run also.
        match (old_ready_state, ready_state) {
            // previous ready state was HAVE_CURRENT_DATA or less, and the new ready
            // state is HAVE_FUTURE_DATA
            (HAVE_CURRENT_DATA, HAVE_FUTURE_DATA) |
            (HAVE_METADATA, HAVE_FUTURE_DATA) |
            (HAVE_NOTHING, HAVE_FUTURE_DATA) => {
                self.queue_fire_simple_event("canplay");

                if !self.Paused() {
                    self.queue_notify_about_playing();
                }
            }

            // new ready state is HAVE_ENOUGH_DATA
            (_, HAVE_ENOUGH_DATA) => {
                if old_ready_state <= HAVE_CURRENT_DATA {
                    self.queue_fire_simple_event("canplay");

                    if !self.Paused() {
                        self.queue_notify_about_playing();
                    }
                }

                //TODO: check sandboxed automatic features browsing context flag
                if self.autoplaying.get() &&
                   self.Paused() &&
                   self.Autoplay() {
                    // Step 1
                    self.paused.set(false);
                    // TODO step 2: show poster
                    // Step 3
                    self.queue_fire_simple_event("play");
                    // Step 4
                    self.queue_notify_about_playing();
                    // Step 5
                    self.autoplaying.set(false);
                }

                self.queue_fire_simple_event("canplaythrough");
            }

            _ => (),
        }

        // TODO Step 2: media controller
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn invoke_resource_selection_algorithm(&self) {
        // Step 1
        self.network_state.set(NETWORK_NO_SOURCE);

        // TODO step 2 (show poster)
        // TODO step 3 (delay load event)

        // Step 4
        let doc = document_from_node(self);
        ScriptThread::await_stable_state(ResourceSelectionTask::new(self, doc.base_url()));
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn resource_selection_algorithm_sync(&self, base_url: ServoUrl) {
        // TODO step 5 (populate pending text tracks)

        // Step 6
        let mode = if false {
            // TODO media provider object
            ResourceSelectionMode::Object
        } else if let Some(attr) = self.upcast::<Element>().get_attribute(&ns!(), &local_name!("src")) {
            ResourceSelectionMode::Attribute(attr.Value().to_string())
        } else if false {
            // TODO <source> child
            ResourceSelectionMode::Children(panic!())
        } else {
            self.network_state.set(NETWORK_EMPTY);
            return;
        };

        // Step 7
        self.network_state.set(NETWORK_LOADING);

        // Step 8
        self.queue_fire_simple_event("loadstart");

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
                self.network_state.set(NETWORK_IDLE);

                // 4.1.2
                self.queue_fire_simple_event("suspend");

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
                origin: document.url().clone(),
                pipeline_id: Some(self.global().pipeline_id()),
                referrer_url: Some(document.url().clone()),
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
        self.network_state.set(NETWORK_NO_SOURCE);

        // TODO step 4 (show poster)

        // Step 5
        self.fire_simple_event("error");

        // TODO step 6 (resolve pending play promises)
        // TODO step 7 (delay load event)
    }

    // https://html.spec.whatwg.org/multipage/#media-element-load-algorithm
    fn media_element_load_algorithm(&self) {
        self.first_data_load.set(true);

        // TODO Step 1 (abort resource selection algorithm instances)

        // Step 2
        self.generation_id.set(self.generation_id.get() + 1);
        // TODO reject pending play promises

        // Step 3
        let network_state = self.NetworkState();
        if network_state == NETWORK_LOADING || network_state == NETWORK_IDLE {
            self.queue_fire_simple_event("abort");
        }

        // Step 4
        if network_state != NETWORK_EMPTY {
            // 4.1
            self.queue_fire_simple_event("emptied");

            // TODO 4.2 (abort in-progress fetch)

            // TODO 4.3 (detach media provider object)
            // TODO 4.4 (forget resource tracks)

            // 4.5
            if self.ready_state.get() != HAVE_NOTHING {
                self.change_ready_state(HAVE_NOTHING);
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
        self.network_state.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-readystate
    fn ReadyState(&self) -> u16 {
        self.ready_state.get()
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
        if self.network_state.get() == NETWORK_EMPTY {
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

            // 7.3
            self.queue_fire_simple_event("play");

            // 7.4
            if state == HAVE_NOTHING ||
               state == HAVE_METADATA ||
               state == HAVE_CURRENT_DATA {
                self.queue_fire_simple_event("waiting");
            } else {
                self.queue_notify_about_playing();
            }
        }
        // Step 8
        else if state == HAVE_FUTURE_DATA || state == HAVE_ENOUGH_DATA {
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
        if self.network_state.get() == NETWORK_EMPTY {
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
            ScriptThread::await_stable_state(PauseIfNotInDocumentTask::new(self));
        }
    }
}

struct FireSimpleEventTask {
    elem: Trusted<HTMLMediaElement>,
    type_: &'static str,
}

impl FireSimpleEventTask {
    fn new(target: &HTMLMediaElement, type_: &'static str) -> FireSimpleEventTask {
        FireSimpleEventTask {
            elem: Trusted::new(target),
            type_: type_,
        }
    }
}

impl Runnable for FireSimpleEventTask {
    fn name(&self) -> &'static str { "FireSimpleEventTask" }

    fn handler(self: Box<FireSimpleEventTask>) {
        let elem = self.elem.root();
        elem.fire_simple_event(self.type_);
    }
}

struct ResourceSelectionTask {
    elem: Trusted<HTMLMediaElement>,
    base_url: ServoUrl,
}

impl ResourceSelectionTask {
    fn new(elem: &HTMLMediaElement, url: ServoUrl) -> ResourceSelectionTask {
        ResourceSelectionTask {
            elem: Trusted::new(elem),
            base_url: url,
        }
    }
}

impl Runnable for ResourceSelectionTask {
    fn name(&self) -> &'static str { "ResourceSelectionTask" }

    fn handler(self: Box<ResourceSelectionTask>) {
        self.elem.root().resource_selection_algorithm_sync(self.base_url);
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
    fn name(&self) -> &'static str { "DedicatedMediaSourceFailureTask" }

    fn handler(self: Box<DedicatedMediaSourceFailureTask>) {
        self.elem.root().dedicated_media_source_failure();
    }
}

struct PauseIfNotInDocumentTask {
    elem: Trusted<HTMLMediaElement>,
}

impl PauseIfNotInDocumentTask {
    fn new(elem: &HTMLMediaElement) -> PauseIfNotInDocumentTask {
        PauseIfNotInDocumentTask {
            elem: Trusted::new(elem),
        }
    }
}

impl Runnable for PauseIfNotInDocumentTask {
    fn name(&self) -> &'static str { "PauseIfNotInDocumentTask" }

    fn handler(self: Box<PauseIfNotInDocumentTask>) {
        let elem = self.elem.root();
        if !elem.upcast::<Node>().is_in_doc() {
            elem.internal_pause_steps();
        }
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
