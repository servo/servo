/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLVideoElementBinding;
use dom::bindings::codegen::Bindings::HTMLVideoElementBinding::HTMLVideoElementMethods;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::htmlmediaelement::{HTMLMediaElement, ReadyState};
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use std::cell::Cell;

#[dom_struct]
pub struct HTMLVideoElement {
    htmlmediaelement: HTMLMediaElement,
    /// https://html.spec.whatwg.org/multipage/#dom-video-videowidth
    video_width: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#dom-video-videoheight
    video_height: Cell<u32>,
}

impl HTMLVideoElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLVideoElement {
        HTMLVideoElement {
            htmlmediaelement: HTMLMediaElement::new_inherited(local_name, prefix, document),
            video_width: Cell::new(0),
            video_height: Cell::new(0),
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
}
