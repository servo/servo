/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::HTMLTableCellElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::{ElementTypeId, HTMLTableDataCellElementTypeId, HTMLTableHeaderCellElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::ElementNodeTypeId;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTableCellElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTableCellElementDerived for EventTarget {
    fn is_htmltablecellelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTableDataCellElementTypeId)) |
            NodeTargetTypeId(ElementNodeTypeId(HTMLTableHeaderCellElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTableCellElement {
    pub fn new_inherited(type_id: ElementTypeId, tag_name: DOMString, document: JS<Document>) -> HTMLTableCellElement {
        HTMLTableCellElement {
            htmlelement: HTMLElement::new_inherited(type_id, tag_name, document)
        }
    }
}

impl HTMLTableCellElement {
    pub fn ColSpan(&self) -> u32 {
        0
    }

    pub fn SetColSpan(&self, _col_span: u32) -> ErrorResult {
        Ok(())
    }

    pub fn RowSpan(&self) -> u32 {
        0
    }

    pub fn SetRowSpan(&self, _col_span: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Headers(&self) -> DOMString {
        ~""
    }

    pub fn SetHeaders(&self, _headers: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CellIndex(&self) -> i32 {
        0
    }

    pub fn GetCellIndex(&self, _cell_index: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Abbr(&self) -> DOMString {
        ~""
    }

    pub fn SetAbbr(&self, _abbr: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Scope(&self) -> DOMString {
        ~""
    }

    pub fn SetScope(&self, _abbr: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Axis(&self) -> DOMString {
        ~""
    }

    pub fn SetAxis(&self, _axis: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Height(&self) -> DOMString {
        ~""
    }

    pub fn SetHeight(&self, _height: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> DOMString {
        ~""
    }

    pub fn SetWidth(&self, _width: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Ch(&self) -> DOMString {
        ~""
    }

    pub fn SetCh(&self, _ch: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ChOff(&self) -> DOMString {
        ~""
    }

    pub fn SetChOff(&self, _ch_off: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn NoWrap(&self) -> bool {
        false
    }

    pub fn SetNoWrap(&self, _no_wrap: bool) -> ErrorResult {
        Ok(())
    }

    pub fn VAlign(&self) -> DOMString {
        ~""
    }

    pub fn SetVAlign(&self, _valign: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn BgColor(&self) -> DOMString {
        ~""
    }

    pub fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult {
        Ok(())
    }
}

