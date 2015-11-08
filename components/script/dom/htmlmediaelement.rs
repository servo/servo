/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::LoadType;
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::CanPlayTypeResult;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementMethods;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementConstants::*;
use dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorConstants::*;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, MutNullableHeap, JS};
use dom::bindings::refcounted::Trusted;
use dom::document::Document;
use dom::element::{Element, AttributeMutation};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlelement::HTMLElement;
use dom::htmlsourceelement::HTMLSourceElement;
use dom::mediaerror::MediaError;
use dom::node::{window_from_node, document_from_node, Node};
use dom::virtualmethods::VirtualMethods;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::{AsyncResponseListener, AsyncResponseTarget, Metadata};
use network_listener::{NetworkListener, PreInvoke};
use script_task::{CommonScriptMsg, ScriptTaskEventCategory, Runnable, ScriptChan, ScriptTask};
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use string_cache::Atom;
use time::{self, Timespec, Duration};
use url::{Url, UrlParser};
use util::str::DOMString;

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
    url: Url,
    /// Whether the media metadata has been completely received.
    have_metadata: bool,
}

// https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
impl AsyncResponseListener for HTMLMediaElementContext {
    fn headers_available(&mut self, metadata: Metadata) {
        if metadata.status.as_ref().map(|s| s.0 < 200 && s.0 >= 300).unwrap_or(false) {
            self.elem.root().queue_dedicated_media_source_failure_steps();
            // Ensure that the element doesn't receive any further notifications
            // of the aborted fetch.
            self.generation_id -= 1;
        }

        self.metadata = Some(metadata);
    }

    fn data_available(&mut self, payload: Vec<u8>) {
        let mut payload = payload;
        self.data.append(&mut payload);

        let elem = self.elem.root();

        if !self.have_metadata {
            elem.change_ready_state(HAVE_METADATA);
            self.have_metadata = true;
        } else {
            elem.change_ready_state(HAVE_CURRENT_DATA);
        }

        if time::get_time() > self.next_progress_event {
            elem.queue_fire_simple_event("progress");
            self.next_progress_event = time::get_time() + Duration::milliseconds(350);
        }
    }

    // https://html.spec.whatwg.org/multipage/#loading-the-media-resource:media-resource-17
    fn response_complete(&mut self, status: Result<(), String>) {
        let elem = self.elem.root();

        if status.is_ok() {
            elem.change_ready_state(HAVE_ENOUGH_DATA);

            elem.fire_simple_event("progress");

            elem.network_state.set(NETWORK_IDLE);

            elem.fire_simple_event("suspend");
        } else if elem.ready_state.get() != HAVE_NOTHING {
            elem.error.set(Some(&*MediaError::new(&*window_from_node(&*elem),
                                                  MEDIA_ERR_NETWORK)));

            elem.network_state.set(NETWORK_IDLE);

            // TODO: update delay load flag

            elem.fire_simple_event("error");
        } else {
            elem.queue_dedicated_media_source_failure_steps();
        }

        let document = document_from_node(&*elem);
        document.finish_load(LoadType::Media(self.url.clone()));
    }
}

impl PreInvoke for HTMLMediaElementContext {
    fn should_invoke(&self) -> bool {
        self.elem.root().generation_id.get() == self.generation_id
    }
}

impl HTMLMediaElementContext {
    fn new(elem: &HTMLMediaElement, url: Url) -> HTMLMediaElementContext {
        let win = window_from_node(elem);
        HTMLMediaElementContext {
            elem: Trusted::new(win.get_cx(), elem, win.script_chan()),
            data: vec![],
            metadata: None,
            generation_id: elem.generation_id.get(),
            next_progress_event: time::get_time() + Duration::milliseconds(350),
            url: url,
            have_metadata: false,
        }
    }
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
}

