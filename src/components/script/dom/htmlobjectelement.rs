/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLObjectElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLObjectElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::{Element, HTMLObjectElementTypeId};
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::HTMLFormElement;
use dom::node::{Node, ElementNodeTypeId, NodeHelpers, window_from_node};
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;
use dom::window::Window;
use servo_util::str::DOMString;

use servo_net::image_cache_task;
use servo_net::image_cache_task::ImageCacheTask;
use servo_util::url::parse_url;
use servo_util::namespace::Null;
use servo_util::url::is_image_data;
use url::Url;

#[deriving(Encodable)]
pub struct HTMLObjectElement {
    pub htmlelement: HTMLElement,
}

impl HTMLObjectElementDerived for EventTarget {
    fn is_htmlobjectelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLObjectElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLObjectElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLObjectElement {
        HTMLObjectElement {
            htmlelement: HTMLElement::new_inherited(HTMLObjectElementTypeId, localName, document),
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLObjectElement> {
        let element = HTMLObjectElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLObjectElementBinding::Wrap)
    }
}

trait ProcessDataURL {
    fn process_data_url(&mut self, image_cache: ImageCacheTask, url: Option<Url>);
}

impl ProcessDataURL for JS<HTMLObjectElement> {
    // Makes the local `data` member match the status of the `data` attribute and starts
    /// prefetching the image. This method must be called after `data` is changed.
    fn process_data_url(&mut self, image_cache: ImageCacheTask, url: Option<Url>) {
        let elem: JS<Element> = ElementCast::from(self);

        // TODO: support other values
        match (elem.get_attribute(Null, "type").map(|x| x.get().Value()),
               elem.get_attribute(Null, "data").map(|x| x.get().Value())) {
            (None, Some(uri)) => {
                if is_image_data(uri) {
                    let data_url = parse_url(uri, url);
                    // Issue #84
                    image_cache.send(image_cache_task::Prefetch(data_url));
                }
            }
            _ => { }
        }
    }
}

impl HTMLObjectElement {
    pub fn Data(&self) -> DOMString {
        ~""
    }

    pub fn SetData(&mut self, _data: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn UseMap(&self) -> DOMString {
        ~""
    }

    pub fn SetUseMap(&mut self, _use_map: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetForm(&self) -> Option<JS<HTMLFormElement>> {
        None
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

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn Validity(&self) -> JS<ValidityState> {
        let doc = self.htmlelement.element.node.owner_doc();
        let doc = doc.get();
        ValidityState::new(&doc.window)
    }

    pub fn ValidationMessage(&self) -> DOMString {
        ~""
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&mut self, _error: DOMString) {
    }

    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Archive(&self) -> DOMString {
        ~""
    }

    pub fn SetArchive(&mut self, _archive: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Code(&self) -> DOMString {
        ~""
    }

    pub fn SetCode(&mut self, _code: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Declare(&self) -> bool {
        false
    }

    pub fn SetDeclare(&mut self, _declare: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Hspace(&self) -> u32 {
        0
    }

    pub fn SetHspace(&mut self, _hspace: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Standby(&self) -> DOMString {
        ~""
    }

    pub fn SetStandby(&mut self, _standby: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Vspace(&self) -> u32 {
        0
    }

    pub fn SetVspace(&mut self, _vspace: u32) -> ErrorResult {
        Ok(())
    }

    pub fn CodeBase(&self) -> DOMString {
        ~""
    }

    pub fn SetCodeBase(&mut self, _codebase: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CodeType(&self) -> DOMString {
        ~""
    }

    pub fn SetCodeType(&mut self, _codetype: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Border(&self) -> DOMString {
        ~""
    }

    pub fn SetBorder(&mut self, _border: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetSVGDocument(&self) -> Option<JS<Document>> {
        None
    }
}

impl VirtualMethods for JS<HTMLObjectElement> {
    fn super_type(&self) -> Option<~VirtualMethods:> {
        let htmlelement: JS<HTMLElement> = HTMLElementCast::from(self);
        Some(~htmlelement as ~VirtualMethods:)
    }

    fn after_set_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.after_set_attr(name.clone(), value),
            _ => (),
        }

        if "data" == name {
            let window = window_from_node(self);
            let url = Some(window.get().get_url());
            self.process_data_url(window.get().image_cache_task.clone(), url);
        }
    }
}
