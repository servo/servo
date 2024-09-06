/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DataTransferItemListBinding::DataTransferItemListMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::weakref::MutableWeakRef;
use crate::dom::datatransfer::DataTransfer;
use crate::dom::datatransferitem::{DataTransferItem, Kind};
use crate::dom::file::File;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub struct DataTransferItemList {
    reflector_: Reflector,
    data_transfer: MutableWeakRef<DataTransfer>,
    items: DomRefCell<Vec<DomRoot<DataTransferItem>>>,
}

impl DataTransferItemList {
    pub fn new_inherited() -> DataTransferItemList {
        DataTransferItemList {
            reflector_: Reflector::new(),
            data_transfer: MutableWeakRef::new(None),
            items: DomRefCell::new(Vec::new()),
        }
    }

    pub fn new(window: &Window, proto: Option<HandleObject>) -> DomRoot<DataTransferItemList> {
        reflect_dom_object_with_proto(
            Box::new(DataTransferItemList::new_inherited()),
            window,
            proto,
            CanGc::note(),
        )
    }

    pub fn set_data_transfer(&self, data_transfer: Option<&DataTransfer>) {
        self.data_transfer.set(data_transfer);
        for item in self.items.borrow_mut().iter() {
            item.set_data_transfer(data_transfer);
        }
    }
}

impl DataTransferItemListMethods for DataTransferItemList {
    // https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-length
    fn Length(&self) -> u32 {
        self.data_transfer
            .root()
            .map_or(0, |_| self.items.borrow().len() as u32)
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add
    fn Add(
        &self,
        data: DOMString,
        mut type_: DOMString,
    ) -> Fallible<Option<DomRoot<DataTransferItem>>> {
        if self
            .data_transfer
            .root()
            .is_some_and(|data_transfer| data_transfer.can_write())
        {
            type_.make_ascii_lowercase();
            for item in self.items.borrow().iter() {
                if item.type_already_present(&type_) {
                    return Err(Error::NotSupported);
                }
            }

            let item = DataTransferItem::new(
                &self.global(),
                type_,
                Kind::Text(data),
                self.data_transfer.root().as_deref(),
            );
            self.items.borrow_mut().push(item.clone());

            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add
    fn Add_(&self, data: &File) -> Fallible<Option<DomRoot<DataTransferItem>>> {
        if self
            .data_transfer
            .root()
            .is_some_and(|data_transfer| data_transfer.can_write())
        {
            let mut type_ = data.file_type();
            type_.make_ascii_lowercase();
            let item = DataTransferItem::new(
                &self.global(),
                DOMString::from(type_),
                Kind::File(DomRoot::from_ref(data)),
                self.data_transfer.root().as_deref(),
            );
            self.items.borrow_mut().push(item.clone());
            return Ok(Some(item));
        }
        Ok(None)
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-remove
    fn Remove(&self, index: u32) -> Fallible<()> {
        if self
            .data_transfer
            .root()
            .is_some_and(|data_transfer| data_transfer.can_write())
        {
            if (index as usize) < self.items.borrow().len() {
                self.items.borrow_mut().remove(index as usize);
            }
            return Ok(());
        }
        Err(Error::InvalidState)
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-clear
    fn Clear(&self) {
        if self
            .data_transfer
            .root()
            .is_some_and(|data_transfer| data_transfer.can_write())
        {
            self.items.borrow_mut().clear();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-item
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<DataTransferItem>> {
        self.data_transfer
            .root()
            .and_then(|_| self.items.borrow().get(index as usize).cloned())
    }
}
