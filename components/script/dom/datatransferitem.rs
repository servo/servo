/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DataTransferItemBinding::{
    DataTransferItemMethods, FunctionStringCallback,
};
use crate::dom::bindings::import::module::Rc;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::weakref::MutableWeakRef;
use crate::dom::datatransfer::DataTransfer;
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;

#[derive(JSTraceable, MallocSizeOf)]
pub enum Kind {
    Text(DOMString),
    File(DomRoot<File>),
}

#[dom_struct]
pub struct DataTransferItem {
    reflector_: Reflector,
    type_: DomRefCell<DOMString>,
    item: Kind,
    data_transfer: MutableWeakRef<DataTransfer>,
}

impl DataTransferItem {
    pub fn new_inherited(
        type_: DOMString,
        item: Kind,
        data_transfer: Option<&DataTransfer>,
    ) -> DataTransferItem {
        DataTransferItem {
            reflector_: Reflector::new(),
            type_: DomRefCell::new(type_),
            item,
            data_transfer: MutableWeakRef::new(data_transfer),
        }
    }

    pub fn new(
        global: &GlobalScope,
        type_: DOMString,
        item: Kind,
        data_transfer: Option<&DataTransfer>,
    ) -> DomRoot<DataTransferItem> {
        reflect_dom_object(
            Box::new(DataTransferItem::new_inherited(type_, item, data_transfer)),
            global,
        )
    }

    pub fn type_already_present(&self, type_: &DOMString) -> bool {
        matches!(self.item, Kind::Text(_) if self.type_.borrow().eq(type_))
    }

    pub fn set_data_transfer(&self, data_transfer: Option<&DataTransfer>) {
        self.data_transfer.set(data_transfer);
    }
}

impl DataTransferItemMethods for DataTransferItem {
    // https://html.spec.whatwg.org/multipage/#dom-datatransferitem-kind
    fn Kind(&self) -> DOMString {
        self.data_transfer
            .root()
            .map_or(DOMString::from(""), |_| match self.item {
                Kind::Text(_) => DOMString::from("string"),
                Kind::File(_) => DOMString::from("file"),
            })
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransferitem-type
    fn Type(&self) -> DOMString {
        self.data_transfer
            .root()
            .map_or(DOMString::from(""), |_| self.type_.borrow().clone())
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasstring
    fn GetAsString(&self, callback: Option<Rc<FunctionStringCallback>>) {
        if let Some(callback) = callback {
            if self
                .data_transfer
                .root()
                .is_some_and(|data_transfer| data_transfer.can_read())
            {
                match &self.item {
                    Kind::Text(data) => {
                        let _ = callback.Call__(data.clone(), ExceptionHandling::Report);
                    },
                    Kind::File(_) => {},
                }
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasfile
    fn GetAsFile(&self) -> Option<DomRoot<File>> {
        if self
            .data_transfer
            .root()
            .is_some_and(|data_transfer| data_transfer.can_read())
        {
            return match &self.item {
                Kind::Text(_) => None,
                Kind::File(file) => Some(file.clone()),
            };
        }
        None
    }
}
