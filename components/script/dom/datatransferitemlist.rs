/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DataTransferItemListBinding::DataTransferItemListMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::weakref::MutableWeakRef;
use crate::dom::datatransfer::DataTransfer;
use crate::dom::datatransferitem::{DataTransferItem, Kind};
use crate::dom::file::File;
use crate::dom::window::Window;

#[dom_struct]
pub struct DataTransferItemList {
    reflector_: Reflector,
    items: DomRefCell<Vec<Dom<DataTransferItem>>>,
    data_store: MutableWeakRef<DataTransfer>,
}

impl DataTransferItemList {
    pub fn new_inherited() -> DataTransferItemList {
        DataTransferItemList {
            reflector_: Reflector::new(),
            items: DomRefCell::new(Vec::new()),
            data_store: MutableWeakRef::new(None),
        }
    }

    pub fn new(window: &Window) -> DomRoot<DataTransferItemList> {
        reflect_dom_object(Box::new(DataTransferItemList::new_inherited()), window)
    }

    fn add_item(&self, kind: Kind, type_: DOMString) -> DomRoot<DataTransferItem> {
        let item = DataTransferItem::new(
            &self.global(),
            type_,
            kind,
            self.data_store.root().as_deref(),
        );
        self.items.borrow_mut().push(Dom::from_ref(&item));

        item
    }

    pub fn get_data(&self, mut format: DOMString) -> DOMString {
        if self
            .data_store
            .root()
            .is_some_and(|data_store| data_store.can_read())
        {
            // Step 3 Convert format to ASCII lowercase.
            format.make_ascii_lowercase();
            // Step 4 Let convert-to-URL be false.
            let mut convert_to_url = false;

            // Step 5 & 6
            let type_ = match format.as_ref() {
                "text" => DOMString::from("text/plain"),
                "url" => {
                    convert_to_url = true;
                    DOMString::from("text/uri-list")
                },
                _ => format,
            };

            // Step 8
            let data = self
                .items
                .borrow()
                .iter()
                .find(|item| item.text_type_matches(&type_))
                .map(|item| item.as_string())
                .flatten();

            if let Some(result) = data {
                // Step 9
                if convert_to_url {
                    //TODO parse uri-list as [RFC2483]
                }
                // Step 10
                result
            } else {
                // Step 7 If there is no item whose kind is text and whose type is equal to format, return the empty string.
                DOMString::new()
            }
        } else {
            // Step 1 & 2
            DOMString::new()
        }
    }

    pub fn set_data(&self, mut format: DOMString, data: DOMString) {
        //Step 1 If the DataTransfer is no longer associated with a data store, return.
        if let Some(data_store) = self.data_store.root() {
            //Step 2 If the data store is not int the read/write mode, return.
            if !data_store.can_write() {
                return;
            }

            // Step 3 Convert format to ASCII lowercase.
            format.make_ascii_lowercase();
            // Step 4
            let type_ = match format.as_ref() {
                "text" => DOMString::from("text/plain"),
                "url" => DOMString::from("text/uri-list"),
                _ => format,
            };

            // Step 5 Remove the item in the item list whose kind is text and whose type is equal to format.
            self.items
                .borrow_mut()
                .retain(|item| !item.text_type_matches(&type_));

            // Step 6 Add an item to the item list whose kind is text,
            // whose type is equal to format, and whose data is the method's second argument.
            self.add_item(Kind::Text(data), type_);
        }
    }

    pub fn clear_data(&self, format: Option<DOMString>) {
        // Step 1 If the DataTransfer is no longer associated with a data store, return.
        if let Some(data_store) = self.data_store.root() {
            // Step 2 If the data store is not int the read/write mode, return.
            if !data_store.can_write() {
                return;
            }

            if let Some(mut format) = format {
                // Step 4 Convert format to ASCII lowercase.
                format.make_ascii_lowercase();
                // Step 5
                let type_ = match format.as_ref() {
                    "text" => DOMString::from("text/plain"),
                    "url" => DOMString::from("text/uri-list"),
                    _ => format,
                };

                // Step 6 Remove the item in the item list whose kind is text and whose type is equal to format.
                self.items
                    .borrow_mut()
                    .retain(|item| !item.text_type_matches(&type_));
            } else {
                // Step 3 If format is None, remove each item in the item list whose kind is text.
                self.items.borrow_mut().retain(|item| item.is_file());
            }
        }
    }
}

impl DataTransferItemListMethods for DataTransferItemList {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-length>
    fn Length(&self) -> u32 {
        // Step 1 Return 0 if it isn't associated with a data store
        if self.data_store.root().is_none() {
            return 0;
        }

        // Step 2
        self.items.borrow().len() as u32
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-item>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<DataTransferItem>> {
        // Step 1 Return null if it isn't associated with a data store
        self.data_store.root()?;

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
        if let Some(data_store) = self.data_store.root() {
            // Step 1 If the data store is not in the read/write mode, return null.
            if !data_store.can_write() {
                return Ok(None);
            }

            // Step 2.1 If there is already an item in the item list whose kind is text
            // and whose type string is equal to the method's second argument, throw "NotSupportedError".
            type_.make_ascii_lowercase();
            if self
                .items
                .borrow()
                .iter()
                .any(|item| item.text_type_matches(&type_))
            {
                return Err(Error::NotSupported);
            }

            // Step 2.2
            Ok(Some(self.add_item(Kind::Text(data), type_)))
        } else {
            Ok(None)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add>
    fn Add_(&self, data: &File) -> Fallible<Option<DomRoot<DataTransferItem>>> {
        if let Some(data_store) = self.data_store.root() {
            // Step 1 If the data store is not in the read/write mode, return null.
            if !data_store.can_write() {
                return Ok(None);
            }

            // Step 2
            let mut type_ = DOMString::from(data.file_type());
            type_.make_ascii_lowercase();
            Ok(Some(self.add_item(Kind::File(data), type_)))
        } else {
            Ok(None)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-remove>
    fn Remove(&self, index: u32) -> Fallible<()> {
        self.data_store
            .root()
            .and_then(|data_store| {
                if !data_store.can_write() {
                    return None;
                }

                if (index as usize) < self.items.borrow().len() {
                    self.items.borrow_mut().remove(index as usize);
                }
                Some(())
            })
            .ok_or(Error::InvalidState)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-clear>
    fn Clear(&self) {
        if let Some(data_store) = self.data_store.root() {
            // Step 1 If the data store is not in the read/write mode, return.
            if !data_store.can_write() {
                return;
            }

            // Step 2 remove all the items
            if !self.items.borrow().is_empty() {
                self.items.borrow_mut().clear();
            }
        }
    }
}
