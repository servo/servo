/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::HTMLTableCellElementDerived;
use dom::bindings::js::JSRef;
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
    pub fn new_inherited(type_id: ElementTypeId, tag_name: DOMString, document: &JSRef<Document>) -> HTMLTableCellElement {
        HTMLTableCellElement {
            htmlelement: HTMLElement::new_inherited(type_id, tag_name, document)
        }
    }
}

pub trait HTMLTableCellElementMethods {
    fn ColSpan(&self) -> u32;
    fn SetColSpan(&self, _col_span: u32) -> ErrorResult;
    fn RowSpan(&self) -> u32;
    fn SetRowSpan(&self, _col_span: u32) -> ErrorResult;
    fn Headers(&self) -> DOMString;
    fn SetHeaders(&self, _headers: DOMString) -> ErrorResult;
    fn CellIndex(&self) -> i32;
    fn GetCellIndex(&self, _cell_index: i32) -> ErrorResult;
    fn Abbr(&self) -> DOMString;
    fn SetAbbr(&self, _abbr: DOMString) -> ErrorResult;
    fn Scope(&self) -> DOMString;
    fn SetScope(&self, _abbr: DOMString) -> ErrorResult;
    fn Align(&self) -> DOMString;
    fn SetAlign(&self, _align: DOMString) -> ErrorResult;
    fn Axis(&self) -> DOMString;
    fn SetAxis(&self, _axis: DOMString) -> ErrorResult;
    fn Height(&self) -> DOMString;
    fn SetHeight(&self, _height: DOMString) -> ErrorResult;
    fn Width(&self) -> DOMString;
    fn SetWidth(&self, _width: DOMString) -> ErrorResult;
    fn Ch(&self) -> DOMString;
    fn SetCh(&self, _ch: DOMString) -> ErrorResult;
    fn ChOff(&self) -> DOMString;
    fn SetChOff(&self, _ch_off: DOMString) -> ErrorResult;
    fn NoWrap(&self) -> bool;
    fn SetNoWrap(&self, _no_wrap: bool) -> ErrorResult;
    fn VAlign(&self) -> DOMString;
    fn SetVAlign(&self, _valign: DOMString) -> ErrorResult;
    fn BgColor(&self) -> DOMString;
    fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult;
}

impl<'a> HTMLTableCellElementMethods for JSRef<'a, HTMLTableCellElement> {
    fn ColSpan(&self) -> u32 {
        0
    }

    fn SetColSpan(&self, _col_span: u32) -> ErrorResult {
        Ok(())
    }

    fn RowSpan(&self) -> u32 {
        0
    }

    fn SetRowSpan(&self, _col_span: u32) -> ErrorResult {
        Ok(())
    }

    fn Headers(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHeaders(&self, _headers: DOMString) -> ErrorResult {
        Ok(())
    }

    fn CellIndex(&self) -> i32 {
        0
    }

    fn GetCellIndex(&self, _cell_index: i32) -> ErrorResult {
        Ok(())
    }

    fn Abbr(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAbbr(&self, _abbr: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Scope(&self) -> DOMString {
        "".to_owned()
    }

    fn SetScope(&self, _abbr: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Align(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAlign(&self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Axis(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAxis(&self, _axis: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Height(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHeight(&self, _height: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Width(&self) -> DOMString {
        "".to_owned()
    }

    fn SetWidth(&self, _width: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Ch(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCh(&self, _ch: DOMString) -> ErrorResult {
        Ok(())
    }

    fn ChOff(&self) -> DOMString {
        "".to_owned()
    }

    fn SetChOff(&self, _ch_off: DOMString) -> ErrorResult {
        Ok(())
    }

    fn NoWrap(&self) -> bool {
        false
    }

    fn SetNoWrap(&self, _no_wrap: bool) -> ErrorResult {
        Ok(())
    }

    fn VAlign(&self) -> DOMString {
        "".to_owned()
    }

    fn SetVAlign(&self, _valign: DOMString) -> ErrorResult {
        Ok(())
    }

    fn BgColor(&self) -> DOMString {
        "".to_owned()
    }

    fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult {
        Ok(())
    }
}
