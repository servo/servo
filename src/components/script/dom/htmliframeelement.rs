/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLIFrameElementBinding;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLIFrameElementDerived, HTMLElementCast};
use dom::bindings::error::ErrorResult;
use dom::bindings::js::JS;
use dom::bindings::trace::Untraceable;
use dom::document::Document;
use dom::element::{HTMLIFrameElementTypeId, Element};
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use dom::virtualmethods::VirtualMethods;
use dom::window::Window;
use servo_util::str::DOMString;

use servo_msg::constellation_msg::{PipelineId, SubpageId};
use std::ascii::StrAsciiExt;
use url::Url;

enum SandboxAllowance {
    AllowNothing = 0x00,
    AllowSameOrigin = 0x01,
    AllowTopNavigation = 0x02,
    AllowForms = 0x04,
    AllowScripts = 0x08,
    AllowPointerLock = 0x10,
    AllowPopups = 0x20
}

#[deriving(Encodable)]
pub struct HTMLIFrameElement {
    pub htmlelement: HTMLElement,
    frame: Untraceable<Option<Url>>,
    pub size: Option<IFrameSize>,
    pub sandbox: Option<u8>
}

impl HTMLIFrameElementDerived for EventTarget {
    fn is_htmliframeelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLIFrameElementTypeId)) => true,
            _ => false
        }
    }
}

#[deriving(Encodable)]
pub struct IFrameSize {
    pub pipeline_id: PipelineId,
    pub subpage_id: SubpageId,
}

impl HTMLIFrameElement {
    pub fn is_sandboxed(&self) -> bool {
        self.sandbox.is_some()
    }

    pub fn set_frame(&mut self, frame: Url) {
        *self.frame = Some(frame);
    }
}

impl HTMLIFrameElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLIFrameElement {
        HTMLIFrameElement {
            htmlelement: HTMLElement::new_inherited(HTMLIFrameElementTypeId, localName, document),
            frame: Untraceable::new(None),
            size: None,
            sandbox: None,
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLIFrameElement> {
        let element = HTMLIFrameElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLIFrameElementBinding::Wrap)
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

    pub fn Sandbox(&self, abstract_self: &JS<HTMLIFrameElement>) -> DOMString {
        let element: JS<Element> = ElementCast::from(abstract_self);
        element.get_string_attribute("sandbox")
    }

    pub fn SetSandbox(&mut self, abstract_self: &mut JS<HTMLIFrameElement>, sandbox: DOMString) {
        let mut element: JS<Element> = ElementCast::from(abstract_self);
        element.set_string_attribute("sandbox", sandbox);
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

    pub fn GetContentDocument(&self) -> Option<JS<Document>> {
        None
    }

    pub fn GetContentWindow(&self) -> Option<JS<Window>> {
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

    pub fn GetSVGDocument(&self) -> Option<JS<Document>> {
        None
    }
}

impl VirtualMethods for JS<HTMLIFrameElement> {
    fn super_type(&self) -> Option<~VirtualMethods:> {
        let htmlelement: JS<HTMLElement> = HTMLElementCast::from(self);
        Some(~htmlelement as ~VirtualMethods:)
    }

    fn after_set_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.after_set_attr(name.clone(), value.clone()),
            _ => (),
        }

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
            self.get_mut().sandbox = Some(modes);
        }
    }

    fn before_remove_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.before_remove_attr(name.clone(), value),
            _ => (),
        }

        if "sandbox" == name {
            self.get_mut().sandbox = None;
        }
    }
}
