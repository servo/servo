/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DataTransferBinding::DataTransferMethods;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransferitemlist::DataTransferItemList;
use crate::dom::element::Element;
use crate::dom::window::Window;

#[dom_struct]
pub struct DataTransfer {
    reflector_: Reflector,
    dropEffect: DomRefCell<DOMString>,
    effectAllowed: DomRefCell<DOMString>,
    items: Dom<DataTransferItemList>,
}

impl DataTransfer {
    pub fn new_inherited() -> DataTransfer {
        DataTransfer {
            reflector_: Reflector::new(),
            dropEffect: DomRefCell::new(DOMString::from("none")),
            effectAllowed: DomRefCell::new(DOMString::from("none")),
            items: Dom::from_ref(&DataTransferItemList::new_inherited()),
        }
    }

    pub fn new(window: &Window, proto: Option<HandleObject>) -> DomRoot<DataTransfer> {
        reflect_dom_object_with_proto(Box::new(DataTransfer::new_inherited()), window, proto)
    }

    pub fn Constructor(window: &Window, proto: Option<HandleObject>) -> DomRoot<DataTransfer> {
        DataTransfer::new(window, proto)
    }
}

impl DataTransferMethods for DataTransfer {
    // https://html.spec.whatwg.org/multipage/#dom-datatransfer-dropeffect
    fn DropEffect(&self) -> DOMString {
        self.dropEffect.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransfer-dropeffect
    fn SetDropEffect(&self, value: DOMString) {
        *self.dropEffect.borrow_mut() = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransfer-effectallowed
    fn EffectAllowed(&self) -> DOMString {
        self.effectAllowed.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransfer-effectallowed
    fn SetEffectAllowed(&self, value: DOMString) {
        *self.effectAllowed.borrow_mut() = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransfer-items
    fn Items(&self) -> DomRoot<DataTransferItemList> {
        DomRoot::from_ref(&self.items)
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransfer-setdragimage
    fn SetDragImage(&self, image: &Element, x: i32, y: i32) {
        todo!()
    }
}
