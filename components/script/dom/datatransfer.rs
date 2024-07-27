/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DataTransferBinding::DataTransferMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransferitemlist::DataTransferItemList;
use crate::dom::element::Element;
use crate::dom::window::Window;

#[dom_struct]
pub struct DataTransfer {
    reflector_: Reflector,
    dropEffect: DomRefCell<DOMString>,
    effectAllowed: DomRefCell<DOMString>,
}

impl DataTransfer {
    pub fn Constructor(window: &Window, proto: Option<HandleObject>) -> DomRoot<DataTransfer> {
        todo!()
    }
}

impl DataTransferMethods for DataTransfer {
    fn DropEffect(&self) -> DOMString {
        todo!()
    }
    fn SetDropEffect(&self, value: DOMString) {
        todo!()
    }
    fn EffectAllowed(&self) -> DOMString {
        todo!()
    }
    fn SetEffectAllowed(&self, value: DOMString) {
        todo!()
    }
    fn Items(&self) -> DomRoot<DataTransferItemList> {
        todo!()
    }
    fn SetDragImage(&self, image: &Element, x: i32, y: i32) {
        todo!()
    }
}
