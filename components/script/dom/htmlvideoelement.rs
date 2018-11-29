/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::HTMLVideoElementBinding;
use crate::dom::bindings::codegen::Bindings::HTMLVideoElementBinding::HTMLVideoElementMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::htmlmediaelement::{HTMLMediaElement, ReadyState};
use crate::dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use std::cell::Cell;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[dom_struct]
pub struct HTMLVideoElement {
    htmlmediaelement: HTMLMediaElement,
    /// https://html.spec.whatwg.org/multipage/#dom-video-videowidth
    video_width: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#dom-video-videoheight
    video_height: Cell<u32>,
}

impl HTMLVideoElement {
    #[allow(unrooted_must_root)]
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLVideoElement {
        HTMLVideoElement {
            htmlmediaelement: HTMLMediaElement::new_inherited(local_name, prefix, document),
            video_width: Cell::new(DEFAULT_WIDTH),
            video_height: Cell::new(DEFAULT_HEIGHT),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLVideoElement> {
        Node::reflect_node(
            Box::new(HTMLVideoElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            HTMLVideoElementBinding::Wrap,
        )
    }

    pub fn get_video_width(&self) -> u32 {
        self.video_width.get()
    }

    pub fn set_video_width(&self, width: u32) {
        self.video_width.set(width);
    }

    pub fn get_video_height(&self) -> u32 {
        self.video_height.get()
    }

    pub fn set_video_height(&self, height: u32) {
        self.video_height.set(height);
    }
}

impl HTMLVideoElementMethods for HTMLVideoElement {
    // https://html.spec.whatwg.org/multipage/#dom-video-videowidth
    fn VideoWidth(&self) -> u32 {
        if self.htmlmediaelement.get_ready_state() == ReadyState::HaveNothing {
            return 0;
        }
        self.video_width.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-video-videoheight
    fn VideoHeight(&self) -> u32 {
        if self.htmlmediaelement.get_ready_state() == ReadyState::HaveNothing {
            return 0;
        }
        self.video_height.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-video-poster
    make_getter!(Poster, "poster");

    // https://html.spec.whatwg.org/multipage/#dom-video-poster
    make_setter!(SetPoster, "poster");
}
