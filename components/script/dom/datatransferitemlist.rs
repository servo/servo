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
    data_transfer: MutableWeakRef<DataTransfer>,
    items: DomRefCell<Vec<Dom<DataTransferItem>>>,
}

impl DataTransferItemList {
    pub fn new_inherited() -> DataTransferItemList {
        DataTransferItemList {
            reflector_: Reflector::new(),
            data_transfer: MutableWeakRef::new(None),
            items: DomRefCell::new(Vec::new()),
        }
    }

    pub fn new(window: &Window) -> DomRoot<DataTransferItemList> {
        reflect_dom_object(Box::new(DataTransferItemList::new_inherited()), window)
    }

    pub fn set_data_transfer(&self, data_transfer: Option<&DataTransfer>) {
        self.data_transfer.set(data_transfer);
        for item in self.items.borrow().iter() {
            item.set_data_transfer(data_transfer);
        }
    }

    fn has_write_permission(&self) -> bool {
        self.data_transfer
            .root()
            .is_some_and(|data_transfer| data_transfer.can_write())
    }

    pub fn get_data(&self, mut format: DOMString) -> DOMString {
        if self
            .data_transfer
            .root()
            .is_some_and(|data_transfer| data_transfer.can_read())
        {
            let mut convert_to_url = false;
            format.make_ascii_lowercase();
            let type_ = match format.as_ref() {
                "text" => DOMString::from("text/plain"),
                "url" => {
                    convert_to_url = true;
                    DOMString::from("text/uri-list")
                },
                _ => return DOMString::from(""),
            };

            for item in self.items.borrow().iter() {
                if let Some(result) = item.get_string_of_type(&type_) {
                    if convert_to_url {
                        //TODO parse uri-list as [RFC2483]
                    }
                    return result;
                };
            }
        }
        DOMString::from("")
    }

    pub fn set_data(&self, mut format: DOMString, data: DOMString) {
        if self.has_write_permission() {
            format.make_ascii_lowercase();
            let type_ = match format.as_ref() {
                "text" => DOMString::from("text/plain"),
                "url" => DOMString::from("text/uri-list"),
                _ => return,
            };

            self.items
                .borrow_mut()
                .retain(|item| !item.type_matches(&type_));
            self.add_text_item(data, type_);
        }
    }

    pub fn clear_data(&self, format: Option<DOMString>) {
        if self.has_write_permission() {
            if let Some(mut format) = format {
                format.make_ascii_lowercase();
                let type_ = match format.as_ref() {
                    "text" => DOMString::from("text/plain"),
                    "url" => DOMString::from("text/uri-list"),
                    _ => return,
                };

                self.items
                    .borrow_mut()
                    .retain(|item| !item.type_matches(&type_));
            } else {
                self.items.borrow_mut().retain(|item| item.is_file_kind());
            }
        }
    }

    pub fn add_text_item(&self, data: DOMString, type_: DOMString) {
        let item = DataTransferItem::new(
            &self.global(),
            type_,
            Kind::Text(data),
            self.data_transfer.root().as_deref(),
        );
        self.items.borrow_mut().push(Dom::from_ref(&item));
    }

    pub fn is_empty(&self) -> bool {
        self.items.borrow().is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = DomRoot<DataTransferItem>> + '_ {
        let len = self.items.borrow().len() as u32;
        (0..len).flat_map(move |i| self.IndexedGetter(i))
    }

    pub fn files(&self) -> Vec<DomRoot<File>> {
        let mut files = Vec::new();

        if self
            .data_transfer
            .root()
            .is_some_and(|data_transfer| data_transfer.can_read())
        {
            for item in self.items.borrow().iter() {
                if let Some(file) = item.get_as_file() {
                    files.push(file);
                }
            }
        }
        files
    }
}

impl DataTransferItemListMethods for DataTransferItemList {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-length>
    fn Length(&self) -> u32 {
        self.data_transfer
            .root()
            .map_or(0, |_| self.items.borrow().len() as u32)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add>
    fn Add(
        &self,
        data: DOMString,
        mut type_: DOMString,
    ) -> Fallible<Option<DomRoot<DataTransferItem>>> {
        if self.has_write_permission() {
            type_.make_ascii_lowercase();
            for item in self.items.borrow().iter() {
                if item.type_matches(&type_) {
                    return Err(Error::NotSupported);
                }
            }

            let item = DataTransferItem::new(
                &self.global(),
                type_,
                Kind::Text(data),
                self.data_transfer.root().as_deref(),
            );
            self.items.borrow_mut().push(Dom::from_ref(&item));

            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add>
    fn Add_(&self, data: &File) -> Fallible<Option<DomRoot<DataTransferItem>>> {
        if self.has_write_permission() {
            let mut type_ = data.file_type();
            type_.make_ascii_lowercase();

            let item = DataTransferItem::new(
                &self.global(),
                DOMString::from(type_),
                Kind::File(data),
                self.data_transfer.root().as_deref(),
            );
            self.items.borrow_mut().push(Dom::from_ref(&item));

            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-remove>
    fn Remove(&self, index: u32) -> Fallible<()> {
        if self.has_write_permission() {
            if (index as usize) < self.items.borrow().len() {
                self.items.borrow_mut().remove(index as usize);
            }
            Ok(())
        } else {
            Err(Error::InvalidState)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-clear>
    fn Clear(&self) {
        if self.has_write_permission() {
            self.items.borrow_mut().clear();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-item>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<DataTransferItem>> {
        self.data_transfer.root().and_then(|_| {
            self.items
                .borrow()
                .get(index as usize)
                .map(|item| DomRoot::from_ref(&**item))
        })
    }
}
