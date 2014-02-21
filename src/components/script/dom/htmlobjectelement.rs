/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLObjectElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::document::AbstractDocument;
use dom::element::HTMLObjectElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use dom::validitystate::ValidityState;
use dom::windowproxy::WindowProxy;
use servo_util::str::DOMString;

use extra::url::Url;
use servo_net::image_cache_task;
use servo_net::image_cache_task::ImageCacheTask;
use servo_util::url::parse_url;
use servo_util::namespace::Null;
use servo_util::url::is_image_data;

pub struct HTMLObjectElement {
    htmlelement: HTMLElement,
}

impl HTMLObjectElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLObjectElement {
        HTMLObjectElement {
            htmlelement: HTMLElement::new_inherited(HTMLObjectElementTypeId, localName, document),
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLObjectElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLObjectElementBinding::Wrap)
    }
}

impl HTMLObjectElement {

    // Makes the local `data` member match the status of the `data` attribute and starts
    /// prefetching the image. This method must be called after `data` is changed.
    pub fn process_data_url(&mut self, image_cache: ImageCacheTask, url: Option<Url>) {
        let elem = &mut self.htmlelement.element;

        // TODO: support other values
        match (elem.get_attribute(Null, "type").map(|x| x.Value()),
               elem.get_attribute(Null, "data").map(|x| x.Value())) {
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

    pub fn AfterSetAttr(&mut self, name: DOMString, _value: DOMString) {
        if "data" == name {
            let document = self.htmlelement.element.node.owner_doc();
            let window = document.document().window;
            let url = window.page.url.as_ref().map(|&(ref url, _)| url.clone());
            self.process_data_url(window.image_cache_task.clone(), url);
        }
    }

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

    pub fn GetForm(&self) -> Option<AbstractNode> {
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

    pub fn GetContentDocument(&self) -> Option<AbstractDocument> {
        None
    }

    pub fn GetContentWindow(&self) -> Option<@mut WindowProxy> {
        None
    }

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn Validity(&self) -> @mut ValidityState {
        let global = self.htmlelement.element.node.owner_doc().document().window;
        ValidityState::new(global)
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

    pub fn GetSVGDocument(&self) -> Option<AbstractDocument> {
        None
    }
}