impl HTMLMediaElement {
    pub fn new_inherited(tag_name: DOMString,
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
        }
    }

    #[inline]
    pub fn htmlelement(&self) -> &HTMLElement {
        &self.htmlelement
    }

    // https://html.spec.whatwg.org/multipage/#playing-the-media-resource:internal-pause-steps
    fn internal_pause_steps(&self) {
        // Step 1
        self.autoplaying.set(false);

        // Step 2
        if !self.Paused() {
            // 2.1
            self.paused.set(true);

            // 2.2
            self.queue_fire_simple_event("timeupdate");

            // 2.3
            self.queue_fire_simple_event("pause");

            // TODO 2.4 (official playback position)
        }

        // TODO step 3 (media controller)
    }

    fn queue_fire_simple_event(&self, type_: &'static str) {
        let win = window_from_node(self);
        let task = FireSimpleEventTask::new(self, type_);
        let _ = win.script_chan().send(CommonScriptMsg::RunnableMsg(
            ScriptTaskEventCategory::DomEvent, box task));
    }

    fn fire_simple_event(&self, type_: &str) {
        let window = window_from_node(self);
        let event = Event::new(GlobalRef::Window(&*window),
                               DOMString(type_.to_owned()),
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

        // If the new ready state is HAVE_FUTURE_DATA or HAVE_ENOUGH_DATA,
        // then the relevant steps below must then be run also.
        match (old_ready_state, ready_state) {
            // previous ready state was HAVE_CURRENT_DATA or less, and the new ready
            // state is HAVE_FUTURE_DATA
            (HAVE_CURRENT_DATA, HAVE_FUTURE_DATA) |
            (HAVE_METADATA, HAVE_FUTURE_DATA) |
            (HAVE_NOTHING, HAVE_FUTURE_DATA) => {
                self.queue_fire_simple_event("canplay");

                // TODO: check paused state
                self.queue_fire_simple_event("playing");
            }

            // new ready state is HAVE_ENOUGH_DATA
            (_, HAVE_ENOUGH_DATA) => {
                if old_ready_state <= HAVE_CURRENT_DATA {
                    self.queue_fire_simple_event("canplay");

                    if !self.Paused() {
                        self.queue_fire_simple_event("playing");
                    }
                }

                if self.autoplaying.get() &&
                   self.Paused() &&
                   self.upcast::<Element>().has_attribute(&Atom::from_slice("autoplay")) {
                    self.paused.set(false);
                    // TODO: show poster
                    self.queue_fire_simple_event("play");
                    self.queue_fire_simple_event("playing");
                }

                self.queue_fire_simple_event("canplaythrough");
            }

            _ => (),
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn invoke_resource_selection_algorithm(&self) {
        // Step 1
        self.network_state.set(NETWORK_NO_SOURCE);

        // TODO step 2 (show poster)
        // TODO step 3 (delay load event)

        // Step 4
        let doc = document_from_node(self);
        ScriptTask::await_stable_state(ResourceSelectionTask::new(self, doc.base_url()));
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn resource_selection_algorithm_sync(&self, base_url: Url) {
        // TODO step 5 (populate pending text tracks)

        // Step 6
        let mode = if false {
            // TODO media provider object
            ResourceSelectionMode::Object
        } else if let Some(attr) = self.upcast::<Element>().get_attribute(&ns!(""), &atom!(src)) {
            ResourceSelectionMode::Attribute(attr.Value().0)
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
                let absolute_url = UrlParser::new().base_url(&base_url)
                                                   .parse(&src)
                                                   .map_err(|_| ());

                // Step 3
                if let Ok(url) = absolute_url {
                    *self.current_src.borrow_mut() = url.serialize();
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
            if self.Preload() == "none" && !self.autoplaying.get(){
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
            let script_chan = window_from_node(self).script_chan();
            let listener = box NetworkListener {
                context: context,
                script_chan: script_chan,
            };

            let response_target = AsyncResponseTarget {
                sender: action_sender,
            };
            ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
                listener.notify(message.to().unwrap());
            });

            // FIXME: we're supposed to block the load event much earlier than now
            let doc = document_from_node(self);
            doc.load_async(LoadType::Media(url), response_target);
        } else {
            // TODO local resource fetch
            self.queue_dedicated_media_source_failure_steps();
        }
    }

    fn queue_dedicated_media_source_failure_steps(&self) {
        let _ = window_from_node(self).script_chan().send(
            CommonScriptMsg::RunnableMsg(ScriptTaskEventCategory::DomEvent,
                                         box DedicatedMediaSourceFailureTask::new(self)));
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

        // TODO step 6 (delay load event)
    }

    // https://html.spec.whatwg.org/multipage/#media-element-load-algorithm
    fn media_element_load_algorithm(&self) {
        self.first_data_load.set(true);

        // TODO Step 1 (abort resource selection algorithm instances)

        // Step 2
        self.generation_id.set(self.generation_id.get() + 1);

        // Step 3
        let network_state = self.NetworkState();
        match network_state {
            NETWORK_LOADING |
            NETWORK_IDLE => {
                self.queue_fire_simple_event("abort");
            }
            _ => (),
        }

        // Step 4
        if network_state != NETWORK_EMPTY {
            // 4.1
            self.queue_fire_simple_event("emptied");

            // TODO 4.2 (abort in-progress fetch)

            // TODO 4.3 (forget resource tracks)

            // 4.4
            self.change_ready_state(HAVE_NOTHING);

            if !self.Paused() {
                self.paused.set(true);
            }
            // TODO 4.6 (seeking)
            // TODO 4.7 (playback position)
            // TODO 4.8 (timeline offset)
            // TODO 4.9 (duration)
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
    fn NetworkState(&self) -> u16 {
        self.network_state.get()
    }

    fn ReadyState(&self) -> u16 {
        self.ready_state.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-media-autoplay
    make_bool_getter!(Autoplay, "autoplay");
    // https://html.spec.whatwg.org/multipage/#dom-media-autoplay
    make_bool_setter!(SetAutoplay, "autoplay");

    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_getter!(Src, "src");
    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#attr-media-preload
    make_enumerated_getter!(Preload, "", ("none") | ("metadata") | ("auto"));
    // https://html.spec.whatwg.org/multipage/#attr-media-preload
    make_setter!(SetPreload, "preload");

    // https://html.spec.whatwg.org/multipage/#dom-media-currentsrc
    fn CurrentSrc(&self) -> DOMString {
        DOMString(self.current_src.borrow().clone())
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
        // Step 1
        if self.network_state.get() == NETWORK_EMPTY {
            self.invoke_resource_selection_algorithm();
        }

        // TODO step 2 (seek backwards)

        // TODO step 3 (media controller)

        // Step 4
        if self.Paused() {
            // 4.1
            self.paused.set(false);

            // TODO 4.2 (show poster)

            // 4.3
            self.queue_fire_simple_event("play");

            // 4.4
            let state = self.ready_state.get();
            if state == HAVE_NOTHING ||
               state == HAVE_METADATA ||
               state == HAVE_CURRENT_DATA {
                self.queue_fire_simple_event("waiting");
            } else {
                self.queue_fire_simple_event("playing");
            }

            // 4.5
            self.autoplaying.set(false);

            // TODO 4.6 (media controller)
        }
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
            &atom!(src) => {
                if mutation.new_value(attr).is_some() {
                    self.media_element_load_algorithm();
                }
            }
            _ => (),
        };
    }

    // https://html.spec.whatwg.org/multipage/#playing-the-media-resource:media-element-75
    fn unbind_from_tree(&self, tree_in_doc: bool) {
        self.super_type().unwrap().unbind_from_tree(tree_in_doc);

        if tree_in_doc {
            ScriptTask::await_stable_state(PauseIfNotInDocumentTask::new(self));
        }
    }
}

struct FireSimpleEventTask {
    elem: Trusted<HTMLMediaElement>,
    type_: &'static str,
}

impl FireSimpleEventTask {
    fn new(target: &HTMLMediaElement, type_: &'static str) -> FireSimpleEventTask {
        let win = window_from_node(target);
        FireSimpleEventTask {
            elem: Trusted::new(win.get_cx(), target, win.script_chan()),
            type_: type_,
        }
    }
}

impl Runnable for FireSimpleEventTask {
    fn handler(self: Box<FireSimpleEventTask>) {
        let elem = self.elem.root();
        elem.fire_simple_event(self.type_);
    }
}

struct ResourceSelectionTask {
    elem: Trusted<HTMLMediaElement>,
    base_url: Url,
}

impl ResourceSelectionTask {
    fn new(elem: &HTMLMediaElement, url: Url) -> ResourceSelectionTask {
        let win = window_from_node(elem);
        ResourceSelectionTask {
            elem: Trusted::new(win.get_cx(), elem, win.script_chan()),
            base_url: url,
        }
    }
}

impl Runnable for ResourceSelectionTask {
    fn handler(self: Box<ResourceSelectionTask>) {
        self.elem.root().resource_selection_algorithm_sync(self.base_url);
    }
}

struct DedicatedMediaSourceFailureTask {
    elem: Trusted<HTMLMediaElement>,
}

impl DedicatedMediaSourceFailureTask {
    fn new(elem: &HTMLMediaElement) -> DedicatedMediaSourceFailureTask {
        let win = window_from_node(elem);
        DedicatedMediaSourceFailureTask {
            elem: Trusted::new(win.get_cx(), elem, win.script_chan()),
        }
    }
}

impl Runnable for DedicatedMediaSourceFailureTask {
    fn handler(self: Box<DedicatedMediaSourceFailureTask>) {
        self.elem.root().dedicated_media_source_failure();
    }
}

struct PauseIfNotInDocumentTask {
    elem: Trusted<HTMLMediaElement>,
}

impl PauseIfNotInDocumentTask {
    fn new(elem: &HTMLMediaElement) -> PauseIfNotInDocumentTask {
        let win = window_from_node(elem);
        PauseIfNotInDocumentTask {
            elem: Trusted::new(win.get_cx(), elem, win.script_chan()),
        }
    }
}

impl Runnable for PauseIfNotInDocumentTask {
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
    Url(Url),
}
