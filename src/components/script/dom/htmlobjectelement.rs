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
    parent: HTMLElement
}

impl HTMLObjectElement {
    pub fn Data(&self) -> DOMString {
        None
    }

    pub fn SetData(&mut self, _data: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        None
    }

    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn UseMap(&self) -> DOMString {
        None
    }

    pub fn SetUseMap(&mut self, _use_map: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn GetForm(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn Width(&self) -> DOMString {
        None
    }

    pub fn SetWidth(&mut self, _width: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Height(&self) -> DOMString {
        None
    }

    pub fn SetHeight(&mut self, _height: &DOMString, _rv: &mut ErrorResult) {
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

    pub fn SetAlign(&mut self, _align: &DOMString, _rv: &mut ErrorResult) {
    }

        pub fn Archive(&self) -> DOMString {
        None
    }

    pub fn SetArchive(&mut self, _archive: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Code(&self) -> DOMString {
        None
    }

    pub fn SetCode(&mut self, _code: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Declare(&self) -> bool {
        false
    }

    pub fn SetDeclare(&mut self, _declare: bool, _rv: &mut ErrorResult) {
    }

    pub fn Hspace(&self) -> u32 {
        0
    }

    pub fn SetHspace(&mut self, _hspace: u32, _rv: &mut ErrorResult) {
    }

    pub fn Standby(&self) -> DOMString {
        None
    }

    pub fn SetStandby(&mut self, _standby: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Vspace(&self) -> u32 {
        0
    }

    pub fn SetVspace(&mut self, _vspace: u32, _rv: &mut ErrorResult) {
    }

    pub fn CodeBase(&self) -> DOMString {
        None
    }

    pub fn SetCodeBase(&mut self, _codebase: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn CodeType(&self) -> DOMString {
        None
    }

    pub fn SetCodeType(&mut self, _codetype: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Border(&self) -> DOMString {
        None
    }

    pub fn SetBorder(&mut self, _border: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn GetSVGDocument(&self) -> Option<AbstractDocument> {
        None
    }
}