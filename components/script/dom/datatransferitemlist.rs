/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DataTransferItemListBinding::DataTransferItemListMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransferitem::DataTransferItem;
use crate::dom::file::File;
use crate::dom::window::Window;
use crate::drag_data_store::{Binary, DragDataStore, Kind, Mode, PlainString};

#[dom_struct]
pub struct DataTransferItemList {
    reflector_: Reflector,
    items: DomRefCell<Vec<Dom<DataTransferItem>>>,
    #[ignore_malloc_size_of = "Rc"]
    #[no_trace]
    data_store: Rc<RefCell<Option<DragDataStore>>>,
}

impl DataTransferItemList {
    fn new_inherited(data_store: Rc<RefCell<Option<DragDataStore>>>) -> DataTransferItemList {
        DataTransferItemList {
            reflector_: Reflector::new(),
            items: DomRefCell::new(Vec::new()),
            data_store,
        }
    }

    pub fn new(
        window: &Window,
        data_store: Rc<RefCell<Option<DragDataStore>>>,
    ) -> DomRoot<DataTransferItemList> {
        reflect_dom_object(
            Box::new(DataTransferItemList::new_inherited(data_store)),
            window,
        )
    }
}

impl DataTransferItemListMethods for DataTransferItemList {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-length>
    fn Length(&self) -> u32 {
        // Return zero if the object is in the disabled mode;
        // otherwise it must return the number of items in the drag data store item list.
        self.data_store
            .borrow()
            .as_ref()
            .map_or(0, |data_store| data_store.list_len())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-item>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<DataTransferItem>> {
        // Step 1 Return null if it isn't associated with a data store

        // Step 2
        self.items
            .borrow()
            .get(index as usize)
            .map(|item| DomRoot::from_ref(&**item))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add>
    fn Add(
        &self,
        data: DOMString,
        mut type_: DOMString,
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
        let string = PlainString { data, type_ };

        data_store.add(Kind::Text(string))?;

        Ok(Some(DataTransferItem::new(&self.global())))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add>
    fn Add_(&self, data: &File) -> Fallible<Option<DomRoot<DataTransferItem>>> {
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
        let binary = Binary {
            bytes: data.file_bytes().unwrap_or_default(),
            name: data.name().clone(),
            type_,
        };

        data_store.add(Kind::File(binary))?;

        Ok(Some(DataTransferItem::new(&self.global())))
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

        // Step 2 If the drag data store does not contain an indexth item, then return.
        if index < data_store.list_len() {
            // Step 3 Remove the indexth item from the drag data store.
            data_store.remove(index);
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

        data_store.clear_list();
    }
}
