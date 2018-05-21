/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use audio_video_metadata;
use document_loader::{LoadBlocker, LoadType};
use dom::attr::Attr;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::CanPlayTypeResult;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementConstants;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementMethods;
use dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorConstants::*;
use dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorMethods;
use dom::bindings::codegen::InheritTypes::{ElementTypeId, HTMLElementTypeId};
use dom::bindings::codegen::InheritTypes::{HTMLMediaElementTypeId, NodeTypeId};
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::bindings::str::DOMString;
use dom::blob::Blob;
use dom::document::Document;
use dom::element::{Element, AttributeMutation};
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::htmlsourceelement::HTMLSourceElement;
use dom::mediaerror::MediaError;
use dom::node::{window_from_node, document_from_node, Node, UnbindContext};
use dom::promise::Promise;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use microtask::{Microtask, MicrotaskRunnable};
use mime::{Mime, SubLevel, TopLevel};
use net_traits::{FetchResponseListener, FetchMetadata, Metadata, NetworkError};
use net_traits::request::{CredentialsMode, Destination, RequestInit};
use network_listener::{NetworkListener, PreInvoke};
use script_thread::ScriptThread;
use servo_url::ServoUrl;
use std::cell::Cell;
use std::collections::VecDeque;
use std::mem;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use task_source::TaskSource;
use time::{self, Timespec, Duration};

