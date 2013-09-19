/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, ScriptView};
use dom::validitystate::ValidityState;
use dom::windowproxy::WindowProxy;

pub struct HTMLObjectElement {
    htmlelement: HTMLElement
}

impl HTMLObjectElement {
    pub fn Data(&self) -> DOMString {
        None
    }

    pub fn SetData(&mut self, _data: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        None
    }

    pub fn SetType(&mut self, _type: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn UseMap(&self) -> DOMString {
        None
    }

    pub fn SetUseMap(&mut self, _use_map: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetForm(&self) -> Option<AbstractNode<ScriptView>> {
        None
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

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn Validity(&self) -> @mut ValidityState {
        @mut ValidityState::valid()
    }

    pub fn ValidationMessage(&self) -> DOMString {
        None
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&mut self, _error: &DOMString) {
    }

    pub fn Align(&self) -> DOMString {
        None
    }

    pub fn SetAlign(&mut self, _align: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Archive(&self) -> DOMString {
        None
    }

    pub fn SetArchive(&mut self, _archive: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Code(&self) -> DOMString {
        None
    }

    pub fn SetCode(&mut self, _code: &DOMString) -> ErrorResult {
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
        None
    }

    pub fn SetStandby(&mut self, _standby: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Vspace(&self) -> u32 {
        0
    }

    pub fn SetVspace(&mut self, _vspace: u32) -> ErrorResult {
        Ok(())
    }

    pub fn CodeBase(&self) -> DOMString {
        None
    }

    pub fn SetCodeBase(&mut self, _codebase: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CodeType(&self) -> DOMString {
        None
    }

    pub fn SetCodeType(&mut self, _codetype: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Border(&self) -> DOMString {
        None
    }

    pub fn SetBorder(&mut self, _border: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetSVGDocument(&self) -> Option<AbstractDocument> {
        None
    }
}
