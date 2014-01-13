/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLIFrameElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLIframeElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use dom::windowproxy::WindowProxy;

use extra::url::Url;
use servo_msg::constellation_msg::{PipelineId, SubpageId};
use std::ascii::StrAsciiExt;

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

pub struct IFrameSize {
    pipeline_id: PipelineId,
    subpage_id: SubpageId,
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

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode {
        let element = HTMLIFrameElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLIFrameElementBinding::Wrap)
    }
}

impl HTMLIFrameElement {
    pub fn Src(&self) -> DOMString {
        ~""
    }

    pub fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Srcdoc(&self) -> DOMString {
        ~""
    }

    pub fn SetSrcdoc(&mut self, _srcdoc: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Sandbox(&self, _abstract_self: AbstractNode) -> DOMString {
        self.htmlelement.element.get_string_attribute("sandbox")
    }

    pub fn SetSandbox(&mut self, abstract_self: AbstractNode, sandbox: DOMString) {
        self.htmlelement.element.set_string_attribute(abstract_self, "sandbox",
                                                      sandbox);
    }

    pub fn AfterSetAttr(&mut self, name: DOMString, value: DOMString) {
        if "sandbox" == name {
            let mut modes = AllowNothing as u8;
            for word in value.split(' ') {
                // FIXME: Workaround for https://github.com/mozilla/rust/issues/10683
                let word_lower = word.to_ascii_lower();
                modes |= match word_lower.as_slice() {
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

    pub fn AfterRemoveAttr(&mut self, name: DOMString) {
        if "sandbox" == name {
            self.sandbox = None;
        }
    }

    pub fn AllowFullscreen(&self) -> bool {
        false
    }

    pub fn SetAllowFullscreen(&mut self, _allow: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> DOMString {
        ~""
    }

    pub fn SetWidth(&mut self, _width: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Height(&self) -> DOMString {
        ~""
    }

    pub fn SetHeight(&mut self, _height: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetContentDocument(&self) -> Option<AbstractDocument> {
        None
    }

    pub fn GetContentWindow(&self) -> Option<@mut WindowProxy> {
        None
    }

    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Scrolling(&self) -> DOMString {
        ~""
    }

    pub fn SetScrolling(&mut self, _scrolling: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn FrameBorder(&self) -> DOMString {
        ~""
    }

    pub fn SetFrameBorder(&mut self, _frameborder: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn LongDesc(&self) -> DOMString {
        ~""
    }

    pub fn SetLongDesc(&mut self, _longdesc: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn MarginHeight(&self) -> DOMString {
        ~""
    }

    pub fn SetMarginHeight(&mut self, _marginheight: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn MarginWidth(&self) -> DOMString {
        ~""
    }

    pub fn SetMarginWidth(&mut self, _marginwidth: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetSVGDocument(&self) -> Option<AbstractDocument> {
        None
    }
}
