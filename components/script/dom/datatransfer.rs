/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::{HandleObject, MutableHandleValue};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DataTransferBinding::DataTransferMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransferitemlist::DataTransferItemList;
use crate::dom::element::Element;
use crate::dom::filelist::FileList;
use crate::dom::htmlimageelement::HTMLImageElement;
use crate::dom::window::Window;
use crate::drag_data_store::{DragDataStore, Mode};
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

#[dom_struct]
pub(crate) struct DataTransfer {
    reflector_: Reflector,
    drop_effect: DomRefCell<DOMString>,
    effect_allowed: DomRefCell<DOMString>,
    items: Dom<DataTransferItemList>,
    #[ignore_malloc_size_of = "Rc"]
    #[no_trace]
    data_store: Rc<RefCell<Option<DragDataStore>>>,
}

impl DataTransfer {
    fn new_inherited(
        data_store: Rc<RefCell<Option<DragDataStore>>>,
        item_list: &DataTransferItemList,
    ) -> DataTransfer {
        DataTransfer {
            reflector_: Reflector::new(),
            drop_effect: DomRefCell::new(DOMString::from("none")),
            effect_allowed: DomRefCell::new(DOMString::from("none")),
            items: Dom::from_ref(item_list),
            data_store,
        }
    }

    pub(crate) fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        data_store: Rc<RefCell<Option<DragDataStore>>>,
    ) -> DomRoot<DataTransfer> {
        let item_list = DataTransferItemList::new(window, Rc::clone(&data_store), can_gc);

        reflect_dom_object_with_proto(
            Box::new(DataTransfer::new_inherited(data_store, &item_list)),
            window,
            proto,
            can_gc,
        )
    }

    pub(crate) fn new(
        window: &Window,
        data_store: Rc<RefCell<Option<DragDataStore>>>,
        can_gc: CanGc,
    ) -> DomRoot<DataTransfer> {
        Self::new_with_proto(window, None, can_gc, data_store)
    }

    pub(crate) fn data_store(&self) -> Option<Ref<DragDataStore>> {
        Ref::filter_map(self.data_store.borrow(), |data_store| data_store.as_ref()).ok()
    }
}

impl DataTransferMethods<crate::DomTypeHolder> for DataTransfer {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<DataTransfer> {
        let mut drag_data_store = DragDataStore::new();
        drag_data_store.set_mode(Mode::ReadWrite);

        let data_store = Rc::new(RefCell::new(Some(drag_data_store)));

        DataTransfer::new_with_proto(window, proto, can_gc, data_store)
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
        if self
            .data_store
            .borrow()
            .as_ref()
            .is_some_and(|data_store| data_store.mode() == Mode::ReadWrite) &&
            VALID_EFFECTS_ALLOWED.contains(&value.as_ref())
        {
            *self.drop_effect.borrow_mut() = value;
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-items>
    fn Items(&self) -> DomRoot<DataTransferItemList> {
        DomRoot::from_ref(&self.items)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-setdragimage>
    fn SetDragImage(&self, image: &Element, x: i32, y: i32) {
        // Step 1 If the DataTransfer is no longer associated with a data store, return.
        let mut option = self.data_store.borrow_mut();
        let data_store = match option.as_mut() {
            Some(value) => value,
            None => return,
        };

        // Step 2 If the data store's mode is not the read/write mode, return.
        if data_store.mode() != Mode::ReadWrite {
            return;
        }

        // Step 3
        if let Some(image) = image.downcast::<HTMLImageElement>() {
            data_store.set_bitmap(image.image_data(), x, y);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-types>
    fn Types(&self, cx: JSContext, retval: MutableHandleValue) {
        self.items.frozen_types(cx, retval);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-getdata>
    fn GetData(&self, mut format: DOMString) -> DOMString {
        // Step 1 If the DataTransfer object is not associated with a drag data store, then return the empty string.
        let option = self.data_store.borrow();
        let data_store = match option.as_ref() {
            Some(value) => value,
            None => return DOMString::new(),
        };

        // Step 2 If the drag data store's mode is the protected mode, then return the empty string.
        if data_store.mode() == Mode::Protected {
            return DOMString::new();
        }

        // Step 3 Let format be the first argument, converted to ASCII lowercase.
        format.make_ascii_lowercase();
        // Step 4 Let convert-to-URL be false.
        let mut convert_to_url = false;

        let type_ = match format.as_ref() {
            // Step 5 If format equals "text", change it to "text/plain".
            "text" => DOMString::from("text/plain"),
            // Step 6 If format equals "url", change it to "text/uri-list" and set convert-to-URL to true.
            "url" => {
                convert_to_url = true;
                DOMString::from("text/uri-list")
            },
            _ => format,
        };

        let data = data_store.find_matching_text(&type_);

        // Step 8
        if let Some(result) = data {
            // Step 9 If convert-to-URL is true, then parse result as appropriate for text/uri-list data,
            // and then set result to the first URL from the list, if any, or the empty string otherwise.
            if convert_to_url {
                //TODO parse uri-list as [RFC2483]
            }

            // Step 10 Return result.
            result
        } else {
            // Step 7 If there is no item in the drag data store item list
            // whose kind is text and whose type string is equal to format, return the empty string.
            DOMString::new()
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-setdata>
    fn SetData(&self, format: DOMString, data: DOMString) {
        // Step 1 If the DataTransfer object is no longer associated with a drag data store, return. Nothing happens.
        let mut option = self.data_store.borrow_mut();
        let data_store = match option.as_mut() {
            Some(value) => value,
            None => return,
        };

        // Step 2 If the drag data store's mode is not the read/write mode, return. Nothing happens.
        if data_store.mode() != Mode::ReadWrite {
            return;
        }

        data_store.set_data(format, data);
        self.items.invalidate_frozen_types();
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-cleardata>
    fn ClearData(&self, format: Option<DOMString>) {
        // Step 1 If the DataTransfer is not associated with a data store, return.
        let mut option = self.data_store.borrow_mut();
        let data_store = match option.as_mut() {
            Some(value) => value,
            None => return,
        };

        // Step 2 If the data store is not in the read/write mode, return.
        if data_store.mode() != Mode::ReadWrite {
            return;
        }

        if data_store.clear_data(format) {
            self.items.invalidate_frozen_types();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-files>
    fn Files(&self, can_gc: CanGc) -> DomRoot<FileList> {
        // Step 1 Start with an empty list.
        let mut files = Vec::new();

        // Step 2 If the DataTransfer is not associated with a data store return the empty list.
        if let Some(data_store) = self.data_store.borrow().as_ref() {
            data_store.files(&self.global(), can_gc, &mut files);
        }

        // Step 5
        FileList::new(self.global().as_window(), files)
    }
}
