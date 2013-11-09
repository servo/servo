/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLObjectElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLObjectElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};
use dom::validitystate::ValidityState;
use dom::windowproxy::WindowProxy;

pub struct HTMLObjectElement {
    htmlelement: HTMLElement
}

impl HTMLObjectElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLObjectElement {
        HTMLObjectElement {
            htmlelement: HTMLElement::new_inherited(HTMLObjectElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLObjectElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLObjectElementBinding::Wrap)
    }
}

impl HTMLObjectElement {
    pub fn Data(&self) -> Option<DOMString> {
        None
    }

    pub fn SetData(&mut self, _data: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> Option<DOMString> {
        None
    }

    pub fn SetType(&mut self, _type: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> Option<DOMString> {
        None
    }

    pub fn SetName(&mut self, _name: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn UseMap(&self) -> Option<DOMString> {
        None
    }

    pub fn SetUseMap(&mut self, _use_map: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn GetForm(&self) -> Option<AbstractNode<ScriptView>> {
        None
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

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn Validity(&self) -> @mut ValidityState {
        let global = self.htmlelement.element.node.owner_doc().document().window;
        ValidityState::new(global)
    }

    pub fn ValidationMessage(&self) -> Option<DOMString> {
        None
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&mut self, _error: &Option<DOMString>) {
    }

    pub fn Align(&self) -> Option<DOMString> {
        None
    }

    pub fn SetAlign(&mut self, _align: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Archive(&self) -> Option<DOMString> {
        None
    }

    pub fn SetArchive(&mut self, _archive: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Code(&self) -> Option<DOMString> {
        None
    }

    pub fn SetCode(&mut self, _code: &Option<DOMString>) -> ErrorResult {
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

    pub fn Standby(&self) -> Option<DOMString> {
        None
    }

    pub fn SetStandby(&mut self, _standby: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Vspace(&self) -> u32 {
        0
    }

    pub fn SetVspace(&mut self, _vspace: u32) -> ErrorResult {
        Ok(())
    }

    pub fn CodeBase(&self) -> Option<DOMString> {
        None
    }

    pub fn SetCodeBase(&mut self, _codebase: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn CodeType(&self) -> Option<DOMString> {
        None
    }

    pub fn SetCodeType(&mut self, _codetype: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Border(&self) -> Option<DOMString> {
        None
    }

    pub fn SetBorder(&mut self, _border: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn GetSVGDocument(&self) -> Option<AbstractDocument> {
        None
    }
}
