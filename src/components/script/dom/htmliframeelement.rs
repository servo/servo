/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult, null_str_as_empty};
use dom::document::AbstractDocument;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, ScriptView};
use dom::windowproxy::WindowProxy;
use geom::size::Size2D;
use geom::rect::Rect;

use servo_msg::constellation_msg::{ConstellationChan, FrameRectMsg, PipelineId, SubpageId};

use std::ascii::StrAsciiExt;
use std::comm::ChanOne;
use extra::url::Url;
use std::util::replace;

enum SandboxAllowance {
    AllowNothing = 0x00,
    AllowSameOrigin = 0x01,
    AllowTopNavigation = 0x02,
    AllowForms = 0x04,
    AllowScripts = 0x08,
    AllowPointerLock = 0x10,
    AllowPopups = 0x20
}

pub struct HTMLIFrameElement {
    htmlelement: HTMLElement,
    frame: Option<Url>,
    size: Option<IFrameSize>,
    sandbox: Option<u8>
}

struct IFrameSize {
    pipeline_id: PipelineId,
    subpage_id: SubpageId,
    future_chan: Option<ChanOne<Size2D<uint>>>,
    constellation_chan: ConstellationChan,
}

impl IFrameSize {
    pub fn set_rect(&mut self, rect: Rect<f32>) {
        let future_chan = replace(&mut self.future_chan, None);
        do future_chan.map_move |future_chan| {
            let Size2D { width, height } = rect.size;
            future_chan.send(Size2D(width as uint, height as uint));
        };
        self.constellation_chan.send(FrameRectMsg(self.pipeline_id, self.subpage_id, rect));
    }
}

impl HTMLIFrameElement {
    pub fn is_sandboxed(&self) -> bool {
        self.sandbox.is_some()
    }
}

impl HTMLIFrameElement {
    pub fn Src(&self) -> DOMString {
        None
    }

    pub fn SetSrc(&mut self, _src: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Srcdoc(&self) -> DOMString {
        None
    }

    pub fn SetSrcdoc(&mut self, _srcdoc: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Sandbox(&self, _abstract_self: AbstractNode<ScriptView>) -> DOMString {
        self.htmlelement.element.GetAttribute(&Some(~"sandbox"))
    }

    pub fn SetSandbox(&mut self, abstract_self: AbstractNode<ScriptView>, sandbox: &DOMString) {
        self.htmlelement.element.SetAttribute(abstract_self, &Some(~"sandbox"), sandbox);
    }

    pub fn AfterSetAttr(&mut self, name: &DOMString, value: &DOMString) {
        let name = null_str_as_empty(name);
        if "sandbox" == name {
            let mut modes = AllowNothing as u8;
            let words = null_str_as_empty(value);
            for word in words.split_iter(' ') {
                modes |= match word.to_ascii_lower().as_slice() {
                    "allow-same-origin" => AllowSameOrigin,
                    "allow-forms" => AllowForms,
                    "allow-pointer-lock" => AllowPointerLock,
                    "allow-popups" => AllowPopups,
                    "allow-scripts" => AllowScripts,
                    "allow-top-navigation" => AllowTopNavigation,
                    _ => AllowNothing
                } as u8;
            }
            self.sandbox = Some(modes);
        }
    }

    pub fn AllowFullscreen(&self) -> bool {
        false
    }

    pub fn SetAllowFullscreen(&mut self, _allow: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> DOMString {
        None
    }

    pub fn SetWidth(&mut self, _width: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Height(&self) -> DOMString {
        None
    }

    pub fn SetHeight(&mut self, _height: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetContentDocument(&self) -> Option<AbstractDocument> {
        None
    }

    pub fn GetContentWindow(&self) -> Option<@mut WindowProxy> {
        None
    }

    pub fn Align(&self) -> DOMString {
        None
    }

    pub fn SetAlign(&mut self, _align: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Scrolling(&self) -> DOMString {
        None
    }

    pub fn SetScrolling(&mut self, _scrolling: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn FrameBorder(&self) -> DOMString {
        None
    }

    pub fn SetFrameBorder(&mut self, _frameborder: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn LongDesc(&self) -> DOMString {
        None
    }

    pub fn SetLongDesc(&mut self, _longdesc: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn MarginHeight(&self) -> DOMString {
        None
    }

    pub fn SetMarginHeight(&mut self, _marginheight: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn MarginWidth(&self) -> DOMString {
        None
    }

    pub fn SetMarginWidth(&mut self, _marginwidth: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetSVGDocument(&self) -> Option<AbstractDocument> {
        None
    }
}