#[dom_struct]
// FIXME(nox): A lot of tasks queued for this element should probably be in the
// media element event task source.
pub struct HTMLMediaElement {
    htmlelement: HTMLElement,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-networkstate>
    network_state: Cell<NetworkState>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-readystate>
    ready_state: Cell<ReadyState>,
    /// <https://html.spec.whatwg.org/multipage/#dom-media-srcobject>
    src_object: MutNullableDom<Blob>,
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
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq, PartialOrd)]
#[repr(u8)]
enum ReadyState {
    HaveNothing = HTMLMediaElementConstants::HAVE_NOTHING as u8,
    HaveMetadata = HTMLMediaElementConstants::HAVE_METADATA as u8,
    HaveCurrentData = HTMLMediaElementConstants::HAVE_CURRENT_DATA as u8,
    HaveFutureData = HTMLMediaElementConstants::HAVE_FUTURE_DATA as u8,
    HaveEnoughData = HTMLMediaElementConstants::HAVE_ENOUGH_DATA as u8,
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
            src_object: Default::default(),
            current_src: DomRefCell::new("".to_owned()),
            generation_id: Cell::new(0),
            fired_loadeddata_event: Cell::new(false),
            error: Default::default(),
            paused: Cell::new(true),
            // FIXME(nox): Why is this initialised to true?
            autoplaying: Cell::new(true),
            delaying_the_load_event_flag: Default::default(),
            pending_play_promises: Default::default(),
            in_flight_play_promises_queue: Default::default(),
        }
    }

    fn media_type_id(&self) -> HTMLMediaElementTypeId {
        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLMediaElement(media_type_id),
            )) => {
                media_type_id
            },
            _ => unreachable!(),
        }
    }

    /// Marks that element as delaying the load event or not.
    ///
    /// Nothing happens if the element was already delaying the load event and
    /// we pass true to that method again.
    ///
    /// <https://html.spec.whatwg.org/multipage/#delaying-the-load-event-flag>
    fn delay_load_event(&self, delay: bool) {
        let mut blocker = self.delaying_the_load_event_flag.borrow_mut();
        if delay && blocker.is_none() {
            *blocker = Some(LoadBlocker::new(&document_from_node(self), LoadType::Media));
        } else if !delay && blocker.is_some() {
            LoadBlocker::terminate(&mut *blocker);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-media-play>
    // FIXME(nox): Move this back to HTMLMediaElementMethods::Play once
    // Rc<Promise> doesn't require #[allow(unrooted_must_root)] anymore.
    fn play(&self, promise: &Rc<Promise>) {
        // Step 1.
        // FIXME(nox): Reject promise if not allowed to play.

        // Step 2.
        if self.error.get().map_or(false, |e| e.Code() == MEDIA_ERR_SRC_NOT_SUPPORTED) {
            promise.reject_error(Error::NotSupported);
            return;
        }

        // Step 3.
        self.push_pending_play_promise(promise);

        // Step 4.
        if self.network_state.get() == NetworkState::Empty {
            self.invoke_resource_selection_algorithm();
        }

        // Step 5.
        // FIXME(nox): Seek to earliest possible position if playback has ended
        // and direction of playback is forwards.

        let state = self.ready_state.get();

        let window = window_from_node(self);
        let task_source = window.dom_manipulation_task_source();
        if self.Paused() {
            // Step 6.1.
            self.paused.set(false);

            // Step 6.2.
            // FIXME(nox): Set show poster flag to false and run time marches on
            // steps if show poster flag is true.

            // Step 6.3.
            task_source.queue_simple_event(self.upcast(), atom!("play"), &window);

            // Step 6.4.
            match state {
                ReadyState::HaveNothing |
                ReadyState::HaveMetadata |
                ReadyState::HaveCurrentData => {
                    task_source.queue_simple_event(
                        self.upcast(),
                        atom!("waiting"),
                        &window,
                    );
                },
                ReadyState::HaveFutureData |
                ReadyState::HaveEnoughData => {
                    self.notify_about_playing();
                }
            }
        } else if state == ReadyState::HaveFutureData || state == ReadyState::HaveEnoughData {
            // Step 7.
            self.take_pending_play_promises(Ok(()));
            let this = Trusted::new(self);
            let generation_id = self.generation_id.get();
            task_source.queue(
                task!(resolve_pending_play_promises: move || {
                    let this = this.root();
                    if generation_id != this.generation_id.get() {
                        return;
                    }

                    this.fulfill_in_flight_play_promises(|| ());
                }),
                window.upcast(),
            ).unwrap();
        }

        // Step 8.
        self.autoplaying.set(false);

        // Step 9.
        // Not applicable here, the promise is returned from Play.
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
            // FIXME(nox): Why are errors silenced here?
            // FIXME(nox): Media element event task source should be used here.
            let _ = window.dom_manipulation_task_source().queue(
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

    // https://html.spec.whatwg.org/multipage/#notify-about-playing
    fn notify_about_playing(&self) {
        // Step 1.
        self.take_pending_play_promises(Ok(()));

        // Step 2.
        let window = window_from_node(self);
        let this = Trusted::new(self);
        let generation_id = self.generation_id.get();
        // FIXME(nox): Why are errors silenced here?
        // FIXME(nox): Media element event task source should be used here.
        let _ = window.dom_manipulation_task_source().queue(
            task!(notify_about_playing: move || {
                let this = this.root();
                if generation_id != this.generation_id.get() {
                    return;
                }

                this.fulfill_in_flight_play_promises(|| {
                    // Step 2.1.
                    this.upcast::<EventTarget>().fire_event(atom!("playing"));

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

        if ready_state == ReadyState::HaveEnoughData {
            // TODO: Check sandboxed automatic features browsing context flag.
            // FIXME(nox): I have no idea what this TODO is about.

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

            // FIXME(nox): According to the spec, this should come *before* the
            // "play" event.
            task_source.queue_simple_event(
                self.upcast(),
                atom!("canplaythrough"),
                &window,
            );
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn invoke_resource_selection_algorithm(&self) {
        // Step 1.
        self.network_state.set(NetworkState::NoSource);

        // Step 2.
        // FIXME(nox): Set show poster flag to true.

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
            base_url: doc.base_url()
        };

        // FIXME(nox): This will later call the resource_selection_algorith_sync
        // method from below, if microtasks were trait objects, we would be able
        // to put the code directly in this method, without the boilerplate
        // indirections.
        ScriptThread::await_stable_state(Microtask::MediaElement(task));
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn resource_selection_algorithm_sync(&self, base_url: ServoUrl) {
        // Step 5.
        // FIXME(nox): Maybe populate the list of pending text tracks.

        // Step 6.
        enum Mode {
            Object,
            Attribute(String),
            Children(DomRoot<HTMLSourceElement>),
        }
        fn mode(media: &HTMLMediaElement) -> Option<Mode> {
            if media.src_object.get().is_some() {
                return Some(Mode::Object);
            }
            if let Some(attr) = media.upcast::<Element>().get_attribute(&ns!(), &local_name!("src")) {
                return Some(Mode::Attribute(attr.Value().into()));
            }
            let source_child_element = media.upcast::<Node>()
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
        window.dom_manipulation_task_source().queue_simple_event(
            self.upcast(),
            atom!("loadstart"),
            &window,
        );

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
                    }
                };

                // Step 9.attr.3.
                *self.current_src.borrow_mut() = url_record.as_str().into();

                // Step 9.attr.4.
                // Note that the resource fetch algorithm itself takes care
                // of the cleanup in case of failure itself.
                self.resource_fetch_algorithm(Resource::Url(url_record));
            },
            Mode::Children(_source) => {
                // Step 9.children.
                self.queue_dedicated_media_source_failure_steps()
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-resource
    fn resource_fetch_algorithm(&self, resource: Resource) {
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
                    window.dom_manipulation_task_source().queue_simple_event(
                        self.upcast(),
                        atom!("suspend"),
                        &window,
                    );

                    // Step 4.remote.1.3.
                    let this = Trusted::new(self);
                    window.dom_manipulation_task_source().queue(
                        task!(set_media_delay_load_event_flag_to_false: move || {
                            this.root().delay_load_event(false);
                        }),
                        window.upcast(),
                    ).unwrap();

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
                // FIXME(nox): Handle CORS setting from crossorigin attribute.
                let document = document_from_node(self);
                let destination = match self.media_type_id() {
                    HTMLMediaElementTypeId::HTMLAudioElement => Destination::Audio,
                    HTMLMediaElementTypeId::HTMLVideoElement => Destination::Video,
                };
                let request = RequestInit {
                    url,
                    destination,
                    credentials_mode: CredentialsMode::Include,
                    use_url_credentials: true,
                    origin: document.origin().immutable().clone(),
                    pipeline_id: Some(self.global().pipeline_id()),
                    referrer_url: Some(document.url()),
                    referrer_policy: document.get_referrer_policy(),
                    .. RequestInit::default()
                };

                let context = Arc::new(Mutex::new(HTMLMediaElementContext::new(self)));
                let (action_sender, action_receiver) = ipc::channel().unwrap();
                let window = window_from_node(self);
                let listener = NetworkListener {
                    context: context,
                    task_source: window.networking_task_source(),
                    canceller: Some(window.task_canceller())
                };
                ROUTER.add_route(action_receiver.to_opaque(), Box::new(move |message| {
                    listener.notify_fetch(message.to().unwrap());
                }));
                document.loader().fetch_async_background(request, action_sender);
            },
            Resource::Object => {
                // FIXME(nox): Actually do something with the object.
                self.queue_dedicated_media_source_failure_steps();
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
        // FIXME(nox): Media element event task source should be used here.
        let _ = window.dom_manipulation_task_source().queue(
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
                    // FIXME(nox): Forget the media-resource-specific tracks.

                    // Step 3.
                    this.network_state.set(NetworkState::NoSource);

                    // Step 4.
                    // FIXME(nox): Set show poster flag to true.

                    // Step 5.
                    this.upcast::<EventTarget>().fire_event(atom!("error"));

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
        let task_source = window.dom_manipulation_task_source();

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
            // FIXME(nox): Abort in-progress fetching process.

            // Step 6.3.
            // FIXME(nox): Detach MediaSource media provider object.

            // Step 6.4.
            // FIXME(nox): Forget the media-resource-specific tracks.

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
            // FIXME(nox): If seeking is true, set it to false.

            // Step 6.8.
            // FIXME(nox): Set current and official playback position to 0 and
            // maybe queue a task to fire a timeupdate event.

            // Step 6.9.
            // FIXME(nox): Set timeline offset to NaN.

            // Step 6.10.
            // FIXME(nox): Set duration to NaN.
        }

        // Step 7.
        // FIXME(nox): Set playbackRate to defaultPlaybackRate.

        // Step 8.
        self.error.set(None);
        self.autoplaying.set(true);

        // Step 9.
        self.invoke_resource_selection_algorithm();

        // Step 10.
        // FIXME(nox): Stop playback of any previously running media resource.
    }

    /// Appends a promise to the list of pending play promises.
    #[allow(unrooted_must_root)]
    fn push_pending_play_promise(&self, promise: &Rc<Promise>) {
        self.pending_play_promises.borrow_mut().push(promise.clone());
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
    #[allow(unrooted_must_root)]
    fn take_pending_play_promises(&self, result: ErrorResult) {
        let pending_play_promises = mem::replace(
            &mut *self.pending_play_promises.borrow_mut(),
            vec![],
        );
        self.in_flight_play_promises_queue.borrow_mut().push_back((
            pending_play_promises.into(),
            result,
        ));
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
        let (promises, result) = self.in_flight_play_promises_queue
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

    // https://html.spec.whatwg.org/multipage/#dom-media-srcobject
    fn GetSrcObject(&self) -> Option<DomRoot<Blob>> {
        self.src_object.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-srcobject
    fn SetSrcObject(&self, value: Option<&Blob>) {
        self.src_object.set(value);
        self.media_element_load_algorithm();
    }

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
    fn CanPlayType(&self, type_: DOMString) -> CanPlayTypeResult {
        match type_.parse::<Mime>() {
            Ok(Mime(TopLevel::Application, SubLevel::OctetStream, _)) |
            Err(_) => {
                CanPlayTypeResult::_empty
            },
            _ => CanPlayTypeResult::Maybe
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-error
    fn GetError(&self) -> Option<DomRoot<MediaError>> {
        self.error.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-play
    #[allow(unrooted_must_root)]
    fn Play(&self) -> Rc<Promise> {
        let promise = Promise::new(&self.global());
        self.play(&promise);
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
                elem: DomRoot::from_ref(self)
            };
            ScriptThread::await_stable_state(Microtask::MediaElement(task));
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
    }
}

impl MicrotaskRunnable for MediaElementMicrotask {
    fn handler(&self) {
        match self {
            &MediaElementMicrotask::ResourceSelectionTask { ref elem, generation_id, ref base_url } => {
                if generation_id == elem.generation_id.get() {
                    elem.resource_selection_algorithm_sync(base_url.clone());
                }
            },
            &MediaElementMicrotask::PauseIfNotInDocumentTask { ref elem } => {
                if !elem.upcast::<Node>().is_in_doc() {
                    elem.internal_pause_steps();
                }
            },
        }
    }
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
    /// Whether the media metadata has been completely received.
    have_metadata: bool,
    /// True if this response is invalid and should be ignored.
    ignore_response: bool,
}

// https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
impl FetchResponseListener for HTMLMediaElementContext {
    fn process_request_body(&mut self) {}

    fn process_request_eof(&mut self) {}

    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
        self.metadata = metadata.ok().map(|m| {
            match m {
                FetchMetadata::Unfiltered(m) => m,
                FetchMetadata::Filtered { unsafe_, .. } => unsafe_
            }
        });

        let status_is_ok = self.metadata.as_ref()
            .and_then(|m| m.status.as_ref())
            .map_or(true, |s| s.0 >= 200 && s.0 < 300);

        // => "If the media data cannot be fetched at all..."
        if !status_is_ok {
            // Ensure that the element doesn't receive any further notifications
            // of the aborted fetch.
            self.ignore_response = true;
            self.elem.root().queue_dedicated_media_source_failure_steps();
        }
    }

    fn process_response_chunk(&mut self, mut payload: Vec<u8>) {
        if self.ignore_response {
            // An error was received previously, skip processing the payload.
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
        if self.ignore_response {
            // An error was received previously, skip processing the payload.
            return;
        }
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

            // Step 4.
            elem.delay_load_event(false);

            // Step 5
            elem.upcast::<EventTarget>().fire_event(atom!("error"));
        } else {
            // => "If the media data cannot be fetched at all..."
            elem.queue_dedicated_media_source_failure_steps();
        }
    }
}

impl PreInvoke for HTMLMediaElementContext {
    fn should_invoke(&self) -> bool {
        //TODO: finish_load needs to run at some point if the generation changes.
        self.elem.root().generation_id.get() == self.generation_id
    }
}

impl HTMLMediaElementContext {
    fn new(elem: &HTMLMediaElement) -> HTMLMediaElementContext {
        HTMLMediaElementContext {
            elem: Trusted::new(elem),
            data: vec![],
            metadata: None,
            generation_id: elem.generation_id.get(),
            next_progress_event: time::get_time() + Duration::milliseconds(350),
            have_metadata: false,
            ignore_response: false,
        }
    }

    fn check_metadata(&mut self, elem: &HTMLMediaElement) {
        if audio_video_metadata::get_format_from_slice(&self.data).is_ok() {
            // Step 6.
            elem.change_ready_state(ReadyState::HaveMetadata);
            self.have_metadata = true;
        }
    }
}
