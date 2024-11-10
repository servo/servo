/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::rust::{HandleObject, MutableHandleValue};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DataTransferBinding::DataTransferMethods;
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransferitemlist::DataTransferItemList;
use crate::dom::element::Element;
use crate::dom::filelist::FileList;
use crate::dom::htmlimageelement::HTMLImageElement;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext};

const VALID_DROP_EFFECTS: [&str; 4] = ["none", "copy", "link", "move"];
const VALID_EFFECTS_ALLOWED: [&str; 9] = [
    "none",
    "copy",
    "copyLink",
    "copyMove",
    "link",
    "linkMove",
    "move",
    "all",
    "uninitialized",
];

// An image that can be shown when a drag event occurs
#[derive(JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
struct DragBitmap {
    image: Dom<HTMLImageElement>,
    hot_spot_x: i32,
    hot_spot_y: i32,
}

#[derive(Clone, Copy, Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub enum Mode {
    ReadWrite,
    #[allow(dead_code)]
    ReadOnly,
    Protected,
}

#[dom_struct]
pub struct DataTransfer {
    reflector_: Reflector,
    drop_effect: DomRefCell<DOMString>,
    effect_allowed: DomRefCell<DOMString>,
    items: Dom<DataTransferItemList>,
    drag_image: DomRefCell<Option<DragBitmap>>,
    #[ignore_malloc_size_of = "mozjs"]
    frozen_types: CachedFrozenArray,
    mode: Cell<Mode>,
}

impl DataTransfer {
    pub fn new_inherited(item_list: &DataTransferItemList) -> DataTransfer {
        DataTransfer {
            reflector_: Reflector::new(),
            drop_effect: DomRefCell::new(DOMString::from("none")),
            effect_allowed: DomRefCell::new(DOMString::from("none")),
            items: Dom::from_ref(item_list),
            drag_image: DomRefCell::new(None),
            frozen_types: CachedFrozenArray::new(),
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
        data_transfer.items.set_data_store(Some(&data_transfer));
        data_transfer
    }

    pub fn invalidate_frozen_types(&self) {
        self.frozen_types.clear();
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
        if VALID_DROP_EFFECTS.contains(&value.as_ref()) {
            *self.drop_effect.borrow_mut() = value;
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-effectallowed>
    fn EffectAllowed(&self) -> DOMString {
        self.effect_allowed.borrow().clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-effectallowed>
    fn SetEffectAllowed(&self, value: DOMString) {
        if self.can_write() && VALID_EFFECTS_ALLOWED.contains(&value.as_ref()) {
            *self.drop_effect.borrow_mut() = value;
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-items>
    fn Items(&self) -> DomRoot<DataTransferItemList> {
        DomRoot::from_ref(&self.items)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-setdragimage>
    fn SetDragImage(&self, image: &Element, x: i32, y: i32) {
        // Step 1 If the DataTransfer is no longer associated with a data store, return. TODO

        // Step 2 If the data store's mode is not the read/write mode, return.
        if !self.can_write() {
            return;
        }

        // Step 3
        if let Some(image) = image.downcast::<HTMLImageElement>() {
            *self.drag_image.borrow_mut() = Some(DragBitmap {
                image: Dom::from_ref(image),
                hot_spot_x: x,
                hot_spot_y: y,
            });
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-types>
    fn Types(&self, cx: JSContext, retval: MutableHandleValue) {
        self.frozen_types
            .get_or_init(|| self.items.types(), cx, retval);
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
