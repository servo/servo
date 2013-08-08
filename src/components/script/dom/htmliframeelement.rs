/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::document::AbstractDocument;
use dom::htmlelement::HTMLElement;
use dom::windowproxy::WindowProxy;
use geom::size::Size2D;

use servo_msg::constellation_msg::SubpageId;

use std::comm::ChanOne;
use extra::url::Url;

pub struct HTMLIFrameElement {
    parent: HTMLElement,
    frame: Option<Url>,
    size: Option<IframeSize>,
}

struct IframeSize {
    pipeline_id: PipelineId,
    subpage_id: SubpageId,
    future_chan: Option<ChanOne<Size2D<uint>>>,
    constellation_chan: ConstellationChan,
}

impl IframeSize {
    pub fn set_rect(&mut self, rect: Rect<f32>) {
        let future_chan = replace(&mut self.future_chan, None);
        do future_chan.map_consume |future_chan| {
            let Size2D { width, height } = rect.size;
            future_chan.send(Size2D(width as uint, height as uint));
        };
        self.constellation_chan.send(FrameRectMsg(self.pipeline_id, self.subpage_id, rect));
    }
}


impl HTMLIFrameElement {
    pub fn Src(&self) -> DOMString {
        null_string
    }

    pub fn SetSrc(&mut self, _src: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Srcdoc(&self) -> DOMString {
        null_string
    }

    pub fn SetSrcdoc(&mut self, _srcdoc: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Name(&self) -> DOMString {
        null_string
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Sandbox(&self) -> DOMString {
        null_string
    }

    pub fn SetSandbox(&self, _sandbox: &DOMString) {
    }

    pub fn AllowFullscreen(&self) -> bool {
        false
    }

    pub fn SetAllowFullscreen(&mut self, _allow: bool, _rv: &mut ErrorResult) {
    }

    pub fn Width(&self) -> DOMString {
        null_string
    }

    pub fn SetWidth(&mut self, _width: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Height(&self) -> DOMString {
        null_string
    }

    pub fn SetHeight(&mut self, _height: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn GetContentDocument(&self) -> Option<AbstractDocument> {
        None
    }

    pub fn GetContentWindow(&self) -> Option<@mut WindowProxy> {
        None
    }

    pub fn Align(&self) -> DOMString {
        null_string
    }

    pub fn SetAlign(&mut self, _align: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Scrolling(&self) -> DOMString {
        null_string
    }

    pub fn SetScrolling(&mut self, _scrolling: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn FrameBorder(&self) -> DOMString {
        null_string
    }

    pub fn SetFrameBorder(&mut self, _frameborder: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn LongDesc(&self) -> DOMString {
        null_string
    }

    pub fn SetLongDesc(&mut self, _longdesc: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn MarginHeight(&self) -> DOMString {
        null_string
    }

    pub fn SetMarginHeight(&mut self, _marginheight: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn MarginWidth(&self) -> DOMString {
        null_string
    }

    pub fn SetMarginWidth(&mut self, _marginwidth: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn GetSVGDocument(&self) -> Option<AbstractDocument> {
        None
    }
}
