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
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::weakref::MutableWeakRef;
use crate::dom::datatransfer::DataTransfer;
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;

pub enum Kind<'a> {
    Text(DOMString),
    File(&'a File),
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
enum KindStorage {
    Text(DOMString),
    File(Dom<File>),
}

#[dom_struct]
pub struct DataTransferItem {
    reflector_: Reflector,
    type_: DomRefCell<DOMString>,
    item: KindStorage,
    data_transfer: MutableWeakRef<DataTransfer>,
}

impl DataTransferItem {
    fn new_inherited(
        type_: DOMString,
        item: Kind,
        data_transfer: Option<&DataTransfer>,
    ) -> DataTransferItem {
        DataTransferItem {
            reflector_: Reflector::new(),
            type_: DomRefCell::new(type_),
            item: match item {
                Kind::Text(text) => KindStorage::Text(text),
                Kind::File(file) => KindStorage::File(Dom::from_ref(file)),
            },
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

    pub fn type_matches(&self, type_: &DOMString) -> bool {
        matches!(self.item, KindStorage::Text(_) if self.type_.borrow().eq(type_))
    }

    pub fn is_file_kind(&self) -> bool {
        matches!(self.item, KindStorage::File(_))
    }

    pub fn get_string_of_type(&self, type_: &DOMString) -> Option<DOMString> {
        match self.item {
            KindStorage::Text(ref data) if self.type_.borrow().eq(type_) => Some(data.clone()),
            _ => None,
        }
    }

    pub fn get_as_file(&self) -> Option<DomRoot<File>> {
        if let KindStorage::File(file) = &self.item {
            Some(DomRoot::from_ref(file))
        } else {
            None
        }
    }

    pub fn set_data_transfer(&self, data_transfer: Option<&DataTransfer>) {
        self.data_transfer.set(data_transfer);
    }

    pub fn kind(&self) -> Kind {
        match &self.item {
            KindStorage::Text(text) => Kind::Text(text.clone()),
            KindStorage::File(file) => Kind::File(file),
        }
    }
}

impl DataTransferItemMethods for DataTransferItem {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-kind>
    fn Kind(&self) -> DOMString {
        self.data_transfer
            .root()
            .map_or(DOMString::from(""), |_| match self.item {
                KindStorage::Text(_) => DOMString::from("string"),
                KindStorage::File(_) => DOMString::from("file"),
            })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-type>
    fn Type(&self) -> DOMString {
        self.data_transfer
            .root()
            .map_or(DOMString::from(""), |_| self.type_.borrow().clone())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasstring>
    fn GetAsString(&self, callback: Option<Rc<FunctionStringCallback>>) {
        if let Some(callback) = callback {
            if self
                .data_transfer
                .root()
                .is_some_and(|data_transfer| data_transfer.can_read())
            {
                if let KindStorage::Text(data) = &self.item {
                    let _ = callback.Call__(data.clone(), ExceptionHandling::Report);
                }
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasfile>
    fn GetAsFile(&self) -> Option<DomRoot<File>> {
        self.data_transfer
            .root()
            .filter(|data_transfer| data_transfer.can_read())
            .and_then(|_| self.get_as_file())
    }
}
