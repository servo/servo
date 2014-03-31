/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLObjectElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLObjectElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::js::{JS, JSRef, RootCollection, Unrooted};
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

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Unrooted<HTMLObjectElement> {
        let element = HTMLObjectElement::new_inherited(localName, document.unrooted());
        Node::reflect_node(~element, document, HTMLObjectElementBinding::Wrap)
    }
}

trait ProcessDataURL {
    fn process_data_url(&mut self, image_cache: ImageCacheTask, url: Option<Url>);
}

impl<'a> ProcessDataURL for JSRef<'a, HTMLObjectElement> {
    // Makes the local `data` member match the status of the `data` attribute and starts
    /// prefetching the image. This method must be called after `data` is changed.
    fn process_data_url(&mut self, image_cache: ImageCacheTask, url: Option<Url>) {
        let roots = RootCollection::new();
        let elem: &JSRef<Element> = ElementCast::from_ref(self);

        // TODO: support other values
        match (elem.get_attribute(Null, "type").map(|x| x.root(&roots).Value()),
               elem.get_attribute(Null, "data").map(|x| x.root(&roots).Value())) {
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

    pub fn GetForm(&self) -> Option<Unrooted<HTMLFormElement>> {
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

    pub fn GetContentDocument(&self) -> Option<Unrooted<Document>> {
        None
    }

    pub fn GetContentWindow(&self) -> Option<Unrooted<Window>> {
        None
    }

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn Validity(&self) -> Unrooted<ValidityState> {
        let roots = RootCollection::new();
        let doc = self.htmlelement.element.node.owner_doc().root(&roots);
        let window = doc.deref().window.root(&roots);
        ValidityState::new(&window.root_ref())
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

    pub fn GetSVGDocument(&self) -> Option<Unrooted<Document>> {
        None
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLObjectElement> {
    fn super_type(&self) -> Option<~VirtualMethods:> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_ref(self);
        Some(~htmlelement.clone() as ~VirtualMethods:)
    }

    fn after_set_attr(&mut self, name: DOMString, value: DOMString) {
        let roots = RootCollection::new();
        match self.super_type() {
            Some(ref mut s) => s.after_set_attr(name.clone(), value),
            _ => (),
        }

        if "data" == name {
            let window = window_from_node(self).root(&roots);
            let url = Some(window.get().get_url());
            self.process_data_url(window.get().image_cache_task.clone(), url);
        }
    }
}
