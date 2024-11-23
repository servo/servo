/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use dom_struct::dom_struct;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::DataTransferItemBinding::{
    DataTransferItemMethods, FunctionStringCallback,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;
use crate::drag_data_store::{DragDataStore, Kind, Mode};

#[dom_struct]
pub struct DataTransferItem {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "Rc"]
    #[no_trace]
    data_store: Rc<RefCell<Option<DragDataStore>>>,
    index: usize,
}

impl DataTransferItem {
    fn new_inherited(
        data_store: Rc<RefCell<Option<DragDataStore>>>,
        index: usize,
    ) -> DataTransferItem {
        DataTransferItem {
            reflector_: Reflector::new(),
            data_store,
            index,
        }
    }

    pub fn new(
        global: &GlobalScope,
        data_store: Rc<RefCell<Option<DragDataStore>>>,
        index: usize,
    ) -> DomRoot<DataTransferItem> {
        reflect_dom_object(
            Box::new(DataTransferItem::new_inherited(data_store, index)),
            global,
        )
    }
}

impl DataTransferItemMethods for DataTransferItem {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-kind>
    fn Kind(&self) -> DOMString {
        match self
            .data_store
            .borrow()
            .as_ref()
            .and_then(|data_store| data_store.get_item(self.index))
        {
            // Step 1 Return the empty string if it isn't associated with a data store
            None => DOMString::new(),
            // Step 2
            Some(Kind::Text(_)) => DOMString::from("string"),
            Some(Kind::File(_)) => DOMString::from("file"),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-type>
    fn Type(&self) -> DOMString {
        match self
            .data_store
            .borrow()
            .as_ref()
            .and_then(|data_store| data_store.get_item(self.index))
        {
            // Step 1 Return the empty string if it isn't associated with a data store
            None => DOMString::new(),
            // Step 2
            Some(item) => item.type_(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasstring>
    fn GetAsString(&self, callback: Option<Rc<FunctionStringCallback>>) {
        let option = self.data_store.borrow();
        let data_store = match option.as_ref() {
            Some(value) if value.mode() != Mode::Protected => value,
            _ => return,
        };

        if let (Some(callback), Some(data)) = (
            callback,
            data_store
                .get_item(self.index)
                .and_then(|item| item.as_string()),
        ) {
            let _ = callback.Call__(data, ExceptionHandling::Report);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasfile>
    fn GetAsFile(&self) -> Option<DomRoot<File>> {
        let option = self.data_store.borrow();
        let data_store = match option.as_ref() {
            Some(value) if value.mode() != Mode::Protected => value,
            _ => return None,
        };

        data_store
            .get_item(self.index)
            .and_then(|item| item.as_file(&self.global()))
    }
}
