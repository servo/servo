/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use content_security_policy::Destination;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::context::JSContext;
use js::rust::HandleObject;
use net_traits::request::{CorsSettings, RequestId};
use net_traits::{FetchMetadata, NetworkError, ResourceFetchTiming};
use script_bindings::cell::DomRefCell;
use servo_url::ServoUrl;
use servo_webvtt::{IncrementalWebVTTParser, WebVttCue, WebVttParserSink};

use crate::dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLTrackElementBinding::{
    HTMLTrackElementConstants, HTMLTrackElementMethods,
};
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::TextTrackBinding::{TextTrackMethods, TextTrackMode};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::csp::Violation;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::element::storage::AttrRef;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlmediaelement::HTMLMediaElement;
use crate::dom::node::node::NodeTraits;
use crate::dom::node::{BindContext, MoveContext, Node, UnbindContext};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::security::csp::GlobalCspReporting;
use crate::dom::texttrack::TextTrack;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::webvtt::vttcue::VTTCue;
use crate::dom::{AttributeMutation, cors_setting_for_element};
use crate::fetch::{RequestWithGlobalScope, create_a_potential_cors_request};
use crate::microtask::MicrotaskRunnable;
use crate::network_listener::{FetchResponseListener, ResourceTimingListener};
use crate::realms::enter_auto_realm;
use crate::{ScriptThread, network_listener};

#[derive(Clone, Copy, Default, JSTraceable, MallocSizeOf, PartialEq)]
#[repr(u16)]
/// <https://html.spec.whatwg.org/multipage/#text-track-readiness-state>
pub(crate) enum TextTrackReadinessState {
    /// <https://html.spec.whatwg.org/multipage/#text-track-not-loaded>
    #[default]
    None = HTMLTrackElementConstants::NONE,
    /// <https://html.spec.whatwg.org/multipage/#text-track-loading>
    Loading = HTMLTrackElementConstants::LOADING,
    /// <https://html.spec.whatwg.org/multipage/#text-track-loaded>
    Loaded = HTMLTrackElementConstants::LOADED,
    /// <https://html.spec.whatwg.org/multipage/#text-track-failed-to-load>
    FailedToLoad = HTMLTrackElementConstants::ERROR,
}

#[dom_struct]
pub(crate) struct HTMLTrackElement {
    htmlelement: HTMLElement,
    /// <https://html.spec.whatwg.org/multipage/#text-track-readiness-state>
    readiness_state: Cell<TextTrackReadinessState>,
    /// <https://html.spec.whatwg.org/multipage/#text-track>
    track: Dom<TextTrack>,
    /// <https://html.spec.whatwg.org/multipage/#track-url>
    #[no_trace]
    track_url: DomRefCell<Option<ServoUrl>>,
    /// Used as part of
    /// <https://html.spec.whatwg.org/multipage/#start-the-track-processing-model>
    /// whether the algorithm is running or not.
    is_running_processing_model_algorithm: Cell<bool>,
}

impl HTMLTrackElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        track: &TextTrack,
    ) -> HTMLTrackElement {
        HTMLTrackElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            readiness_state: Default::default(),
            track: Dom::from_ref(track),
            track_url: Default::default(),
            is_running_processing_model_algorithm: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLTrackElement> {
        let track = TextTrack::new(
            cx,
            document.window(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            None,
        );
        let track_element = Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLTrackElement::new_inherited(
                local_name, prefix, document, &track,
            )),
            document,
            proto,
        );
        track.set_associated_track(&track_element);
        track_element
    }

    /// <https://html.spec.whatwg.org/multipage/#start-the-track-processing-model>
    pub(crate) fn start_the_track_processing_model(&self, cx: &mut JSContext) {
        // Step 1. If another occurrence of this algorithm is already running
        // for this text track and its track element, return,
        // letting that other algorithm take care of this element.
        if self.is_running_processing_model_algorithm.get() {
            return;
        }
        // Step 2. If the text track's text track mode is not set to one of hidden or showing, then return.
        if !matches!(
            self.track.Mode(),
            TextTrackMode::Hidden | TextTrackMode::Showing
        ) {
            return;
        }
        // Step 3. If the text track's track element does not have a media element as a parent, return.
        let Some(parent) = self
            .upcast::<Node>()
            .GetParentElement()
            .filter(|parent| parent.is::<HTMLMediaElement>())
        else {
            return;
        };
        // Step 4. Run the remainder of these steps in parallel, allowing whatever caused these steps to run to continue.
        // Step 5. Top: Await a stable state. The synchronous section consists of the following steps.
        // (The steps in the synchronous section are marked with ⌛.)
        // Step 6. ⌛ Set the text track readiness state to loading.
        self.readiness_state.set(TextTrackReadinessState::Loading);
        // Step 7. ⌛ Let URL be the track URL of the track element.
        let url = self.track_url.borrow().clone();
        // Step 8. ⌛ If the track element's parent is a media element,
        // then let corsAttributeState be the state of the parent media element's
        // crossorigin content attribute. Otherwise, let corsAttributeState be No CORS.
        let cors_attribute_state = cors_setting_for_element(&parent);
        let task = TrackElementMicrotask::ProcessingModel {
            elem: DomRoot::from_ref(self),
            cors_attribute_state,
            url,
        };
        self.is_running_processing_model_algorithm.set(true);

        ScriptThread::await_stable_state(cx, Box::new(task));
    }

    fn check_if_track_parent_element_changed(&self, cx: &mut JSContext) {
        if let Some(parent) = self
            .upcast::<Node>()
            .GetParentNode()
            .and_then(DomRoot::downcast::<HTMLMediaElement>)
        {
            // https://html.spec.whatwg.org/multipage/#sourcing-out-of-band-text-tracks
            // > When a track element's parent element changes and the new parent is a media element,
            // > then the user agent must add the track element's corresponding text track to
            // > the media element's list of text tracks, and then queue a media element task
            // > given the media element to fire an event named addtrack at the media element's
            // > textTracks attribute's TextTrackList object, using TrackEvent,
            // > with the track attribute initialized to the text track's TextTrack object.
            parent.TextTracks(cx).add(&parent, &self.track);

            // https://html.spec.whatwg.org/multipage/#sourcing-out-of-band-text-tracks:start-the-track-processing-model
            // > The track element's parent element changes and the new parent is a media element.
            self.start_the_track_processing_model(cx);
        }
    }
}

