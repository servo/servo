/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLIFrameElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult, null_str_as_empty};
use dom::document::AbstractDocument;
use dom::element::HTMLIframeElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};
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
        do future_chan.map |future_chan| {
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
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLIFrameElement {
        HTMLIFrameElement {
            htmlelement: HTMLElement::new_inherited(HTMLIframeElementTypeId, localName, document),
            frame: None,
            size: None,
            sandbox: None,
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLIFrameElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLIFrameElementBinding::Wrap)
    }
}

impl HTMLIFrameElement {
    pub fn Src(&self) -> Option<DOMString> {
        None
    }

    pub fn SetSrc(&mut self, _src: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Srcdoc(&self) -> Option<DOMString> {
        None
    }

    pub fn SetSrcdoc(&mut self, _srcdoc: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> Option<DOMString> {
        None
    }

    pub fn SetName(&mut self, _name: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Sandbox(&self, _abstract_self: AbstractNode<ScriptView>) -> Option<DOMString> {
        self.htmlelement.element.GetAttribute(&Some(~"sandbox"))
    }

    pub fn SetSandbox(&mut self, abstract_self: AbstractNode<ScriptView>, sandbox: &Option<DOMString>) {
        self.htmlelement.element.SetAttribute(abstract_self, &Some(~"sandbox"), sandbox);
    }

    pub fn AfterSetAttr(&mut self, name: &Option<DOMString>, value: &Option<DOMString>) {
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

    pub fn Width(&self) -> Option<DOMString> {
        None
    }

    pub fn SetWidth(&mut self, _width: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Height(&self) -> Option<DOMString> {
        None
    }

    pub fn SetHeight(&mut self, _height: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn GetContentDocument(&self) -> Option<AbstractDocument> {
        None
    }

    pub fn GetContentWindow(&self) -> Option<@mut WindowProxy> {
        None
    }

    pub fn Align(&self) -> Option<DOMString> {
        None
    }

    pub fn SetAlign(&mut self, _align: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Scrolling(&self) -> Option<DOMString> {
        None
    }

    pub fn SetScrolling(&mut self, _scrolling: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn FrameBorder(&self) -> Option<DOMString> {
        None
    }

    pub fn SetFrameBorder(&mut self, _frameborder: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn LongDesc(&self) -> Option<DOMString> {
        None
    }

    pub fn SetLongDesc(&mut self, _longdesc: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn MarginHeight(&self) -> Option<DOMString> {
        None
    }

    pub fn SetMarginHeight(&mut self, _marginheight: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn MarginWidth(&self) -> Option<DOMString> {
        None
    }

    pub fn SetMarginWidth(&mut self, _marginwidth: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn GetSVGDocument(&self) -> Option<AbstractDocument> {
        None
    }
}
