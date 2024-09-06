/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

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
use crate::script_runtime::CanGc;

#[derive(Clone, Copy, Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub enum Mode {
    ReadWrite,
    ReadOnly,
    Protected,
}

#[dom_struct]
pub struct DataTransfer {
    reflector_: Reflector,
    drop_effect: DomRefCell<DOMString>,
    effect_allowed: DomRefCell<DOMString>,
    mode: Cell<Mode>,
    items: Dom<DataTransferItemList>,
}

impl DataTransfer {
    pub fn new_inherited(item_list: &DataTransferItemList) -> DataTransfer {
        DataTransfer {
            reflector_: Reflector::new(),
            drop_effect: DomRefCell::new(DOMString::from("none")),
            effect_allowed: DomRefCell::new(DOMString::from("none")),
            items: Dom::from_ref(item_list),
            mode: Cell::new(Mode::ReadWrite),
        }
    }

    pub fn new(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<DataTransfer> {
        let item_list = DataTransferItemList::new(window, proto);
        let data_transfer = reflect_dom_object_with_proto(
            Box::new(DataTransfer::new_inherited(&item_list)),
            window,
            proto,
            can_gc,
        );
        data_transfer.items.set_data_transfer(Some(&data_transfer));
        data_transfer
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<DataTransfer> {
        DataTransfer::new(window, proto, can_gc)
    }

    pub fn can_write(&self) -> bool {
        self.mode.get() == Mode::ReadWrite
    }

    pub fn can_read(&self) -> bool {
        self.mode.get() != Mode::Protected
    }
}

impl DataTransferMethods for DataTransfer {
    // https://html.spec.whatwg.org/multipage/#dom-datatransfer-dropeffect
    fn DropEffect(&self) -> DOMString {
        self.drop_effect.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransfer-dropeffect
    fn SetDropEffect(&self, value: DOMString) {
        match value.as_ref() {
            "none" | "copy" | "link" | "move" => *self.drop_effect.borrow_mut() = value,
            _ => {},
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransfer-effectallowed
    fn EffectAllowed(&self) -> DOMString {
        self.effect_allowed.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransfer-effectallowed
    fn SetEffectAllowed(&self, value: DOMString) {
        if self.can_write() {
            match value.as_ref() {
                "none" | "copy" | "copyLink" | "copyMove" | "link" | "linkMove" | "move" |
                "all" | "uninitialized" => *self.drop_effect.borrow_mut() = value,
                _ => {},
            }
        }
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
