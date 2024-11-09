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

impl KindStorage {
    fn new(kind: Kind) -> KindStorage {
        match kind {
            Kind::Text(text) => KindStorage::Text(text),
            Kind::File(file) => KindStorage::File(Dom::from_ref(file)),
        }
    }
}

#[dom_struct]
pub struct DataTransferItem {
    reflector_: Reflector,
    kind: KindStorage,
    type_: DomRefCell<DOMString>,
    data_store: MutableWeakRef<DataTransfer>,
}

impl DataTransferItem {
    fn new_inherited(
        kind: Kind,
        type_: DOMString,
        data_transfer: Option<&DataTransfer>,
    ) -> DataTransferItem {
        DataTransferItem {
            reflector_: Reflector::new(),
            kind: KindStorage::new(kind),
            type_: DomRefCell::new(type_),
            data_store: MutableWeakRef::new(data_transfer),
        }
    }

    pub fn new(
        global: &GlobalScope,
        type_: DOMString,
        kind: Kind,
        data_transfer: Option<&DataTransfer>,
    ) -> DomRoot<DataTransferItem> {
        reflect_dom_object(
            Box::new(DataTransferItem::new_inherited(kind, type_, data_transfer)),
            global,
        )
    }

    pub fn type_(&self) -> DOMString {
        self.type_.borrow().clone()
    }

    pub fn as_file(&self) -> Option<DomRoot<File>> {
        match &self.kind {
            KindStorage::File(file) => Some(DomRoot::from_ref(file)),
            _ => None,
        }
    }

    pub fn text_type_matches(&self, type_: &DOMString) -> bool {
        matches!(self.kind, KindStorage::Text(_) if self.type_.borrow().eq(type_))
    }
}

impl DataTransferItemMethods for DataTransferItem {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-kind>
    fn Kind(&self) -> DOMString {
        // Step 1 Return the empty string if it isn't associated with a data store
        if self.data_store.root().is_none() {
            return DOMString::new();
        }

        // Step 2
        match self.kind {
            KindStorage::Text(_) => DOMString::from("string"),
            KindStorage::File(_) => DOMString::from("file"),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-type>
    fn Type(&self) -> DOMString {
        // Step 1 Return the empty string if it isn't associated with a data store
        if self.data_store.root().is_none() {
            return DOMString::new();
        }

        // Step 2
        self.type_()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasstring>
    fn GetAsString(&self, callback: Option<Rc<FunctionStringCallback>>) {
        if self
            .data_store
            .root()
            .is_some_and(|data_store| data_store.can_read())
        {
            if let (Some(callback), KindStorage::Text(data)) = (callback, &self.kind) {
                let _ = callback.Call__(data.clone(), ExceptionHandling::Report);
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasfile>
    fn GetAsFile(&self) -> Option<DomRoot<File>> {
        self.data_store
            .root()
            .filter(|data_store| data_store.can_read())
            .and_then(|_| self.as_file())
    }
}
