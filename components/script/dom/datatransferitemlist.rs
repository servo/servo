/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::MutableHandleValue;

use crate::dom::bindings::codegen::Bindings::DataTransferItemListBinding::DataTransferItemListMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransferitem::DataTransferItem;
use crate::dom::file::File;
use crate::dom::window::Window;
use crate::drag_data_store::{DragDataStore, Kind, Mode};
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct DataTransferItemList {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "Rc"]
    #[no_trace]
    data_store: Rc<RefCell<Option<DragDataStore>>>,
    #[ignore_malloc_size_of = "mozjs"]
    frozen_types: CachedFrozenArray,
}

impl DataTransferItemList {
    fn new_inherited(data_store: Rc<RefCell<Option<DragDataStore>>>) -> DataTransferItemList {
        DataTransferItemList {
            reflector_: Reflector::new(),
            frozen_types: CachedFrozenArray::new(),
            data_store,
        }
    }

    pub(crate) fn new(
        window: &Window,
        data_store: Rc<RefCell<Option<DragDataStore>>>,
        can_gc: CanGc,
    ) -> DomRoot<DataTransferItemList> {
        reflect_dom_object(
            Box::new(DataTransferItemList::new_inherited(data_store)),
            window,
            can_gc,
        )
    }

    pub(crate) fn frozen_types(&self, cx: JSContext, retval: MutableHandleValue) {
        self.frozen_types.get_or_init(
            || {
                self.data_store
                    .borrow()
                    .as_ref()
                    .map_or(Vec::new(), |data_store| data_store.types())
            },
            cx,
            retval,
        );
    }

    pub(crate) fn invalidate_frozen_types(&self) {
        self.frozen_types.clear();
    }
}

impl DataTransferItemListMethods<crate::DomTypeHolder> for DataTransferItemList {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-length>
    fn Length(&self) -> u32 {
        // Return zero if the object is in the disabled mode;
        // otherwise it must return the number of items in the drag data store item list.
        self.data_store
            .borrow()
            .as_ref()
            .map_or(0, |data_store| data_store.list_len() as u32)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-item>
    fn IndexedGetter(&self, index: u32, can_gc: CanGc) -> Option<DomRoot<DataTransferItem>> {
        // Step 1 Return null if it isn't associated with a data store
        let option = self.data_store.borrow();
        let data_store = match option.as_ref() {
            Some(value) => value,
            _ => return None,
        };

        // Step 2
        data_store.get_by_index(index as usize).map(|(id, _)| {
            DataTransferItem::new(&self.global(), Rc::clone(&self.data_store), *id, can_gc)
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add>
    fn Add(
        &self,
        data: DOMString,
        mut type_: DOMString,
        can_gc: CanGc,
    ) -> Fallible<Option<DomRoot<DataTransferItem>>> {
        // Step 1 If the DataTransferItemList object is not in the read/write mode, return null.
        let mut option = self.data_store.borrow_mut();
        let data_store = match option.as_mut() {
            Some(value) if value.mode() == Mode::ReadWrite => value,
            _ => return Ok(None),
        };

        // Add an item to the drag data store item list whose kind is text,
        // whose type string is equal to the value of the method's second argument, converted to ASCII lowercase,
        // and whose data is the string given by the method's first argument.
        type_.make_ascii_lowercase();
        data_store.add(Kind::Text { data, type_ }).map(|id| {
            self.frozen_types.clear();

            // Step 3 Determine the value of the indexed property corresponding to the newly added item,
            // and return that value (a newly created DataTransferItem object).
            Some(DataTransferItem::new(
                &self.global(),
                Rc::clone(&self.data_store),
                id,
                can_gc,
            ))
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add>
    fn Add_(&self, data: &File, can_gc: CanGc) -> Fallible<Option<DomRoot<DataTransferItem>>> {
        // Step 1 If the DataTransferItemList object is not in the read/write mode, return null.
        let mut option = self.data_store.borrow_mut();
        let data_store = match option.as_mut() {
            Some(value) if value.mode() == Mode::ReadWrite => value,
            _ => return Ok(None),
        };

        // Add an item to the drag data store item list whose kind is File,
        // whose type string is the type of the File, converted to ASCII lowercase,
        // and whose data is the same as the File's data.
        let mut type_ = data.file_type();
        type_.make_ascii_lowercase();
        let bytes = data.file_bytes().unwrap_or_default();
        let name = data.name().clone();

        data_store.add(Kind::File { bytes, name, type_ }).map(|id| {
            self.frozen_types.clear();

            // Step 3 Determine the value of the indexed property corresponding to the newly added item,
            // and return that value (a newly created DataTransferItem object).
            Some(DataTransferItem::new(
                &self.global(),
                Rc::clone(&self.data_store),
                id,
                can_gc,
            ))
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-remove>
    fn Remove(&self, index: u32) -> Fallible<()> {
        // Step 1 If the DataTransferItemList object is not in the read/write mode,
        // throw an "InvalidStateError" DOMException.
        let mut option = self.data_store.borrow_mut();
        let data_store = match option.as_mut() {
            Some(value) if value.mode() == Mode::ReadWrite => value,
            _ => return Err(Error::InvalidState),
        };

        let index = index as usize;

        // Step 2 If the drag data store does not contain an indexth item, then return.
        if index < data_store.list_len() {
            // Step 3 Remove the indexth item from the drag data store.
            data_store.remove(index);
            self.frozen_types.clear();
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-clear>
    fn Clear(&self) {
        // If the DataTransferItemList object is in the read/write mode, remove all the items from the drag data store.
        // Otherwise, it must do nothing.
        let mut option = self.data_store.borrow_mut();
        let data_store = match option.as_mut() {
            Some(value) if value.mode() == Mode::ReadWrite => value,
            _ => return,
        };

        // If the item list is empty we don't clear it.
        if data_store.list_len() > 0 {
            data_store.clear_list();
            self.frozen_types.clear();
        }
    }
}
