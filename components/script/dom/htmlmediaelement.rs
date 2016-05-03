/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::LoadType;
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementMethods;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementConstants::*;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::refcounted::Trusted;
use dom::document::Document;
use dom::element::{Element, AttributeMutation};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlelement::HTMLElement;
use dom::htmlsourceelement::HTMLSourceElement;
use dom::node::{window_from_node, document_from_node};
use dom::virtualmethods::VirtualMethods;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::{AsyncResponseListener, AsyncResponseTarget, Metadata, NetworkError};
use network_listener::{NetworkListener, PreInvoke};
use script_runtime::ScriptChan;
use script_thread::{Runnable, ScriptThread};
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use string_cache::Atom;
use task_source::dom_manipulation::DOMManipulationTask;
use task_source::TaskSource;
use time::{self, Timespec, Duration};
use url::Url;
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
    /// True if this response is invalid and should be ignored.
    ignore_response: bool,
}

// https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
impl AsyncResponseListener for HTMLMediaElementContext {
    // https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
    fn headers_available(&mut self, metadata: Result<Metadata, NetworkError>) {
        self.metadata = metadata.ok();

        // => "If the media data cannot be fetched at all..."
        let is_failure = self.metadata
                             .as_ref()
                             .and_then(|m| m.status
                                            .as_ref()
                                            .map(|s| s.0 < 200 || s.0 >= 300))
                             .unwrap_or(false);
        if is_failure {
            // Ensure that the element doesn't receive any further notifications
            // of the aborted fetch. The dedicated failure steps will be executed
            // when response_complete runs.
            self.ignore_response = true;
        }
    }

    fn data_available(&mut self, payload: Vec<u8>) {
        if self.ignore_response {
            return;
        }

        let mut payload = payload;
        self.data.append(&mut payload);

        let elem = self.elem.root();

        if !self.have_metadata {
            //TODO: actually check if the payload contains the full metadata
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

    // https://html.spec.whatwg.org/multipage/#media-data-processing-steps-list
    fn response_complete(&mut self, status: Result<(), NetworkError>) {
        let elem = self.elem.root();

        if status.is_ok() {
            elem.change_ready_state(HAVE_ENOUGH_DATA);

            elem.fire_simple_event("progress");

            elem.network_state.set(NETWORK_IDLE);

            elem.fire_simple_event("suspend");
        } else {
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
    fn new(elem: &HTMLMediaElement, url: Url) -> HTMLMediaElementContext {
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
}

#[dom_struct]
pub struct HTMLMediaElement {
    htmlelement: HTMLElement,
    network_state: Cell<u16>,
    ready_state: Cell<u16>,
    current_src: DOMRefCell<String>,
    generation_id: Cell<u32>,
    first_data_load: Cell<bool>,
}

impl HTMLMediaElement {
    pub fn new_inherited(tag_name: Atom,
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
        }
    }

    #[inline]
    pub fn htmlelement(&self) -> &HTMLElement {
        &self.htmlelement
    }

    fn queue_fire_simple_event(&self, type_: &'static str) {
        let win = window_from_node(self);
        let task = FireSimpleEventTask::new(self, type_);
        let _ = win.dom_manipulation_task_source().queue(DOMManipulationTask::MediaTask(box task));
    }

    fn fire_simple_event(&self, type_: &str) {
        let window = window_from_node(self);
        let event = Event::new(GlobalRef::Window(&*window),
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

                    //TODO: check paused state
                    self.queue_fire_simple_event("playing");
                }

                // TODO: autoplay-related logic

                self.queue_fire_simple_event("canplaythrough");
            }

            _ => (),
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn invoke_resource_selection_algorithm(&self, base_url: Url) {
        // Step 1
        self.network_state.set(NETWORK_NO_SOURCE);

        // TODO step 2 (show poster)
        // TODO step 3 (delay load event)

        // Step 4
        ScriptThread::await_stable_state(ResourceSelectionTask::new(self, base_url));
    }

    // https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
    fn resource_selection_algorithm_sync(&self, base_url: Url) {
        // TODO step 5 (populate pending text tracks)

        // Step 6
        let mode = if false {
            // TODO media provider object
            ResourceSelectionMode::Object
        } else if let Some(attr) = self.upcast::<Element>().get_attribute(&ns!(), &atom!("src")) {
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
                    // TODO failed with attribute
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
            // TODO 4.1 (preload=none)

            // 4.2
            let context = Arc::new(Mutex::new(HTMLMediaElementContext::new(self, url.clone())));
            let (action_sender, action_receiver) = ipc::channel().unwrap();
            let script_chan = window_from_node(self).networking_task_source();
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
        let _ = window_from_node(self).dom_manipulation_task_source().queue(
            DOMManipulationTask::MediaTask(box DedicatedMediaSourceFailureTask::new(self)));
    }

    // https://html.spec.whatwg.org/multipage/#dedicated-media-source-failure-steps
    fn dedicated_media_source_failure(&self) {
        // TODO step 1 (error attribute)

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

            // TODO 4.5 (paused)
            // TODO 4.6 (seeking)
            // TODO 4.7 (playback position)
            // TODO 4.8 (timeline offset)
            // TODO 4.9 (duration)
        }

        // TODO step 5 (playback rate)
        // TODO step 6 (error/autoplaying)

        // Step 7
        let doc = document_from_node(self);
        self.invoke_resource_selection_algorithm(doc.base_url());

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

    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_url_getter!(Src, "src");
    // https://html.spec.whatwg.org/multipage/#dom-media-src
    make_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-media-currentsrc
    fn CurrentSrc(&self) -> DOMString {
        DOMString::from(self.current_src.borrow().clone())
    }
}

impl VirtualMethods for HTMLMediaElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        match attr.local_name() {
            &atom!("src") => {
                if mutation.new_value(attr).is_some() {
                    self.media_element_load_algorithm();
                }
            }
            _ => (),
        };
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
        ResourceSelectionTask {
            elem: Trusted::new(elem),
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
    Url(Url),
}
