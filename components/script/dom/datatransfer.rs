/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DataTransferBinding::DataTransferMethods;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransferitemlist::DataTransferItemList;
use crate::dom::element::Element;
use crate::dom::filelist::FileList;
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

    pub fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<DataTransfer> {
        let item_list = DataTransferItemList::new(window);
        let data_transfer = reflect_dom_object_with_proto(
            Box::new(DataTransfer::new_inherited(&item_list)),
            window,
            proto,
            can_gc,
        );
        data_transfer.items.set_data_transfer(Some(&data_transfer));
        data_transfer
    }

    pub fn new(window: &Window) -> DomRoot<DataTransfer> {
        Self::new_with_proto(window, None, CanGc::note())
    }

    pub fn set_mode(&self, mode: Mode) {
        self.mode.set(mode);
    }

    pub fn can_write(&self) -> bool {
        self.mode.get() == Mode::ReadWrite
    }

    pub fn can_read(&self) -> bool {
        self.mode.get() != Mode::Protected
    }
}

impl DataTransferMethods for DataTransfer {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<DataTransfer> {
        DataTransfer::new_with_proto(window, proto, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-dropeffect>
    fn DropEffect(&self) -> DOMString {
        self.drop_effect.borrow().clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-dropeffect>
    fn SetDropEffect(&self, value: DOMString) {
        match value.as_ref() {
            "none" | "copy" | "link" | "move" => *self.drop_effect.borrow_mut() = value,
            _ => {},
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-effectallowed>
    fn EffectAllowed(&self) -> DOMString {
        self.effect_allowed.borrow().clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-effectallowed>
    fn SetEffectAllowed(&self, value: DOMString) {
        if self.can_write() {
            match value.as_ref() {
                "none" | "copy" | "copyLink" | "copyMove" | "link" | "linkMove" | "move" |
                "all" | "uninitialized" => *self.drop_effect.borrow_mut() = value,
                _ => {},
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-items>
    fn Items(&self) -> DomRoot<DataTransferItemList> {
        DomRoot::from_ref(&self.items)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-setdragimage>
    fn SetDragImage(&self, _image: &Element, _x: i32, _y: i32) {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-getdata>
    fn GetData(&self, format: DOMString) -> DOMString {
        self.items.get_data(format)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-setdata>
    fn SetData(&self, format: DOMString, data: DOMString) {
        self.items.set_data(format, data);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-cleardata>
    fn ClearData(&self, format: Option<DOMString>) {
        self.items.clear_data(format);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-files>
    fn Files(&self) -> DomRoot<FileList> {
        FileList::new(self.global().as_window(), self.items.files())
    }
}
