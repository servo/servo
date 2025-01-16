/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLTrackElementBinding::{
    HTMLTrackElementConstants, HTMLTrackElementMethods,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::dom::texttrack::TextTrack;
use crate::script_runtime::CanGc;

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
#[repr(u16)]
#[allow(unused)]
pub(crate) enum ReadyState {
    None = HTMLTrackElementConstants::NONE,
    Loading = HTMLTrackElementConstants::LOADING,
    Loaded = HTMLTrackElementConstants::LOADED,
    Error = HTMLTrackElementConstants::ERROR,
}

#[dom_struct]
pub(crate) struct HTMLTrackElement {
    htmlelement: HTMLElement,
    ready_state: ReadyState,
    track: Dom<TextTrack>,
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
            ready_state: ReadyState::None,
            track: Dom::from_ref(track),
        }
    }

    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLTrackElement> {
        let track = TextTrack::new(
            document.window(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            None,
        );
        Node::reflect_node_with_proto(
            Box::new(HTMLTrackElement::new_inherited(
                local_name, prefix, document, &track,
            )),
            document,
            proto,
            can_gc,
        )
    }
}

impl HTMLTrackElementMethods<crate::DomTypeHolder> for HTMLTrackElement {
    // https://html.spec.whatwg.org/multipage/#dom-track-kind
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

    // https://html.spec.whatwg.org/multipage/#dom-track-readystate
    fn ReadyState(&self) -> u16 {
        self.ready_state as u16
    }

    // https://html.spec.whatwg.org/multipage/#dom-track-track
    fn Track(&self) -> DomRoot<TextTrack> {
        DomRoot::from_ref(&*self.track)
    }
}