impl HTMLTrackElementMethods<crate::DomTypeHolder> for HTMLTrackElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-track-kind>
    fn Kind(&self) -> DOMString {
        let element = self.upcast::<Element>();
        // Get the value of "kind" and transform all uppercase
        // chars into lowercase.
        let kind = element
            .get_string_attribute(&local_name!("kind"))
            .to_lowercase();
        match &*kind {
            "subtitles" | "captions" | "descriptions" | "chapters" | "metadata" => {
                // The value of "kind" is valid. Return the lowercase version
                // of it.
                DOMString::from(kind)
            },
            _ if kind.is_empty() => {
                // The default value should be "subtitles". If "kind" has not
                // been set, the real value for "kind" is "subtitles"
                DOMString::from("subtitles")
            },
            _ => {
                // If "kind" has been set but it is not one of the valid
                // values, return the default invalid value of "metadata"
                DOMString::from("metadata")
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-track-kind
    // Do no transformations on the value of "kind" when setting it.
    // All transformations should be done in the get method.
    make_setter!(SetKind, "kind");

    // https://html.spec.whatwg.org/multipage/#dom-track-src
    make_url_getter!(Src, "src");
    // https://html.spec.whatwg.org/multipage/#dom-track-src
    make_url_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-track-srclang
    make_getter!(Srclang, "srclang");
    // https://html.spec.whatwg.org/multipage/#dom-track-srclang
    make_setter!(SetSrclang, "srclang");

    // https://html.spec.whatwg.org/multipage/#dom-track-label
    make_getter!(Label, "label");
    // https://html.spec.whatwg.org/multipage/#dom-track-label
    make_setter!(SetLabel, "label");

    // https://html.spec.whatwg.org/multipage/#dom-track-default
    make_bool_getter!(Default, "default");
    // https://html.spec.whatwg.org/multipage/#dom-track-default
    make_bool_setter!(SetDefault, "default");

    /// <https://html.spec.whatwg.org/multipage/#dom-track-readystate>
    fn ReadyState(&self) -> u16 {
        self.readiness_state.get() as u16
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-track-track>
    fn Track(&self) -> DomRoot<TextTrack> {
        DomRoot::from_ref(&*self.track)
    }
}

impl VirtualMethods for HTMLTrackElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(
        &self,
        cx: &mut JSContext,
        attr: AttrRef<'_>,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);
        match *attr.local_name() {
            local_name!("src") => {
                // https://html.spec.whatwg.org/multipage/#attr-track-src
                // > When the element's src attribute is set, run these steps:
                if matches!(mutation, AttributeMutation::Set(..)) {
                    // Step 2. Let value be the element's src attribute value.
                    let value = &**attr.value();
                    // Step 1. Let trackURL be failure.
                    // Step 3. If value is not the empty string,
                    // then set trackURL to the result of encoding-parsing-and-serializing
                    // a URL given value, relative to the element's node document.
                    // Step 4. Set the element's track URL to trackURL if it is not failure;
                    // otherwise to the empty string.
                    *self.track_url.borrow_mut() = if !value.is_empty() {
                        self.owner_document().base_url().join(value).ok()
                    } else {
                        None
                    };
                }
                // https://html.spec.whatwg.org/multipage/#sourcing-out-of-band-text-tracks
                // > Whenever a track element has its src attribute set, changed, or removed,
                // > the user agent must immediately empty the element's text track's text track list of cues.
                // > (This also causes the algorithm above to stop adding cues from the resource
                // > being obtained using the previously given URL, if any.)
                self.track.empty_cue_list();
            },
            _ => {},
        }
    }

    fn moving_steps(&self, cx: &mut JSContext, context: &MoveContext) {
        if let Some(super_type) = self.super_type() {
            super_type.moving_steps(cx, context);
        }

        if let Some(parent) = context
            .old_parent
            .and_then(|node| node.downcast::<HTMLMediaElement>())
        {
            // https://html.spec.whatwg.org/multipage/#sourcing-out-of-band-text-tracks
            // > When a track element's parent element changes and the old parent was a media element,
            // > then the user agent must remove the track element's corresponding text track from
            // > the media element's list of text tracks, and then queue a media element task
            // > given the media element to fire an event named removetrack at the media element's
            // > textTracks attribute's TextTrackList object, using TrackEvent,
            // > with the track attribute initialized to the text track's TextTrack object.
            parent.TextTracks(cx).remove(&self.track);
        }

        self.check_if_track_parent_element_changed(cx);
    }

    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        if let Some(super_type) = self.super_type() {
            super_type.bind_to_tree(cx, context);
        }

        self.check_if_track_parent_element_changed(cx);
    }

    fn unbind_from_tree(&self, cx: &mut JSContext, context: &UnbindContext) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(cx, context);
        }

        if let Some(parent) = context.parent.downcast::<HTMLMediaElement>() {
            // https://html.spec.whatwg.org/multipage/#sourcing-out-of-band-text-tracks
            // > When a track element's parent element changes and the old parent was a media element,
            // > then the user agent must remove the track element's corresponding text track from
            // > the media element's list of text tracks, and then queue a media element task
            // > given the media element to fire an event named removetrack at the media element's
            // > textTracks attribute's TextTrackList object, using TrackEvent,
            // > with the track attribute initialized to the text track's TextTrack object.
            parent.TextTracks(cx).remove(&self.track);
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum TrackElementMicrotask {
    ProcessingModel {
        elem: DomRoot<HTMLTrackElement>,
        #[no_trace]
        cors_attribute_state: Option<CorsSettings>,
        #[no_trace]
        url: Option<ServoUrl>,
    },
}

impl MicrotaskRunnable for TrackElementMicrotask {
    fn handler(&self, cx: &mut JSContext) {
        let _realm = match self {
            TrackElementMicrotask::ProcessingModel { elem, .. } => enter_auto_realm(cx, &**elem),
        };
        match self {
            // https://html.spec.whatwg.org/multipage/#start-the-track-processing-model
            TrackElementMicrotask::ProcessingModel {
                elem,
                cors_attribute_state,
                url,
            } => {
                // Step 9. End the synchronous section, continuing the remaining steps in parallel.
                // TODO
                // Step 10. If URL is not the empty string:
                if let Some(url) = url {
                    // Step 10.1. Let request be the result of creating a potential-CORS request given URL,
                    // "track", and corsAttributeState, and with the same-origin fallback flag set.
                    let global = elem.global();
                    let document = elem.owner_document();
                    let request = create_a_potential_cors_request(
                        Some(document.webview_id()),
                        url.clone(),
                        Destination::Track,
                        *cors_attribute_state,
                        None,
                        global.get_referrer(),
                    )
                    // Step 10.2. Set request's client to the track element's node document's relevant
                    // settings object.
                    .with_global_scope(&global);
                    // Step 10.3. Set request's initiator type to "track".
                    //
                    // Set in listener

                    // Step 10.4. Fetch request.
                    let listener = HTMLTrackElementFetchListener {
                        element: Trusted::new(elem),
                        url: url.clone(),
                        payload: vec![],
                    };
                    document.fetch_background(request, listener);
                } else {
                    elem.is_running_processing_model_algorithm.set(false);
                }
                // Step 11. Wait until the text track readiness state is no longer set to loading.
                // TODO
                // Step 12. Wait until the track URL is no longer equal to URL,
                // at the same time as the text track mode is set to hidden or showing.
                // TODO
                // Step 13. Jump to the step labeled top.
                // TODO
            },
        }
    }
}

struct TextTrackCueSink {
    track_element: Trusted<HTMLTrackElement>,
}

impl WebVttParserSink<JSContext> for TextTrackCueSink {
    fn consume_cue(&self, cx: &mut JSContext, cue: WebVttCue) {
        let element = self.track_element.root();
        let global = element.global();
        let text_track = &element.track;

        let cue = VTTCue::create_from_vtt(cx, cue, global.as_window(), Some(text_track));
        text_track.get_cues(cx).add(cue.upcast());
    }
}

struct HTMLTrackElementFetchListener {
    /// The element that initiated the request.
    element: Trusted<HTMLTrackElement>,
    /// URL for the resource.
    url: ServoUrl,
    /// The payload received
    payload: Vec<u8>,
}

impl FetchResponseListener for HTMLTrackElementFetchListener {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        _: &mut JSContext,
        _: RequestId,
        _: Result<FetchMetadata, NetworkError>,
    ) {
    }

    fn process_response_chunk(&mut self, _: &mut JSContext, _: RequestId, payload: Vec<u8>) {
        self.payload.extend_from_slice(&payload);
    }

    /// Step 10.4 of <https://html.spec.whatwg.org/multipage/#start-the-track-processing-model>
    fn process_response_eof(
        self,
        cx: &mut JSContext,
        _: RequestId,
        status: Result<(), NetworkError>,
        timing: ResourceFetchTiming,
    ) {
        let track = self.element.clone();
        let element = self.element.root();
        if status.is_err() {
            // > If fetching fails for any reason (network error, the server returns an error code, CORS fails, etc.),
            // > or if URL is the empty string, then queue an element task on the DOM manipulation task source
            // > given the media element to first change the text track readiness state to failed to load
            // > and then fire an event named error at the track element.
            element
                .global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task!(failed_to_load: move |cx| {
                    let track = track.root();
                    track.readiness_state.set(TextTrackReadinessState::FailedToLoad);
                    track.upcast::<EventTarget>().fire_event(cx, atom!("error"));
                }));
        } else {
            // > The tasks queued by the fetching algorithm on the networking task source to
            // > process the data as it is being fetched must determine the type of the resource.
            // > If the type of the resource is not a supported text track format, the load will fail,
            // > as described below. Otherwise, the resource's data must be passed to the appropriate parser
            // > (e.g., the WebVTT parser) as it is received, with the text track list of cues
            // > being used for that parser's output. [WEBVTT]
            let result = str::from_utf8(&self.payload)
                .map_err(|str_error| debug!("WebVTT file contains non-utf8 data: {str_error}"))
                .and_then(|payload| {
                    let sink = TextTrackCueSink {
                        track_element: track.clone(),
                    };
                    IncrementalWebVTTParser::new(sink)
                        .parse_sync(cx, payload)
                        .map_err(|parser_error| {
                            debug!("Failed to parse WEBVTT file: {parser_error}")
                        })
                });
            if result.is_ok() {
                // > If fetching does not fail, and the file was successfully processed,
                // > then the final task that is queued by the networking task source,
                // > after it has finished parsing the data, must change the text track readiness state to loaded,
                // > and fire an event named load at the track element.
                element
                    .global()
                    .task_manager()
                    .networking_task_source()
                    .queue(task!(successfully_loaded: move |cx| {
                        let track = track.root();
                        track.readiness_state.set(TextTrackReadinessState::Loaded);
                        track.upcast::<EventTarget>().fire_event(cx, atom!("load"));
                    }));
            } else {
                // > If fetching does not fail, but the type of the resource is not a supported text track format,
                // > or the file was not successfully processed (e.g., the format in question is an XML format
                // > and the file contained a well-formedness error that XML requires be detected
                // > and reported to the application), then the task that is queued on the networking task source
                // > in which the aforementioned problem is found must change the text track readiness state
                // > to failed to load and fire an event named error at the track element.
                element
                    .global()
                    .task_manager()
                    .networking_task_source()
                    .queue(task!(failed_to_parse: move |cx| {
                        let track = track.root();
                        track.readiness_state.set(TextTrackReadinessState::FailedToLoad);
                        track.upcast::<EventTarget>().fire_event(cx, atom!("error"));
                    }));
            }
        }
        element.is_running_processing_model_algorithm.set(false);
        network_listener::submit_timing(cx, &self, &status, &timing);
    }

    fn process_csp_violations(
        &mut self,
        cx: &mut JSContext,
        _: RequestId,
        violations: Vec<Violation>,
    ) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(cx, violations, None, None);
    }

    fn should_invoke(&self) -> bool {
        true
    }
}

impl ResourceTimingListener for HTMLTrackElementFetchListener {
    /// Step 10.3. of <https://html.spec.whatwg.org/multipage/#start-the-track-processing-model>
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (InitiatorType::Track, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.element.root().owner_document().global()
    }
}
